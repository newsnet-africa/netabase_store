use proc_macro2::Span;
use syn::Error;

/// Utilities for generating consistent error messages

pub fn multiple_primary_keys(span: Span) -> Error {
    Error::new(
        span,
        "Multiple primary keys found. A model must have exactly one field marked with #[primary_key]"
    )
}

pub fn no_primary_key(span: Span) -> Error {
    Error::new(
        span,
        "No primary key found. A model must have exactly one field marked with #[primary_key]"
    )
}

pub fn invalid_link_target(span: Span, detail: &str) -> Error {
    Error::new(
        span,
        format!(
            "Invalid link target: {}. Links must be in the form #[link(Definition, Model)]",
            detail
        )
    )
}

pub fn invalid_blob_field(span: Span, detail: &str) -> Error {
    Error::new(
        span,
        format!("Invalid blob field: {}", detail)
    )
}

pub fn invalid_subscription(span: Span, detail: &str) -> Error {
    Error::new(
        span,
        format!("Invalid subscription: {}", detail)
    )
}

pub fn unsupported_field_type(span: Span, detail: &str) -> Error {
    Error::new(
        span,
        format!("Unsupported field type: {}", detail)
    )
}

pub fn duplicate_field_attribute(span: Span, attr_name: &str) -> Error {
    Error::new(
        span,
        format!(
            "Duplicate field attribute '{}'. Each field can only have one key attribute",
            attr_name
        )
    )
}

pub fn nested_definition_permission_error(span: Span, detail: &str) -> Error {
    Error::new(
        span,
        format!(
            "Permission structure violation: {}. Parent definitions have full access to children, but siblings/cousins need explicit permissions for relational linking",
            detail
        )
    )
}
