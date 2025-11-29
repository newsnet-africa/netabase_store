use syn::{Field, PathSegment, Token, Type, punctuated::Punctuated};

pub struct ModelKeyInfo<'ast> {
    pub primary_keys: &'ast Field,
    pub secondary_keys: Vec<&'ast Field>,
}

#[allow(dead_code)]
pub struct ModelLinkInfo<'ast> {
    pub link_path: Punctuated<PathSegment, Token![::]>,
    pub link_field: &'ast Field,
    pub is_relational_link: bool,
    pub linked_type: Option<&'ast Type>,
}

impl<'ast> ModelLinkInfo<'ast> {
    /// Check if a field type is a RelationalLink<D, M>
    pub fn is_relational_link_type(field_type: &Type) -> bool {
        if let Type::Path(type_path) = field_type {
            if let Some(segment) = type_path.path.segments.last() {
                return segment.ident == "RelationalLink";
            }
        }
        false
    }

    /// Extract the linked model type from RelationalLink<D, M>
    pub fn extract_linked_type(field_type: &Type) -> Option<&Type> {
        if let Type::Path(type_path) = field_type {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "RelationalLink" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        // Return the second type argument (M) from RelationalLink<D, M>
                        if args.args.len() >= 2 {
                            if let Some(syn::GenericArgument::Type(linked_type)) =
                                args.args.iter().nth(1)
                            {
                                return Some(linked_type);
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
