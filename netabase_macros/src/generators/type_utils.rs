//! Type analysis utilities for zero-copy implementations
//!
//! This module provides centralized type analysis for:
//! - Fixed-width type detection and size calculation
//! - Borrowed type mapping (String → &str)
//! - Type capability checks (borrowable, copyable, etc.)
//!
//! Used by both zerocopy generator and model_key generator.

#![allow(dead_code)] // Some utilities reserved for future use
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Fields, FieldsNamed, Type};

/// Calculate the total fixed width of all fields in a struct.
///
/// Returns `Some(total_bytes)` if all fields are fixed-width, `None` otherwise.
///
/// This is used for redb's `fixed_width()` optimization - fixed-width types
/// can be stored more efficiently without length prefixes.
///
/// # Examples
///
/// ```no_run
/// // struct User { id: u64, age: u32 } → Some(12)
/// // struct User { id: u64, name: String } → None (String is variable-width)
/// ```
pub fn calculate_fixed_width(fields: &FieldsNamed) -> Option<usize> {
    let mut total_width = 0;

    for field in &fields.named {
        match get_type_width(&field.ty) {
            Some(width) => total_width += width,
            None => return None, // One variable-width field makes entire struct variable
        }
    }

    Some(total_width)
}

/// Calculate the fixed width for field collection
///
/// Wrapper that handles Fields enum and delegates to calculate_fixed_width
pub fn calculate_fields_width(fields: &Fields) -> Option<usize> {
    match fields {
        Fields::Named(named) => calculate_fixed_width(named),
        Fields::Unnamed(_) => None, // Not supported
        Fields::Unit => Some(0),
    }
}

/// Get the size in bytes for a type, or None if variable-width.
///
/// # Supported Fixed-Width Types
///
/// - Primitives: u8/i8 (1), u16/i16 (2), u32/i32/f32 (4), u64/i64/f64 (8), u128/i128 (16)
/// - bool (1), char (4 in Rust)
/// - Arrays: [T; N] where T is fixed-width (T.width * N)
/// - Tuples: (T, U, ...) where all elements are fixed-width (sum of widths)
///
/// # Variable-Width Types
///
/// - String, Vec<T>, Option<T> (require length prefix)
/// - Any type containing variable-width fields
///
/// # Examples
///
/// ```
/// # use syn::{Type, parse_quote};
/// # fn get_type_width(ty: &Type) -> Option<usize> {
/// #     // Simplified implementation for doc test
/// #     match ty {
/// #         Type::Path(tp) => {
/// #             let name = tp.path.segments.last()?.ident.to_string();
/// #             match name.as_str() {
/// #                 "u64" | "i64" => Some(8),
/// #                 "u32" | "i32" => Some(4),
/// #                 "String" => None,
/// #                 _ => None,
/// #             }
/// #         }
/// #         Type::Array(arr) => {
/// #             if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(n), .. }) = &arr.len {
/// #                 let count: usize = n.base10_parse().ok()?;
/// #                 get_type_width(&arr.elem).map(|w| w * count)
/// #             } else { None }
/// #         }
/// #         _ => None,
/// #     }
/// # }
/// assert_eq!(get_type_width(&parse_quote!(u64)), Some(8));
/// assert_eq!(get_type_width(&parse_quote!(String)), None);
/// assert_eq!(get_type_width(&parse_quote!([u32; 4])), Some(16));
/// ```
pub fn get_type_width(ty: &Type) -> Option<usize> {
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            let last_segment = path.segments.last()?;
            let type_name = last_segment.ident.to_string();

            match type_name.as_str() {
                // Integer types
                "u8" | "i8" => Some(1),
                "u16" | "i16" => Some(2),
                "u32" | "i32" | "f32" => Some(4),
                "u64" | "i64" | "f64" => Some(8),
                "u128" | "i128" => Some(16),

                // Other primitives
                "bool" => Some(1),
                "char" => Some(4), // Rust char is 4 bytes (Unicode scalar value)

                // Variable-width types
                "String" | "Vec" => None,

                // Option<T> is variable-width because it needs a discriminant
                // Even Option<u64> needs a byte for Some/None tag
                "Option" => None,

                // Unknown types - assume variable width for safety
                _ => None,
            }
        }

        // Arrays: [T; N]
        Type::Array(array) => {
            let elem_width = get_type_width(&array.elem)?;

            // Extract array length from const expression
            if let syn::Expr::Lit(expr_lit) = &array.len
                && let syn::Lit::Int(lit_int) = &expr_lit.lit
            {
                let len: usize = lit_int.base10_parse().ok()?;
                return Some(elem_width * len);
            }

            None
        }

        // Tuples: (T, U, V, ...)
        Type::Tuple(tuple) => {
            let mut total = 0;
            for elem in &tuple.elems {
                total += get_type_width(elem)?;
            }
            Some(total)
        }

        // References, pointers, slices are not stored directly
        Type::Reference(_) | Type::Ptr(_) | Type::Slice(_) => None,

        // Other types assumed variable-width
        _ => None,
    }
}

/// Map owned type to its borrowed form for zero-copy reads.
///
/// This is the core function for generating borrowed reference types.
///
/// # Type Mappings
///
/// - `String` → `&'a str`
/// - `Vec<u8>` → `&'a [u8]`
/// - `Option<String>` → `Option<&'a str>`
/// - `Option<Vec<u8>>` → `Option<&'a [u8]>`
/// - Primitives (u8, u64, etc.) → unchanged (Copy types)
/// - `[T; N]` → `[T::Borrowed; N]`
/// - `(T, U)` → `(T::Borrowed, U::Borrowed)`
///
/// # Examples
///
/// ```ignore
/// map_to_borrowed_type(&parse_quote!(String)) // → &'a str
/// map_to_borrowed_type(&parse_quote!(u64)) // → u64
/// map_to_borrowed_type(&parse_quote!(Option<String>)) // → Option<&'a str>
/// ```
pub fn map_to_borrowed_type(ty: &Type) -> TokenStream {
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            let last_segment = path.segments.last().unwrap();
            let type_name = last_segment.ident.to_string();

            match type_name.as_str() {
                // String -> &'a str
                "String" => quote! { &'a str },

                // Vec<u8> -> &'a [u8]
                "Vec" => {
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                        && is_u8_type(inner)
                    {
                        return quote! { &'a [u8] };
                    }
                    quote! {
                        compile_error!("Vec<T> is only supported for T = u8. Use Vec<u8> or store as String.");
                    }
                }

                // Option<T> -> Option<T::Borrowed>
                "Option" => {
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments
                        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                    {
                        let inner_borrowed = map_to_borrowed_type(inner);
                        return quote! { Option<#inner_borrowed> };
                    }
                    quote! { #ty }
                }

                // Primitives stay the same (Copy)
                "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128"
                | "f32" | "f64" | "bool" | "char" => quote! { #ty },

                // Unsupported types
                _ => quote! {
                    compile_error!(concat!(
                        "Type '", stringify!(#ty), "' is not supported for zero-copy. ",
                        "Supported types: u8-u128, i8-i128, f32, f64, bool, char, String, Vec<u8>, ",
                        "Option<T> where T is supported. See ZERO_COPY_IMPLEMENTATION.md for details."
                    ));
                },
            }
        }

        // Arrays: [T; N] -> [T::Borrowed; N]
        Type::Array(array) => {
            let elem_borrowed = map_to_borrowed_type(&array.elem);
            let len = &array.len;
            quote! { [#elem_borrowed; #len] }
        }

        // Tuples: (T, U) -> (T::Borrowed, U::Borrowed)
        Type::Tuple(tuple) => {
            let elem_borrowed: Vec<TokenStream> =
                tuple.elems.iter().map(map_to_borrowed_type).collect();
            quote! { (#(#elem_borrowed),*) }
        }

        _ => quote! {
            compile_error!("Unsupported type for zero-copy. See ZERO_COPY_IMPLEMENTATION.md.");
        },
    }
}

/// Check if a type can be borrowed (has owned → borrowed conversion).
///
/// Returns true for:
/// - String (→ &str)
/// - Vec<u8> (→ &[u8])
/// - Option<String> (→ Option<&str>)
/// - Option<Vec<u8>> (→ Option<&[u8]>)
///
/// Used to determine which fields need to be included in the `#[borrows(...)]`
/// clause for ouroboros self-referential structs.
///
/// # Examples
///
/// ```ignore
/// is_borrowable_type(&parse_quote!(String)) // true
/// is_borrowable_type(&parse_quote!(u64)) // false
/// is_borrowable_type(&parse_quote!(Option<String>)) // true
/// ```
pub fn is_borrowable_type(ty: &Type) -> bool {
    is_string_type(ty) || is_vec_u8_type(ty) || is_option_string_or_vec(ty)
}

/// Check if type is `String`
pub fn is_string_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "String";
    }
    false
}

/// Check if type is `Vec<u8>`
pub fn is_vec_u8_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Vec"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return is_u8_type(inner);
    }
    false
}

/// Check if type is `u8`
pub fn is_u8_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "u8";
    }
    false
}

/// Check if type is `Option<String>` or `Option<Vec<u8>>`
pub fn is_option_string_or_vec(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return is_string_type(inner) || is_vec_u8_type(inner);
    }
    false
}

/// Check if type is `Option<T>`
pub fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}

/// Check if a type is a primitive copy type
///
/// Returns true for primitives that implement Copy (integers, floats, bool, char).
/// These types don't need borrowing and can be directly copied from database pages.
pub fn is_copy_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        let type_name = segment.ident.to_string();
        return matches!(
            type_name.as_str(),
            "u8" | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "f32"
                | "f64"
                | "bool"
                | "char"
        );
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_get_type_width_primitives() {
        assert_eq!(get_type_width(&parse_quote!(u8)), Some(1));
        assert_eq!(get_type_width(&parse_quote!(u16)), Some(2));
        assert_eq!(get_type_width(&parse_quote!(u32)), Some(4));
        assert_eq!(get_type_width(&parse_quote!(u64)), Some(8));
        assert_eq!(get_type_width(&parse_quote!(u128)), Some(16));
        assert_eq!(get_type_width(&parse_quote!(bool)), Some(1));
        assert_eq!(get_type_width(&parse_quote!(char)), Some(4));
    }

    #[test]
    fn test_get_type_width_variable() {
        assert_eq!(get_type_width(&parse_quote!(String)), None);
        assert_eq!(get_type_width(&parse_quote!(Vec<u8>)), None);
        assert_eq!(get_type_width(&parse_quote!(Option<u64>)), None);
    }

    #[test]
    fn test_get_type_width_array() {
        assert_eq!(get_type_width(&parse_quote!([u8; 4])), Some(4));
        assert_eq!(get_type_width(&parse_quote!([u32; 3])), Some(12));
        assert_eq!(get_type_width(&parse_quote!([u64; 2])), Some(16));
    }

    #[test]
    fn test_get_type_width_tuple() {
        assert_eq!(get_type_width(&parse_quote!((u8, u16))), Some(3));
        assert_eq!(get_type_width(&parse_quote!((u32, u64))), Some(12));
        assert_eq!(get_type_width(&parse_quote!((u8, u16, u32))), Some(7));
        // Tuple with variable-width element
        assert_eq!(get_type_width(&parse_quote!((u8, String))), None);
    }

    #[test]
    fn test_calculate_fixed_width() {
        // All fixed-width fields
        let fields: FieldsNamed = parse_quote!({
            id: u64,
            age: u32,
            active: bool
        });
        assert_eq!(calculate_fixed_width(&fields), Some(13)); // 8 + 4 + 1

        // Contains variable-width field
        let fields: FieldsNamed = parse_quote!({
            id: u64,
            name: String
        });
        assert_eq!(calculate_fixed_width(&fields), None);
    }

    #[test]
    fn test_map_string_to_borrowed() {
        let ty: Type = parse_quote!(String);
        let borrowed = map_to_borrowed_type(&ty);
        assert_eq!(borrowed.to_string(), "& 'a str");
    }

    #[test]
    fn test_map_vec_u8_to_borrowed() {
        let ty: Type = parse_quote!(Vec<u8>);
        let borrowed = map_to_borrowed_type(&ty);
        assert_eq!(borrowed.to_string(), "& 'a [u8]");
    }

    #[test]
    fn test_map_option_string_to_borrowed() {
        let ty: Type = parse_quote!(Option<String>);
        let borrowed = map_to_borrowed_type(&ty);
        assert_eq!(borrowed.to_string(), "Option < & 'a str >");
    }

    #[test]
    fn test_primitive_unchanged() {
        let ty: Type = parse_quote!(u64);
        let borrowed = map_to_borrowed_type(&ty);
        assert_eq!(borrowed.to_string(), "u64");
    }

    #[test]
    fn test_is_borrowable_type() {
        assert!(is_borrowable_type(&parse_quote!(String)));
        assert!(is_borrowable_type(&parse_quote!(Vec<u8>)));
        assert!(is_borrowable_type(&parse_quote!(Option<String>)));
        assert!(!is_borrowable_type(&parse_quote!(u64)));
        assert!(!is_borrowable_type(&parse_quote!(bool)));
    }

    #[test]
    fn test_is_copy_type() {
        assert!(is_copy_type(&parse_quote!(u8)));
        assert!(is_copy_type(&parse_quote!(u64)));
        assert!(is_copy_type(&parse_quote!(bool)));
        assert!(is_copy_type(&parse_quote!(char)));
        assert!(is_copy_type(&parse_quote!(f64)));
        assert!(!is_copy_type(&parse_quote!(String)));
        assert!(!is_copy_type(&parse_quote!(Vec<u8>)));
    }
}
