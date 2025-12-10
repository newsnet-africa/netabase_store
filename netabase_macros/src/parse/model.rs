//! Model parsing using syn visitors
//!
//! This module provides visitors for parsing model structs and extracting
//! their metadata, including fields and attributes.

use syn::{visit::Visit, DeriveInput, Fields};

use super::attributes::{ModelAttributes, FieldAttributes};
use super::metadata::{ModelMetadata, FieldMetadata, ErrorCollector};

/// Visitor for parsing a model struct
pub struct ModelVisitor {
    /// Collected model metadata
    pub metadata: Option<ModelMetadata>,

    /// Accumulated errors during parsing
    pub errors: ErrorCollector,
}

impl ModelVisitor {
    /// Create a new model visitor
    pub fn new() -> Self {
        Self {
            metadata: None,
            errors: ErrorCollector::new(),
        }
    }

    /// Parse a model from a DeriveInput
    ///
    /// This is the main entry point for parsing a #[derive(NetabaseModel)] struct.
    pub fn parse_model(input: &DeriveInput) -> Result<ModelMetadata, syn::Error> {
        let mut visitor = Self::new();
        visitor.visit_derive_input(input);

        if visitor.errors.has_errors() {
            return Err(visitor.errors.into_result().unwrap_err());
        }

        visitor.metadata.ok_or_else(|| {
            syn::Error::new_spanned(input, "Failed to parse model metadata")
        })
    }

    /// Parse model attributes and fields
    fn parse_struct(&mut self, input: &DeriveInput) {
        // Parse model-level attributes using darling
        let model_attrs = match ModelAttributes::from_derive_input(input) {
            Ok(attrs) => attrs,
            Err(e) => {
                self.errors.add(e.into());
                return;
            }
        };

        // Create model metadata
        let mut model = ModelMetadata::new(
            model_attrs.ident.clone(),
            model_attrs.vis.clone()
        );

        // Add subscriptions
        for sub in model_attrs.subscribe {
            model.add_subscription(sub);
        }

        // Parse fields
        if let syn::Data::Struct(data_struct) = &input.data {
            if let Fields::Named(fields) = &data_struct.fields {
                for field in &fields.named {
                    match self.parse_field(field) {
                        Ok(field_meta) => model.add_field(field_meta),
                        Err(e) => self.errors.add(e),
                    }
                }
            } else {
                self.errors.add_spanned(
                    input,
                    "NetabaseModel only supports structs with named fields"
                );
            }
        } else {
            self.errors.add_spanned(
                input,
                "NetabaseModel can only be derived for structs"
            );
        }

        self.metadata = Some(model);
    }

    /// Parse a single field
    fn parse_field(&mut self, field: &syn::Field) -> Result<FieldMetadata, syn::Error> {
        // Parse field attributes using darling
        let field_attrs = FieldAttributes::from_field(field)?;

        // Validate that field has an identifier (not tuple struct)
        let field_name = field_attrs.ident.clone().ok_or_else(|| {
            syn::Error::new_spanned(field, "Field must have a name")
        })?;

        // Validate: only one of primary/secondary/relation
        if field_attrs.attribute_count() > 1 {
            return Err(syn::Error::new_spanned(
                field,
                format!(
                    "Field '{}' can only have one of: #[primary_key], #[secondary_key], #[relation]",
                    field_name
                )
            ));
        }

        // Create field metadata
        let mut field_meta = FieldMetadata::new(
            field_name,
            field_attrs.ty.clone(),
            field_attrs.vis.clone()
        );

        // Set field kind based on attributes (after validation)
        field_meta.is_primary_key = field_attrs.primary_key;
        field_meta.is_secondary_key = field_attrs.secondary_key;
        field_meta.is_relation = field_attrs.relation;
        field_meta.cross_definition_link = field_attrs.cross_definition_link;

        Ok(field_meta)
    }
}

impl<'ast> Visit<'ast> for ModelVisitor {
    fn visit_derive_input(&mut self, input: &'ast DeriveInput) {
        self.parse_struct(input);
    }
}

impl Default for ModelVisitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_simple_model() {
        let input: DeriveInput = parse_quote! {
            #[derive(NetabaseModel)]
            pub struct User {
                #[primary_key]
                pub id: u64,
                pub name: String,
            }
        };

        let model = ModelVisitor::parse_model(&input).unwrap();
        assert_eq!(model.name.to_string(), "User");
        assert_eq!(model.fields.len(), 2);

        let pk = model.primary_key_field().unwrap();
        assert_eq!(pk.name.to_string(), "id");
        assert!(pk.is_primary_key);
    }

    #[test]
    fn test_parse_model_with_secondary_keys() {
        let input: DeriveInput = parse_quote! {
            pub struct User {
                #[primary_key]
                id: u64,
                #[secondary_key]
                email: String,
                #[secondary_key]
                username: String,
                name: String,
            }
        };

        let model = ModelVisitor::parse_model(&input).unwrap();
        assert_eq!(model.secondary_key_fields().len(), 2);
        assert!(model.has_secondary_keys());
    }

    #[test]
    fn test_parse_model_with_subscriptions() {
        let input: DeriveInput = parse_quote! {
            #[subscribe(Updates, Premium)]
            pub struct User {
                #[primary_key]
                id: u64,
            }
        };

        let model = ModelVisitor::parse_model(&input).unwrap();
        assert_eq!(model.subscriptions.len(), 2);
        assert_eq!(model.subscriptions[0].to_string(), "Updates");
        assert_eq!(model.subscriptions[1].to_string(), "Premium");
    }

    #[test]
    fn test_error_no_primary_key() {
        let input: DeriveInput = parse_quote! {
            pub struct User {
                name: String,
            }
        };

        let model = ModelVisitor::parse_model(&input).unwrap();
        // Note: Validation happens separately in MetadataValidator
        assert!(model.primary_key_field().is_none());
    }

    #[test]
    fn test_error_conflicting_attributes() {
        let input: DeriveInput = parse_quote! {
            pub struct User {
                #[primary_key]
                #[secondary_key]
                id: u64,
            }
        };

        let result = ModelVisitor::parse_model(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_model_with_relations() {
        let input: DeriveInput = parse_quote! {
            pub struct User {
                #[primary_key]
                id: u64,
                #[relation]
                posts: Vec<PostId>,
            }
        };

        let model = ModelVisitor::parse_model(&input).unwrap();
        assert!(model.has_relations());
        assert_eq!(model.relational_fields().len(), 1);
    }
}
