use syn::{ItemEnum, ItemStruct, parse_quote};

use crate::{util::append_ident, visitors::model_visitor::ModelVisitor};

impl<'a> ModelVisitor<'a> {
    pub fn generate_keys(&self) -> (ItemStruct, Vec<ItemStruct>, ItemEnum, ItemEnum) {
        let p_keys = match Self::generate_primary_key(&self) {
            Ok(k) => k,
            Err(e) => panic!("{}", e),
        };
        let primary_key_id = p_keys.ident.clone();
        let secondary_newtypes = self.generate_secondary_keys_newtypes();
        let secondary_keys = self.generate_secondary_keys(&secondary_newtypes);
        let secondary_newtypes = secondary_newtypes.iter().map(|(s, _)| s.clone()).collect();
        let secondary_key_id = secondary_keys.ident.clone();
        let name = match self.name {
            Some(n) => append_ident(n, "Key"),
            None => panic!("Visitor error (parsing struct name?)"),
        };
        (
            p_keys,
            secondary_newtypes,
            secondary_keys,
            parse_quote!(
                #[derive(Debug, ::netabase_deps::strum::EnumDiscriminants,
                    ::netabase_deps::derive_more::From, ::netabase_deps::derive_more::TryInto,
                    ::netabase_deps::bincode::Encode, ::netabase_deps::bincode::Decode
                )]
                pub enum #name {
                    Primary(#primary_key_id),
                    Secondary(#secondary_key_id),
                }
            ),
        )
    }
}

mod key_gen {
    use bincode::config::Varint;
    use syn::{Field, Ident, ItemEnum, ItemStruct, Variant, parse_quote, token::Async};

    use crate::{
        errors::NetabaseModelDeriveError, util::append_ident, visitors::model_visitor::ModelVisitor,
    };

    impl<'a> ModelVisitor<'a> {
        fn generate_newtype<'ast>(field: &'ast Field, name: &Ident) -> ItemStruct {
            let ty = &field.ty;
            parse_quote!(
                #[derive(Debug, ::netabase_deps::derive_more::From, ::netabase_deps::derive_more::Into,
                    ::netabase_deps::bincode::Encode, ::netabase_deps::bincode::Decode
                )]
                pub struct #name(#ty);
            )
        }

        pub fn generate_primary_key(&self) -> Result<ItemStruct, NetabaseModelDeriveError> {
            let key = match &self.key {
                Some(k) => k,
                None => return Err(NetabaseModelDeriveError::PrimaryKeyNotFound),
            };
            let name = match &self.name {
                Some(n) => n,
                None => return Err(NetabaseModelDeriveError::MacroVisitorError),
            };
            Ok(Self::generate_newtype(
                key.primary_keys,
                &append_ident(name, "PrimaryKey"),
            ))
        }

        pub fn generate_secondary_keys_newtypes(&self) -> Vec<(ItemStruct, Ident)> {
            let key = match &self.key {
                Some(k) => k,
                None => return vec![],
            };

            key.secondary_keys
                .iter()
                .map(|f| {
                    let ident = if let Some(id) = &f.ident {
                        let id = heck::AsUpperCamelCase(id.to_string());
                        Ident::new(&id.to_string(), proc_macro2::Span::call_site())
                    } else {
                        panic!("Struct fields must be named")
                    };
                    (
                        Self::generate_newtype(f, &append_ident(&ident, "SecondaryKey")),
                        ident,
                    )
                })
                .collect()
        }

        pub fn generate_variant(keys: &(ItemStruct, Ident)) -> Variant {
            let name = &keys.1;
            let ty = &keys.0.ident;
            parse_quote! {
                #name(#ty)
            }
        }

        pub fn generate_secondary_keys(&self, keys: &Vec<(ItemStruct, Ident)>) -> ItemEnum {
            let list = keys.iter().map(|f| Self::generate_variant(f));
            let name = match &self.name {
                Some(n) => &append_ident(n, "SecondaryKeys"),
                None => panic!("Visitor not initialised"),
            };
            parse_quote!(
                #[derive(Debug, ::netabase_deps::strum::EnumDiscriminants,
                    ::netabase_deps::derive_more::From, ::netabase_deps::derive_more::TryInto,
                    ::netabase_deps::bincode::Encode, ::netabase_deps::bincode::Decode
                )]
                pub enum #name {
                    #(#list),*
                }
            )
        }
    }
}
