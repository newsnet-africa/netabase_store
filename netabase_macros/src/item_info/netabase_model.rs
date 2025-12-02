use syn::{Field, Meta, PathSegment, Token, Type, punctuated::Punctuated};

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
    pub relation_name: Option<String>,
}

impl<'ast> ModelLinkInfo<'ast> {
    /// Check if a field type is a RelationalLink<D, M> (direct or wrapped)
    pub fn is_relational_link_type(field_type: &Type) -> bool {
        // Check direct RelationalLink
        if Self::is_direct_relational_link(field_type) {
            return true;
        }

        // Check wrapped types: Vec<RelationalLink<>>, Option<RelationalLink<>>, Box<RelationalLink<>>
        Self::extract_wrapped_relational_link(field_type).is_some()
    }

    /// Check if a type is directly RelationalLink<D, M> (not wrapped)
    fn is_direct_relational_link(field_type: &Type) -> bool {
        if let Type::Path(type_path) = field_type
            && let Some(segment) = type_path.path.segments.last()
        {
            return segment.ident == "RelationalLink";
        }
        false
    }

    /// Extract inner RelationalLink type from Vec<RelationalLink<>>, Option<RelationalLink<>>, or Box<RelationalLink<>>
    /// Returns None if the type is not a wrapped RelationalLink
    fn extract_wrapped_relational_link(field_type: &Type) -> Option<&Type> {
        if let Type::Path(type_path) = field_type
            && let Some(wrapper_segment) = type_path.path.segments.last()
            && let syn::PathArguments::AngleBracketed(wrapper_args) = &wrapper_segment.arguments
        {
            // Check if wrapper is Vec, Option, or Box
            let wrapper_name = wrapper_segment.ident.to_string();
            if wrapper_name == "Vec" || wrapper_name == "Option" || wrapper_name == "Box" {
                // Get the first generic argument (should be RelationalLink<D, M>)
                if let Some(syn::GenericArgument::Type(inner_type)) = wrapper_args.args.first() {
                    // Check if the inner type is RelationalLink
                    if Self::is_direct_relational_link(inner_type) {
                        return Some(inner_type);
                    }
                }
            }
        }
        None
    }

    /// Extract the linked model type from RelationalLink<D, M> (direct or wrapped)
    pub fn extract_linked_type(field_type: &Type) -> Option<&Type> {
        // Try direct RelationalLink first
        if let Some(linked_type) = Self::extract_linked_type_from_relational_link(field_type) {
            return Some(linked_type);
        }

        // Try wrapped RelationalLink
        if let Some(inner_relational_link) = Self::extract_wrapped_relational_link(field_type) {
            return Self::extract_linked_type_from_relational_link(inner_relational_link);
        }

        None
    }

    /// Extract the linked model type from a direct RelationalLink<D, M> type
    fn extract_linked_type_from_relational_link(relational_link_type: &Type) -> Option<&Type> {
        if let Type::Path(type_path) = relational_link_type
            && let Some(segment) = type_path.path.segments.last()
            && segment.ident == "RelationalLink"
            && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
            && args.args.len() >= 2
            && let Some(syn::GenericArgument::Type(linked_type)) = args.args.iter().nth(1)
        {
            return Some(linked_type);
        }
        None
    }

    /// Extract relation name from relation attribute
    /// Supports both:
    /// - #[relation(author)] where 'author' is the relation name
    /// - #[relation(name = "author")] where 'author' is the relation name
    pub fn extract_relation_name(attribute: &syn::Attribute) -> Option<String> {
        if let Meta::List(meta_list) = &attribute.meta {
            // First try to parse as name = value format: #[relation(name = "value")]
            if let Ok(parsed) = meta_list.parse_args::<syn::MetaNameValue>() {
                if parsed.path.is_ident("name")
                    && let syn::Expr::Lit(expr_lit) = &parsed.value
                    && let syn::Lit::Str(lit_str) = &expr_lit.lit
                {
                    return Some(lit_str.value());
                }
            }
            // Fallback to parsing as a single identifier: #[relation(author)]
            else if let Ok(ident) = meta_list.parse_args::<syn::Ident>() {
                return Some(ident.to_string());
            }
        }
        None
    }
}
