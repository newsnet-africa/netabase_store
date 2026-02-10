use crate::utils::naming::*;
use crate::visitors::model::field::{FieldKeyType, ModelFieldVisitor};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

/// Generator for wrapper types (ID type and field wrapper types)
pub struct WrapperTypeGenerator<'a> {
    visitor: &'a ModelFieldVisitor,
    /// Flag to control whether to generate ID type for this model
    generate_id: bool,
}

impl<'a> WrapperTypeGenerator<'a> {
    pub fn new(visitor: &'a ModelFieldVisitor) -> Self {
        Self {
            visitor,
            generate_id: true,
        }
    }

    /// Create a new generator with explicit control over ID generation
    pub fn with_id_generation(visitor: &'a ModelFieldVisitor, generate_id: bool) -> Self {
        Self {
            visitor,
            generate_id,
        }
    }

    /// Generate all wrapper types for the model
    pub fn generate(&self) -> TokenStream {
        let mut output = TokenStream::new();

        // Generate primary key type if flagged to do so
        if self.generate_id {
            output.extend(self.generate_primary_key_type());
        }

        // Generate secondary key wrapper types
        for field in &self.visitor.secondary_keys {
            output.extend(self.generate_field_wrapper(&field.name, &field.ty));
        }

        // Generate relational key wrapper types
        for field in &self.visitor.relational_keys {
            if let FieldKeyType::Relational { model, .. } = &field.key_type {
                output.extend(self.generate_relational_wrapper(&field.name, model));
            }
        }

        output
    }

    fn generate_primary_key_type(&self) -> TokenStream {
        // Use family name for ID type if this is a versioned model
        let id_type_name = primary_key_type_name_for_model(self.visitor);
        let model_name = &self.visitor.model_name;

        let inner_type = if let Some(pk_field) = &self.visitor.primary_key {
            pk_field.ty.clone()
        } else if let Some(ca_config) = &self.visitor.content_addressed_config {
            // Content-addressed model: use key_type or default to [u8; 32]
            ca_config
                .key_type
                .clone()
                .unwrap_or_else(|| syn::parse_str::<Type>("[u8; 32]").unwrap())
        } else {
            panic!("Model must have a primary key or be content-addressed");
        };

        let inner_type_str = quote!(#inner_type).to_string();
        let doc = format!(
            "Primary key type for `{}`.\n\n\
            This is a type-safe newtype wrapper around `{}` that serves as the unique\n\
            identifier for `{}` records in the database.\n\n\
            # Example\n\n\
            ```rust\n\
            # use netabase_store::doc_example::*;\n\
            // Create a new ID\n\
            let id = {}(\"unique_id\".to_string());\n\
            \n\
            // IDs are comparable and hashable\n\
            let id2 = {}(\"unique_id\".to_string());\n\
            assert_eq!(id, id2);\n\
            \n\
            // Access inner value\n\
            let inner: String = id.0;\n\
            assert_eq!(inner, \"unique_id\");\n\
            ```\n\n\
            # Properties\n\n\
            - **Type-safe**: Prevents accidental mixing of IDs from different models\n\
            - **Serializable**: Implements serde traits for storage and network transmission\n\
            - **Display**: Can be printed and logged\n\
            - **Hashable**: Can be used as HashMap/HashSet keys\n\n\
            # Implementation Details\n\n\
            The inner value is public (`pub`) for direct access when needed.",
            model_name,
            inner_type_str,
            model_name,
            id_type_name,
            id_type_name
        );

        quote! {
            #[doc = #doc]
            #[derive(
                Clone, Eq, PartialEq, PartialOrd, Ord, Debug,
                serde::Serialize, serde::Deserialize,
                Hash, derive_more::Display
            )]
            pub struct #id_type_name(pub #inner_type);
        }
    }

    fn generate_field_wrapper(&self, field_name: &Ident, field_type: &Type) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let wrapper_name = field_wrapper_name(model_name, field_name);
        let field_type_str = quote!(#field_type).to_string();

        let doc = format!(
            "Type-safe wrapper for the `{}` field of `{}`.\n\n\
            This wrapper is used in secondary key queries and ensures type safety when\n\
            querying by this specific field.\n\n\
            # Example\n\n\
            ```rust\n\
            # use netabase_store::doc_example::*;\n\
            // Create a field wrapper\n\
            let field_value = {}(\"example_value\".into());\n\
            \n\
            // Use in a secondary key\n\
            let key = {}SecondaryKeys::{}(field_value);\n\
            ```\n\n\
            # Type\n\n\
            Wraps: `{}`",
            field_name,
            model_name,
            wrapper_name,
            model_name,
            to_pascal_case(&field_name.to_string()),
            field_type_str
        );

        quote! {
            #[doc = #doc]
            #[derive(
                Clone, Eq, PartialEq, PartialOrd, Ord, Debug,
                serde::Serialize, serde::Deserialize,
                Hash, derive_more::Display
            )]
            pub struct #wrapper_name(pub #field_type);
        }
    }

    fn generate_relational_wrapper(
        &self,
        field_name: &Ident,
        target_model: &syn::Path,
    ) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let wrapper_name = field_wrapper_name(model_name, field_name);

        // The target model's ID type - we construct this by appending "ID" to the target model name
        let target_model_ident = crate::utils::naming::path_last_segment(target_model)
            .expect("Invalid target model path");
        let target_id_type = primary_key_type_name(target_model_ident);

        quote! {
            #[derive(
                Clone, Eq, PartialEq, PartialOrd, Ord, Debug,
                serde::Serialize, serde::Deserialize,
                Hash, derive_more::Display
            )]
            pub struct #wrapper_name(pub #target_id_type);
        }
    }
}
