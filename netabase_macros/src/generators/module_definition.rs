use syn::{Ident, ItemEnum};

use crate::{
    generators::module_definition::def_gen::{generate_enums, generate_into_inner},
    visitors::definitions_visitor::DefinitionsVisitor,
};

impl<'a> DefinitionsVisitor<'a> {
    pub fn generate_definitions(
        &self,
        definition: &Ident,
        definition_key: &Ident,
    ) -> (ItemEnum, ItemEnum) {
        generate_enums(&self.modules, definition, definition_key)
    }

    pub fn generate_definition_trait_impls(
        &self,
        definition: &Ident,
        definition_key: &Ident,
    ) -> proc_macro2::TokenStream {
        let models = self
            .modules
            .iter()
            .flat_map(|m| def_gen::generate_model_variants(&m.models, m.path.clone()));
        let _into_inner = generate_into_inner(models.collect());
        // panic!("{:?}", into_inner.to_string());
        quote::quote! {
            impl ::netabase_store::traits::definition::NetabaseDefinitionTrait for #definition {
                type Keys = #definition_key;
            }

            impl ::netabase_store::traits::definition::NetabaseDefinitionTraitKey for #definition_key {

            }

            impl ::netabase_store::traits::convert::ToIVec for #definition {}
            impl ::netabase_store::traits::convert::ToIVec for #definition_key {}


        }
    }
}

pub mod def_gen {
    use syn::{
        Arm, Ident, ItemEnum, ItemStruct, PathSegment, Token, Variant, parse_quote,
        punctuated::Punctuated,
    };

    use crate::item_info::netabase_definitions::ModuleInfo;

    pub fn generate_model_variants(
        structs: &[&ItemStruct],
        path: Punctuated<PathSegment, Token![::]>,
    ) -> Vec<Variant> {
        structs
            .iter()
            .map(|s| {
                let name = &s.ident;
                let mut inner_path = path.clone();
                inner_path.push(name.clone().into());
                parse_quote! {
                    #name(#inner_path)
                }
            })
            .collect()
    }
    pub fn generate_into_inner(variants: Vec<Variant>) -> proc_macro2::TokenStream {
        let arms: Vec<Arm> = variants
            .iter()
            .map(|v| {
                let id = &v.ident;
                parse_quote! {
                    Self::#id(x) => x.clone()
                }
            })
            .collect();
        quote::quote! {
            fn into_inner(self) -> Box<dyn NetabaseModelTrait<Self>> {
                match self {
                    #(#arms),*
                }
            }
        }
    }

    pub fn generate_model_key_variants(
        keys: &[Ident],
        path: Punctuated<PathSegment, Token![::]>,
    ) -> Vec<Variant> {
        keys.iter()
            .map(|name| {
                let mut inner_path = path.clone();
                inner_path.push(name.clone().into());
                parse_quote! {
                    #name(#inner_path)
                }
            })
            .collect()
    }
    pub fn generate_enums(
        modules: &Vec<ModuleInfo<'_>>,
        definition: &Ident,
        definition_key: &Ident,
    ) -> (ItemEnum, ItemEnum) {
        let models = modules
            .iter()
            .flat_map(|m| generate_model_variants(&m.models, m.path.clone()));
        let keys = modules
            .iter()
            .flat_map(|m| generate_model_key_variants(&m.keys, m.path.clone()));

        let mut def_enum: ItemEnum = parse_quote! {
            #[derive(Debug, Clone, ::netabase_deps::strum::EnumDiscriminants,
                ::netabase_deps::derive_more::From,::netabase_deps::derive_more::TryInto,
                ::netabase_deps::bincode::Encode, ::netabase_deps::bincode::Decode,
                ::netabase_deps::strum::Display
            )]
            #[strum_discriminants(derive(Hash, ::netabase_deps::strum::EnumIter, ::netabase_deps::strum::EnumString,
            ::netabase_deps::strum::Display, ::netabase_deps::strum::AsRefStr))]
            pub enum #definition {
                #(#models),*
            }
        };

        if cfg!(feature = "uniffi") {
            let uniffi_attr: syn::Attribute = parse_quote!(#[derive(uniffi::Enum)]);
            def_enum.attrs.push(uniffi_attr);
        }

        let mut def_key_enum: ItemEnum = parse_quote! {
            #[derive(Debug, Clone, ::netabase_deps::strum::EnumDiscriminants,
                ::netabase_deps::derive_more::From, ::netabase_deps::derive_more::TryInto,
                ::netabase_deps::bincode::Encode, ::netabase_deps::bincode::Decode
            )]
            #[strum_discriminants(derive(Hash, ::netabase_deps::strum::EnumString,
            ::netabase_deps::strum::AsRefStr,
            ::netabase_deps::strum::Display))]
            pub enum #definition_key {
                #(#keys),*
            }
        };

        if cfg!(feature = "uniffi") {
            let uniffi_attr: syn::Attribute = parse_quote!(#[derive(uniffi::Enum)]);
            def_key_enum.attrs.push(uniffi_attr);
        }

        (def_enum, def_key_enum)
    }
}
