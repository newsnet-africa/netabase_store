/// Zero-copy borrowed type generation for redb backend
///
/// This module generates `*Ref<'a>` borrowed types that enable zero-copy reads
/// from redb by using tuple-based serialization and borrowed string slices.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Fields, Type, ItemStruct, Ident, Field, parse_quote};
use syn::visit_mut::{self, VisitMut};

// Import type utilities for shared functionality
use super::type_utils::{
    calculate_fields_width, is_borrowable_type, is_option_type, is_string_type, is_u8_type,
    is_vec_u8_type, map_to_borrowed_type as map_type_to_borrowed,
};

/// Visitor that injects the borrowed reference field into the user's struct
struct RefFieldInjector {
    borrowed_name: Ident,
}

impl VisitMut for RefFieldInjector {
    fn visit_item_struct_mut(&mut self, node: &mut ItemStruct) {
        // Only process if it has named fields
        if let Fields::Named(ref mut fields) = node.fields {
            let borrowed_name = &self.borrowed_name;

            // Create the injected field
            let injected_field: Field = parse_quote! {
                #[cfg(feature = "redb-zerocopy")]
                #[doc = "Cached borrowed view for zero-copy operations. Lazily initialized."]
                pub(crate) _borrowed_ref: ::std::cell::OnceCell<#borrowed_name<'static>>
            };

            // Add it to the fields
            fields.named.push(injected_field);
        }

        // Continue visiting
        visit_mut::visit_item_struct_mut(self, node);
    }
}

/// Inject the borrowed reference field into the user's struct
///
/// This modifies the struct AST to add a field like:
/// ```ignore
/// #[cfg(feature = "redb-zerocopy")]
/// _borrowed_ref: OnceCell<UserBorrowed<'static>>
/// ```
pub fn inject_ref_field(model: &mut ItemStruct) {
    let borrowed_name = Ident::new(&format!("{}Borrowed", model.ident), model.ident.span());
    let mut injector = RefFieldInjector { borrowed_name };
    injector.visit_item_struct_mut(model);
}

/// Generate the borrowed reference type for a model
///
/// For a struct like:
/// ```ignore
/// struct User {
///     id: u64,
///     name: String,
///     email: String,
/// }
/// ```
///
/// Generates:
/// ```ignore
/// struct UserRef<'a> {
///     id: u64,
///     name: &'a str,
///     email: &'a str,
/// }
/// ```
pub fn generate_borrowed_type(model: &ItemStruct) -> TokenStream {
    let model_name = &model.ident;
    let borrowed_name = Ident::new(&format!("{}Ref", model_name), model_name.span());
    let vis = &model.vis;

    let Fields::Named(fields) = &model.fields else {
        return quote! {
            compile_error!("RedbZeroCopy only supports structs with named fields");
        };
    };

    let borrowed_fields: Vec<TokenStream> = fields
        .named
        .iter()
        .map(|field| {
            let field_name = &field.ident;
            let field_vis = &field.vis;
            let borrowed_ty = map_type_to_borrowed(&field.ty);

            quote! {
                #field_vis #field_name: #borrowed_ty
            }
        })
        .collect();

    quote! {
        #[cfg(feature = "redb-zerocopy")]
        #[derive(Debug, Clone, Copy)]
        #vis struct #borrowed_name<'a> {
            #(#borrowed_fields),*
        }
    }
}

/// Generate the `as_ref()` conversion method
///
/// Generates:
/// ```ignore
/// impl User {
///     pub fn as_ref(&self) -> UserRef<'_> {
///         UserRef {
///             id: self.id,
///             name: self.name.as_str(),
///             email: self.email.as_str(),
///         }
///     }
/// }
/// ```
pub fn generate_as_ref_method(model: &ItemStruct) -> TokenStream {
    let model_name = &model.ident;
    let borrowed_name = Ident::new(&format!("{}Ref", model_name), model_name.span());

    let Fields::Named(fields) = &model.fields else {
        return quote! {};
    };

    let field_conversions: Vec<TokenStream> = fields
        .named
        .iter()
        .map(|field| {
            let field_name = &field.ident;
            let conversion = generate_to_borrowed_conversion(field_name.as_ref().unwrap(), &field.ty);
            quote! { #field_name: #conversion }
        })
        .collect();

    quote! {
        #[cfg(feature = "redb-zerocopy")]
        impl #model_name {
            /// Convert to borrowed form for zero-copy serialization
            pub fn as_ref(&self) -> #borrowed_name<'_> {
                #borrowed_name {
                    #(#field_conversions),*
                }
            }
        }
    }
}

/// Generate the ouroboros self-referential wrapper
///
/// This creates a struct that holds owned data and borrows from it:
/// ```ignore
/// #[ouroboros::self_referencing]
/// struct UserBorrowed {
///     id: u64,
///     name: String,
///     email: String,
///     #[borrows(name, email)]
///     #[not_covariant]
///     ref_view: UserRef<'this>,
/// }
/// ```
pub fn generate_ouroboros_wrapper(model: &ItemStruct) -> TokenStream {
    let model_name = &model.ident;
    let borrowed_name = Ident::new(&format!("{}Ref", model_name), model_name.span());
    let wrapper_name = Ident::new(&format!("{}Borrowed", model_name), model_name.span());

    let Fields::Named(fields) = &model.fields else {
        return quote! {};
    };

    // Fields for the wrapper (owned copies)
    let wrapper_fields: Vec<TokenStream> = fields
        .named
        .iter()
        .map(|field| {
            let field_name = &field.ident;
            let field_ty = &field.ty;
            quote! { #field_name: #field_ty }
        })
        .collect();

    // Field names that contain borrowable data (String, Vec<u8>)
    let borrowable_fields: Vec<&Ident> = fields
        .named
        .iter()
        .filter(|f| is_borrowable_type(&f.ty))
        .filter_map(|f| f.ident.as_ref())
        .collect();

    // Field names for construction
    let field_names: Vec<&Ident> = fields
        .named
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .collect();

    // Build the UserRef from borrowed fields
    let ref_construction: Vec<TokenStream> = fields
        .named
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            if is_string_type(&field.ty) {
                quote! { #field_name: fields.#field_name.as_str() }
            } else if is_vec_u8_type(&field.ty) {
                quote! { #field_name: fields.#field_name.as_slice() }
            } else if is_option_type(&field.ty) {
                // Need to handle Option<String> specially
                quote! { #field_name: fields.#field_name.as_deref() }
            } else {
                // Primitives - just copy/dereference
                quote! { #field_name: *fields.#field_name }
            }
        })
        .collect();

    let borrows_clause = if !borrowable_fields.is_empty() {
        quote! { #[borrows(#(#borrowable_fields),*)] }
    } else {
        quote! {}
    };

    quote! {
        #[cfg(feature = "redb-zerocopy")]
        #[derive(::ouroboros::self_referencing)]
        struct #wrapper_name {
            #(#wrapper_fields),*,

            #borrows_clause
            #[not_covariant]
            ref_view: #borrowed_name<'this>,
        }

        #[cfg(feature = "redb-zerocopy")]
        impl #wrapper_name {
            /// Create a new borrowed wrapper from owned data
            fn from_owned(value: #model_name) -> Self {
                #wrapper_name Builder {
                    #(#wrapper_fields: value.#field_names),*,
                    ref_view_builder: |fields| {
                        #borrowed_name {
                            #(#ref_construction),*
                        }
                    },
                }
                .build()
            }
        }
    }
}

// Note: is_borrowable_type, is_option_type, is_string_type, is_vec_u8_type, is_u8_type
// are now imported from type_utils module

/// Generate the `Borrow<UserRef<'_>>` trait implementation
///
/// This uses the injected `_borrowed_ref` field to cache the borrowed view.
pub fn generate_borrow_impl(model: &ItemStruct) -> TokenStream {
    let model_name = &model.ident;
    let wrapper_name = Ident::new(&format!("{}Borrowed", model_name), model_name.span());
    let borrowed_name = Ident::new(&format!("{}Ref", model_name), model_name.span());

    quote! {
        #[cfg(feature = "redb-zerocopy")]
        impl std::borrow::Borrow<#borrowed_name<'_>> for #model_name {
            fn borrow(&self) -> &#borrowed_name<'_> {
                // Initialize the cached borrowed view if not already done
                // Safety: This is safe because the ouroboros wrapper ensures the
                // borrowed data lives as long as the owned data
                self._borrowed_ref
                    .get_or_init(|| {
                        // Create the wrapper with a clone of self
                        #wrapper_name::from_owned(self.clone())
                    })
                    .borrow_ref_view()
            }
        }

        #[cfg(feature = "redb-zerocopy")]
        impl #model_name {
            /// Create an ouroboros self-referential wrapper for zero-copy borrowing
            ///
            /// This is useful when you need to return borrowed data from a function.
            /// The wrapper owns the data and provides a borrowed view into it.
            pub fn into_borrowed(self) -> #wrapper_name {
                #wrapper_name::from_owned(self)
            }

            /// Create a borrowed wrapper and execute a closure with the borrowed view
            ///
            /// This is the recommended way to work with zero-copy views when you
            /// need the data to outlive a function call.
            ///
            /// # Example
            /// ```ignore
            /// let user = User { /* ... */ };
            /// user.with_borrowed(|user_ref| {
            ///     println!("Name: {}", user_ref.name);  // Zero-copy!
            /// });
            /// ```
            pub fn with_borrowed<F, R>(self, f: F) -> R
            where
                F: for<'a> FnOnce(&#borrowed_name<'a>) -> R,
            {
                let wrapper = self.into_borrowed();
                wrapper.with_ref_view(f)
            }
        }
    }
}

/// Generate the `From` trait for converting borrowed back to owned
///
/// Generates:
/// ```ignore
/// impl<'a> From<UserRef<'a>> for User {
///     fn from(r: UserRef<'a>) -> Self {
///         User {
///             id: r.id,
///             name: r.name.to_owned(),
///             email: r.email.to_owned(),
///         }
///     }
/// }
/// ```
pub fn generate_from_borrowed(model: &ItemStruct) -> TokenStream {
    let model_name = &model.ident;
    let borrowed_name = Ident::new(&format!("{}Ref", model_name), model_name.span());

    let Fields::Named(fields) = &model.fields else {
        return quote! {};
    };

    let field_conversions: Vec<TokenStream> = fields
        .named
        .iter()
        .map(|field| {
            let field_name = &field.ident;
            let conversion = generate_to_owned_conversion(field_name.as_ref().unwrap(), &field.ty);
            quote! { #field_name: #conversion }
        })
        .collect();

    quote! {
        #[cfg(feature = "redb-zerocopy")]
        impl<'a> From<#borrowed_name<'a>> for #model_name {
            fn from(r: #borrowed_name<'a>) -> Self {
                #model_name {
                    #(#field_conversions),*
                }
            }
        }
    }
}

/// Generate tuple-based `redb::Value` implementation
///
/// This uses redb's native tuple support for zero-copy deserialization.
pub fn generate_value_impl(model: &ItemStruct) -> TokenStream {
    let model_name = &model.ident;
    let borrowed_name = Ident::new(&format!("{}Ref", model_name), model_name.span());

    let Fields::Named(fields) = &model.fields else {
        return quote! {
            compile_error!("RedbZeroCopy requires named fields");
        };
    };

    // Build tuple type for serialization: (u64, &str, &str, ...)
    let borrowed_types: Vec<TokenStream> = fields
        .named
        .iter()
        .map(|field| {
            let borrowed_ty = map_type_to_borrowed(&field.ty);
            quote! { #borrowed_ty }
        })
        .collect();

    let tuple_type = quote! { (#(#borrowed_types),*) };

    // Field names for tuple construction
    let field_names: Vec<&Ident> = fields
        .named
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .collect();

    // Build tuple from borrowed ref: (value.id, value.name, value.email, ...)
    let tuple_construction = quote! {
        (#(value.#field_names),*)
    };

    // Destructure tuple in from_bytes
    let tuple_destructure = quote! {
        let (#(#field_names),*) = #tuple_type::from_bytes(data);
    };

    // Calculate fixed width using type_utils
    let fixed_width_expr = match calculate_fields_width(&model.fields) {
        Some(width) => quote! { Some(#width) },
        None => quote! { None },
    };

    quote! {
        #[cfg(feature = "redb-zerocopy")]
        impl ::netabase_store::netabase_deps::redb::Value for #model_name {
            type SelfType<'a> = #borrowed_name<'a>
            where
                Self: 'a;

            type AsBytes<'a> = Vec<u8>
            where
                Self: 'a;

            fn fixed_width() -> Option<usize> {
                #fixed_width_expr
            }

            fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
            where
                Self: 'a,
            {
                #tuple_destructure
                #borrowed_name {
                    #(#field_names),*
                }
            }

            fn as_bytes<'a>(value: &'a Self::SelfType<'a>) -> Self::AsBytes<'a>
            where
                Self: 'a,
            {
                #tuple_type::as_bytes(&#tuple_construction)
            }

            fn type_name() -> ::netabase_store::netabase_deps::redb::TypeName {
                #tuple_type::type_name()
            }
        }
    }
}

// Note: map_to_borrowed_type is now imported as map_type_to_borrowed from type_utils module

/// Generate conversion from owned field to borrowed
///
/// Examples:
/// - `self.name` (String) → `self.name.as_str()`
/// - `self.id` (u64) → `self.id`
/// - `self.bio` (Option<String>) → `self.bio.as_deref()`
fn generate_to_borrowed_conversion(field_name: &Ident, ty: &Type) -> TokenStream {
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            let last_segment = path.segments.last().unwrap();
            let type_name = last_segment.ident.to_string();

            match type_name.as_str() {
                "String" => quote! { self.#field_name.as_str() },
                "Vec" => {
                    // Assume Vec<u8> (validation happens in map_to_borrowed_type)
                    quote! { self.#field_name.as_slice() }
                }
                "Option" => {
                    // Need to map the inner type
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                            if is_string_type(inner) {
                                return quote! { self.#field_name.as_deref() };
                            } else if is_vec_u8_type(inner) {
                                return quote! { self.#field_name.as_deref() };
                            }
                        }
                    }
                    quote! { self.#field_name }
                }
                // Primitives - just copy
                _ => quote! { self.#field_name },
            }
        }
        // Arrays and tuples - copy/clone
        _ => quote! { self.#field_name },
    }
}

/// Generate conversion from borrowed field to owned
///
/// Examples:
/// - `r.name` (&str) → `r.name.to_owned()`
/// - `r.id` (u64) → `r.id`
/// - `r.bio` (Option<&str>) → `r.bio.map(|s| s.to_owned())`
fn generate_to_owned_conversion(field_name: &Ident, ty: &Type) -> TokenStream {
    match ty {
        Type::Path(type_path) => {
            let path = &type_path.path;
            let last_segment = path.segments.last().unwrap();
            let type_name = last_segment.ident.to_string();

            match type_name.as_str() {
                "String" => quote! { r.#field_name.to_owned() },
                "Vec" => quote! { r.#field_name.to_vec() },
                "Option" => {
                    // Map to owned for inner type
                    if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                            if is_string_type(inner) {
                                return quote! { r.#field_name.map(|s| s.to_owned()) };
                            } else if is_vec_u8_type(inner) {
                                return quote! { r.#field_name.map(|s| s.to_vec()) };
                            }
                        }
                    }
                    quote! { r.#field_name }
                }
                // Primitives - just copy
                _ => quote! { r.#field_name },
            }
        }
        _ => quote! { r.#field_name },
    }
}

// Note: is_u8_type, is_string_type, is_vec_u8_type are now imported from type_utils module

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_map_string_to_borrowed() {
        let ty: Type = parse_quote! { String };
        let borrowed = map_type_to_borrowed(&ty);
        assert_eq!(borrowed.to_string(), "& 'a str");
    }

    #[test]
    fn test_map_vec_u8_to_borrowed() {
        let ty: Type = parse_quote! { Vec<u8> };
        let borrowed = map_type_to_borrowed(&ty);
        assert_eq!(borrowed.to_string(), "& 'a [u8]");
    }

    #[test]
    fn test_map_option_string_to_borrowed() {
        let ty: Type = parse_quote! { Option<String> };
        let borrowed = map_type_to_borrowed(&ty);
        assert_eq!(borrowed.to_string(), "Option < & 'a str >");
    }

    #[test]
    fn test_primitive_unchanged() {
        let ty: Type = parse_quote! { u64 };
        let borrowed = map_type_to_borrowed(&ty);
        assert_eq!(borrowed.to_string(), "u64");
    }
}
