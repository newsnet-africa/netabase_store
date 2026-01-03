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

        let pk_field = self.visitor.primary_key.as_ref().unwrap();
        let inner_type = &pk_field.ty;

        quote! {
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

        quote! {
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
