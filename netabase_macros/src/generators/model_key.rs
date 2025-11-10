use syn::{Ident, ItemEnum, ItemStruct, parse_quote};

use crate::{util::append_ident, visitors::model_visitor::ModelVisitor};

impl<'a> ModelVisitor<'a> {
    pub fn generate_keys(&self) -> (ItemStruct, Vec<ItemStruct>, ItemEnum, ItemEnum) {
        let p_keys = match Self::generate_primary_key(self) {
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
        let keys_enum: ItemEnum = parse_quote!(
            #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
            ::netabase_store::derive_more::From, ::netabase_store::derive_more::TryInto,
                ::netabase_store::bincode::Encode, ::netabase_store::bincode::Decode
            )]
            pub enum #name {
                Primary(#primary_key_id),
                Secondary(#secondary_key_id),
            }
        );

        (p_keys, secondary_newtypes, secondary_keys, keys_enum)
    }

    pub fn generate_borrow_impls(&self) -> Vec<proc_macro2::TokenStream> {
        let model_name = match self.name {
            Some(n) => n,
            None => panic!("Visitor error (parsing struct name?)"),
        };

        let mut impls = Vec::new();

        // Get primary key info
        let key = match &self.key {
            Some(k) => k,
            None => return impls,
        };

        let primary_key_ty = append_ident(model_name, "PrimaryKey");
        let primary_inner_ty = &key.primary_keys.ty;

        // 1. Primary key newtype implements Borrow<InnerType>
        impls.push(quote::quote! {
            impl ::std::borrow::Borrow<#primary_inner_ty> for #primary_key_ty {
                fn borrow(&self) -> &#primary_inner_ty {
                    &self.0
                }
            }
        });

        // 2. Each secondary key newtype implements Borrow<InnerType>
        let secondary_newtypes = self.generate_secondary_keys_newtypes();
        for (newtype_struct, _variant_name) in &secondary_newtypes {
            let newtype_ty = &newtype_struct.ident;
            // Extract inner type from the newtype struct
            if let syn::Fields::Unnamed(fields) = &newtype_struct.fields {
                if let Some(field) = fields.unnamed.first() {
                    let inner_ty = &field.ty;
                    impls.push(quote::quote! {
                        impl ::std::borrow::Borrow<#inner_ty> for #newtype_ty {
                            fn borrow(&self) -> &#inner_ty {
                                &self.0
                            }
                        }
                    });
                }
            }
        }

        // 3. SecondaryKeys enum implements Borrow<VariantType> for each variant
        let secondary_keys_ty = append_ident(model_name, "SecondaryKeys");
        for (newtype_struct, variant_name) in &secondary_newtypes {
            let newtype_ty = &newtype_struct.ident;
            impls.push(quote::quote! {
                impl ::std::borrow::Borrow<#newtype_ty> for #secondary_keys_ty {
                    fn borrow(&self) -> &#newtype_ty {
                        match self {
                            Self::#variant_name(inner) => inner,
                            _ => panic!(
                                "Attempted to borrow {} from wrong SecondaryKeys variant. Use pattern matching for safe access.",
                                stringify!(#newtype_ty)
                            ),
                        }
                    }
                }
            });
        }

        // 4. Main Keys enum implements Borrow<PrimaryKey> and Borrow<SecondaryKeys>
        let keys_ty = append_ident(model_name, "Key");
        impls.push(quote::quote! {
            impl ::std::borrow::Borrow<#primary_key_ty> for #keys_ty {
                fn borrow(&self) -> &#primary_key_ty {
                    match self {
                        Self::Primary(key) => key,
                        _ => panic!(
                            "Attempted to borrow PrimaryKey from Secondary variant. Use pattern matching for safe access."
                        ),
                    }
                }
            }
        });

        impls.push(quote::quote! {
            impl ::std::borrow::Borrow<#secondary_keys_ty> for #keys_ty {
                fn borrow(&self) -> &#secondary_keys_ty {
                    match self {
                        Self::Secondary(keys) => keys,
                        _ => panic!(
                            "Attempted to borrow SecondaryKeys from Primary variant. Use pattern matching for safe access."
                        ),
                    }
                }
            }
        });

        impls
    }

    pub fn generate_model_trait_impl(&self) -> Vec<proc_macro2::TokenStream> {
        let model_name = match self.name {
            Some(n) => n,
            None => panic!("Visitor error (parsing struct name?)"),
        };
        let primary_key_ty = match Self::generate_primary_key(self) {
            Ok(k) => k.ident,
            Err(e) => panic!("{}", e),
        };
        let secondary_keys_ty = append_ident(model_name, "SecondaryKeys");
        let keys_ty = append_ident(model_name, "Key");

        // Get the primary key field identifier
        let primary_field = match &self.key {
            Some(k) => k.primary_keys.ident.as_ref().unwrap(),
            None => panic!("Primary key not found"),
        };

        // Get secondary key field identifiers
        let secondary_fields: Vec<_> = match &self.key {
            Some(k) => k
                .secondary_keys
                .iter()
                .map(|f| {
                    let field_name = f.ident.as_ref().unwrap();
                    let field_name_upper = heck::AsUpperCamelCase(field_name.to_string());
                    let variant = Ident::new(
                        &field_name_upper.to_string(),
                        proc_macro2::Span::call_site(),
                    );
                    quote::quote! {
                        #secondary_keys_ty::#variant(self.#field_name.clone().into())
                    }
                })
                .collect(),
            None => vec![],
        };

        // Use discriminant name for both trait implementation and redb TypeName
        // This ensures consistency when models are used across different definition enums
        let discriminant_name = model_name.to_string();
        let primary_key_name_str = format!("{}::PrimaryKey", discriminant_name);
        let secondary_keys_name_str = format!("{}::SecondaryKeys", discriminant_name);

        // Generics support removed - not yet implemented
        // Extract generics information
        // let (impl_generics, ty_generics, where_clause) = match self.generics {
        //     Some(g) => {
        //         let (ig, tg, wc) = g.split_for_impl();
        //         (quote::quote! { #ig }, quote::quote! { #tg }, quote::quote! { #wc })
        //     }
        //     None => (quote::quote! {}, quote::quote! {}, quote::quote! {}),
        // };

        self.definitions.iter().map(|def_path| {
            quote::quote! {
                impl ::netabase_store::traits::model::NetabaseModelTrait<#def_path> for #model_name {
                    type PrimaryKey = #primary_key_ty;
                    type SecondaryKeys = #secondary_keys_ty;
                    type Keys = #keys_ty;

                    const DISCRIMINANT:<#def_path as ::netabase_store::strum::IntoDiscriminant>::Discriminant
                        = <#def_path as ::netabase_store::strum::IntoDiscriminant>::Discriminant::#model_name;

                    fn primary_key(&self) -> Self::PrimaryKey {
                        #primary_key_ty(self.#primary_field.clone())
                    }

                    fn secondary_keys(&self) -> Vec<Self::SecondaryKeys> {
                        vec![#(#secondary_fields),*]
                    }

                    fn discriminant_name() -> &'static str {
                        #discriminant_name
                    }
                }

                impl ::netabase_store::traits::model::NetabaseModelTraitKey<#def_path> for #primary_key_ty {

                    const DISCRIMINANT:<<#def_path as ::netabase_store::traits::definition::NetabaseDefinitionTrait>::Keys as ::netabase_store::strum::IntoDiscriminant>::Discriminant
                        = <<#def_path as ::netabase_store::traits::definition::NetabaseDefinitionTrait>::Keys as ::netabase_store::strum::IntoDiscriminant>::Discriminant::#keys_ty;
                }
                impl ::netabase_store::traits::model::NetabaseModelTraitKey<#def_path> for #secondary_keys_ty {

                    const DISCRIMINANT:<<#def_path as ::netabase_store::traits::definition::NetabaseDefinitionTrait>::Keys as ::netabase_store::strum::IntoDiscriminant>::Discriminant
                        = <<#def_path as ::netabase_store::traits::definition::NetabaseDefinitionTrait>::Keys as ::netabase_store::strum::IntoDiscriminant>::Discriminant::#keys_ty;
                }
                impl ::netabase_store::traits::model::NetabaseModelTraitKey<#def_path> for #keys_ty {

                    const DISCRIMINANT:<<#def_path as ::netabase_store::traits::definition::NetabaseDefinitionTrait>::Keys as ::netabase_store::strum::IntoDiscriminant>::Discriminant
                        = <<#def_path as ::netabase_store::traits::definition::NetabaseDefinitionTrait>::Keys as ::netabase_store::strum::IntoDiscriminant>::Discriminant::#keys_ty;
                }

                // redb trait implementations (only when redb feature is enabled)
                // For redb, we use owned types for SelfType since bincode requires ownership
                // and we implement Borrow<Self> which is automatic for all types
                #[cfg(feature = "redb")]
                impl ::netabase_store::netabase_deps::redb::Value for #model_name {
                    type SelfType<'a> = #model_name where Self: 'a;
                    type AsBytes<'a> = Vec<u8> where Self: 'a;

                    fn fixed_width() -> Option<usize> {
                        None
                    }

                    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
                    where
                        Self: 'a,
                    {
                        ::netabase_store::bincode::decode_from_slice(data, ::netabase_store::bincode::config::standard())
                            .unwrap()
                            .0
                    }

                    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                    where
                        Self: 'a,
                        Self: 'b,
                    {
                        ::netabase_store::bincode::encode_to_vec(value, ::netabase_store::bincode::config::standard()).unwrap()
                    }

                    fn type_name() -> ::netabase_store::netabase_deps::redb::TypeName {
                        ::netabase_store::netabase_deps::redb::TypeName::new(#discriminant_name)
                    }
                }

                // Implement FromRedbValue trait for safe conversion
                #[cfg(feature = "redb")]
                impl ::netabase_store::databases::redb_store::FromRedbValue for #model_name {
                    #[inline]
                    fn from_redb_value(value: &<Self as ::netabase_store::netabase_deps::redb::Value>::SelfType<'_>) -> Self {
                        value.clone()
                    }
                }

                #[cfg(feature = "redb")]
                impl ::netabase_store::netabase_deps::redb::Value for #primary_key_ty {
                    type SelfType<'a> = #primary_key_ty where Self: 'a;
                    type AsBytes<'a> = Vec<u8> where Self: 'a;

                    fn fixed_width() -> Option<usize> {
                        None
                    }

                    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
                    where
                        Self: 'a,
                    {
                        ::netabase_store::bincode::decode_from_slice(data, ::netabase_store::bincode::config::standard())
                            .unwrap()
                            .0
                    }

                    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                    where
                        Self: 'a,
                        Self: 'b,
                    {
                        ::netabase_store::bincode::encode_to_vec(value, ::netabase_store::bincode::config::standard()).unwrap()
                    }

                    fn type_name() -> ::netabase_store::netabase_deps::redb::TypeName {
                        ::netabase_store::netabase_deps::redb::TypeName::new(#primary_key_name_str)
                    }
                }

                #[cfg(feature = "redb")]
                impl ::netabase_store::netabase_deps::redb::Key for #primary_key_ty {
                    fn compare(data1: &[u8], data2: &[u8]) -> ::std::cmp::Ordering {
                        Self::from_bytes(data1).cmp(&Self::from_bytes(data2))
                    }
                }

                // Implement FromRedbValue trait for PrimaryKey
                #[cfg(feature = "redb")]
                impl ::netabase_store::databases::redb_store::FromRedbValue for #primary_key_ty {
                    #[inline]
                    fn from_redb_value(value: &<Self as ::netabase_store::netabase_deps::redb::Value>::SelfType<'_>) -> Self {
                        value.clone()
                    }
                }


                #[cfg(feature = "redb")]
                impl ::netabase_store::netabase_deps::redb::Value for #secondary_keys_ty {
                    type SelfType<'a> = #secondary_keys_ty where Self: 'a;
                    type AsBytes<'a> = Vec<u8> where Self: 'a;

                    fn fixed_width() -> Option<usize> {
                        None
                    }

                    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
                    where
                        Self: 'a,
                    {
                        ::netabase_store::bincode::decode_from_slice(data, ::netabase_store::bincode::config::standard())
                            .unwrap()
                            .0
                    }

                    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                    where
                        Self: 'a,
                        Self: 'b,
                    {
                        ::netabase_store::bincode::encode_to_vec(value, ::netabase_store::bincode::config::standard()).unwrap()
                    }

                    fn type_name() -> ::netabase_store::netabase_deps::redb::TypeName {
                        ::netabase_store::netabase_deps::redb::TypeName::new(#secondary_keys_name_str)
                    }
                }

                #[cfg(feature = "redb")]
                impl ::netabase_store::netabase_deps::redb::Key for #secondary_keys_ty {
                    fn compare(data1: &[u8], data2: &[u8]) -> ::std::cmp::Ordering {
                        Self::from_bytes(data1).cmp(&Self::from_bytes(data2))
                    }
                }

                // Implement FromRedbValue trait for SecondaryKeys
                #[cfg(feature = "redb")]
                impl ::netabase_store::databases::redb_store::FromRedbValue for #secondary_keys_ty {
                    #[inline]
                    fn from_redb_value(value: &<Self as ::netabase_store::netabase_deps::redb::Value>::SelfType<'_>) -> Self {
                        value.clone()
                    }
                }

            }
        }).collect::<Vec<proc_macro2::TokenStream>>()
    }
}

mod key_gen {
    use syn::{Field, Ident, ItemEnum, ItemStruct, Variant, parse_quote};

    use crate::{
        errors::NetabaseModelDeriveError, util::append_ident, visitors::model_visitor::ModelVisitor,
    };

    impl<'a> ModelVisitor<'a> {
        fn generate_newtype(field: &Field, name: &Ident) -> ItemStruct {
            let ty = &field.ty;
            parse_quote!(
                #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ::netabase_store::derive_more::From, ::netabase_store::derive_more::Into,
                    ::netabase_store::bincode::Encode, ::netabase_store::bincode::Decode
                )]
                pub struct #name(pub #ty);
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

            let model_name = match self.name {
                Some(n) => n,
                None => panic!("Model name not found"),
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
                    // Prefix secondary key type with model name to avoid conflicts
                    let type_name = format!("{}{}", model_name, append_ident(&ident, "SecondaryKey"));
                    let type_ident = Ident::new(&type_name, proc_macro2::Span::call_site());
                    (
                        Self::generate_newtype(f, &type_ident),
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
            let list = keys.iter().map(Self::generate_variant);
            let name = match &self.name {
                Some(n) => &append_ident(n, "SecondaryKeys"),
                None => panic!("Visitor not initialised"),
            };
            parse_quote!(
                #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ::netabase_store::strum::EnumDiscriminants,
                    ::netabase_store::strum::Display,
                    ::netabase_store::derive_more::From, ::netabase_store::derive_more::TryInto,
                    ::netabase_store::bincode::Encode, ::netabase_store::bincode::Decode
                )]
                #[strum_discriminants(derive(::netabase_store::strum::Display,
                ::netabase_store::strum::AsRefStr ))]
                pub enum #name {
                    #(#list),*
                }
            )
        }
    }
}
