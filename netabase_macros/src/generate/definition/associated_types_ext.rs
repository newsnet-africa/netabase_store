//! ModelAssociatedTypesExt implementation generation
//!
//! Generates the implementation of ModelAssociatedTypesExt for the associated types enum.
//! This trait delegates wrapping operations to the individual models via NetabaseModelTrait.

use proc_macro2::TokenStream;
use quote::quote;
use crate::parse::metadata::ModuleMetadata;

/// Generate ModelAssociatedTypesExt implementation
pub fn generate_associated_types_ext(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let associated_types_name = quote::format_ident!("{}ModelAssociatedTypes", definition_name);

    quote! {
        impl netabase_store::traits::definition::ModelAssociatedTypesExt<#definition_name> for #associated_types_name {
            fn from_primary_key<M: netabase_store::traits::model::NetabaseModelTrait<#definition_name>>(key: M::PrimaryKey) -> Self {
                M::wrap_primary_key(key)
            }
            
            fn from_model<M: netabase_store::traits::model::NetabaseModelTrait<#definition_name>>(model: M) -> Self {
                M::wrap_model(model)
            }
            
            fn from_secondary_key<M: netabase_store::traits::model::NetabaseModelTrait<#definition_name>>(
                key: <<M::Keys as netabase_store::traits::model::key::NetabaseModelKeyTrait<#definition_name, M>>::SecondaryEnum as strum::IntoDiscriminant>::Discriminant
            ) -> Self {
                M::wrap_secondary_discriminant(key)
            }
            
            fn from_relational_key_discriminant<M: netabase_store::traits::model::NetabaseModelTrait<#definition_name>>(
                key: <<M::Keys as netabase_store::traits::model::key::NetabaseModelKeyTrait<#definition_name, M>>::RelationalEnum as strum::IntoDiscriminant>::Discriminant
            ) -> Self {
                M::wrap_relational_discriminant(key)
            }
            
            fn from_secondary_key_data<M: netabase_store::traits::model::NetabaseModelTrait<#definition_name>>(
                key: <M::Keys as netabase_store::traits::model::key::NetabaseModelKeyTrait<#definition_name, M>>::SecondaryEnum
            ) -> Self {
                M::wrap_secondary_key(key)
            }
            
            fn from_relational_key_data<M: netabase_store::traits::model::NetabaseModelTrait<#definition_name>>(
                key: <M::Keys as netabase_store::traits::model::key::NetabaseModelKeyTrait<#definition_name, M>>::RelationalEnum
            ) -> Self {
                M::wrap_relational_key(key)
            }

            fn from_subscription_key_discriminant<M: netabase_store::traits::model::NetabaseModelTrait<#definition_name>>(
                key: <<M as netabase_store::traits::model::NetabaseModelTrait<#definition_name>>::SubscriptionEnum as strum::IntoDiscriminant>::Discriminant
            ) -> Self {
                M::wrap_subscription_discriminant(key)
            }
        }
    }
}
