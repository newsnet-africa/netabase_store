use syn::{Ident, ItemEnum, parse_quote};

use crate::{
    generators::module_definition::def_gen::generate_enums,
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
}

pub mod def_gen {
    use syn::{
        Ident, ItemEnum, ItemStruct, PathSegment, Token, Variant, parse_quote,
        punctuated::Punctuated,
    };

    use crate::{item_info::netabase_definitions::ModuleInfo, util::append_ident};

    pub fn generate_model_variants(
        structs: &Vec<&ItemStruct>,
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
    pub fn generate_model_key_variants(
        keys: &Vec<Ident>,
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

        (
            parse_quote! {
                #[derive(Debug, ::netabase_deps::strum::EnumDiscriminants,
                    ::netabase_deps::derive_more::From,::netabase_deps::derive_more::TryInto,
                    ::netabase_deps::bincode::Encode, ::netabase_deps::bincode::Decode
                )]
                pub enum #definition {
                    #(#models),*
                }
            },
            parse_quote! {
                #[derive(Debug, ::netabase_deps::strum::EnumDiscriminants,
                    ::netabase_deps::derive_more::From, ::netabase_deps::derive_more::TryInto,
                    ::netabase_deps::bincode::Encode, ::netabase_deps::bincode::Decode
                )]
                pub enum #definition_key {
                    #(#keys),*
                }
            },
        )
    }
}
