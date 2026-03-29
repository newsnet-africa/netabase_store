use crate::generators::key::{KeyGenerator, KeyPlan};
use crate::visitors::model::{FieldType, ModelVisitor, FieldVisitor};
use proc_macro_flow::{FlowPlan, Generatable};
use quote::{quote, ToTokens};
use syn::{Ident, Path, Type, Item};

/// Plan for generating model implementations.
#[derive(Debug, FlowPlan)]
#[plan(visitor = ModelVisitor<'_>)]
pub struct ModelPlan {
    #[plan(from = "v.key.ident.clone()")]
    pub ident: Ident,
    #[plan(from = "v.fields.fields.iter().map(ModelFieldPlan::from).collect()")]
    pub fields: Vec<ModelFieldPlan>,
    #[plan(copy)]
    pub subscriptions: Vec<Path>,
    #[plan(copy)]
    pub primary_key_hasher: Option<Option<Path>>,
    #[plan(copy)]
    pub version: Option<u32>,
}

/// Plan for a single model field.
#[derive(Debug)]
pub struct ModelFieldPlan {
    pub ident: Ident,
    pub ty: Type,
    pub field_type: ModelFieldType,
}

impl From<&FieldVisitor> for ModelFieldPlan {
    fn from(f: &FieldVisitor) -> Self {
        Self {
            ident: f.ident.clone(),
            ty: f.ty.clone(),
            field_type: match &f.field_type {
                FieldType::None => ModelFieldType::Regular,
                FieldType::PrimaryKey(ty, manual) => ModelFieldType::PrimaryKey(ty.clone(), *manual),
                FieldType::SecondaryKey(ty, manual) => ModelFieldType::SecondaryKey(ty.clone(), *manual),
                FieldType::ForeignKey(ty, manual) => ModelFieldType::ForeignKey(ty.clone(), *manual),
                FieldType::Blob(ty) => ModelFieldType::Blob(ty.clone()),
            },
        }
    }
}

/// The type classification of a model field.
#[derive(Debug, Clone)]
pub enum ModelFieldType {
    Regular,
    PrimaryKey(Type, bool),
    SecondaryKey(Type, bool),
    ForeignKey(Type, bool),
    Blob(Type),
}

/// Generator for model trait implementations.
pub struct ModelGenerator {
    pub plan: ModelPlan,
}

impl From<ModelPlan> for ModelGenerator {
    fn from(plan: ModelPlan) -> Self {
        Self { plan }
    }
}

impl Generatable for ModelGenerator {
    type Output = Item;

    fn generate(self) -> Result<Self::Output, syn::Error> {
        let ident = &self.plan.ident;
        let mut generated_items = proc_macro2::TokenStream::new();

        // Find the primary key field
        let primary_key_field = self.plan.fields.iter().find(|f| {
            matches!(f.field_type, ModelFieldType::PrimaryKey(_, _))
        });

        // Find secondary key fields
        let secondary_key_fields: Vec<_> = self.plan.fields.iter().filter(|f| {
            matches!(f.field_type, ModelFieldType::SecondaryKey(_, _))
        }).collect();

        // Find foreign key fields  
        let foreign_key_fields: Vec<_> = self.plan.fields.iter().filter(|f| {
            matches!(f.field_type, ModelFieldType::ForeignKey(_, _))
        }).collect();

        // Generate primary key type and impl
        let primary_key_type: Type = if let Some(pk) = primary_key_field {
            let (pk_ty, is_manual) = match &pk.field_type {
                ModelFieldType::PrimaryKey(ty, manual) => (ty, *manual),
                _ => unreachable!(),
            };
            
            if is_manual {
                syn::parse_quote!(#pk_ty)
            } else {
                let pk_ident = quote::format_ident!("{}PrimaryKey", ident);
                
                // Generate the key struct
                generated_items.extend(quote! {
                    #[repr(transparent)]
                    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
                    pub struct #pk_ident(pub #pk_ty);
                });

                // Generate NetabaseKeyItem impl using KeyGenerator
                let key_plan = KeyPlan {
                    ident: pk_ident.clone(),
                    generics: syn::Generics::default(),
                };
                let key_gen = KeyGenerator::from(key_plan);
                let key_impl = key_gen.generate()?;
                generated_items.extend(key_impl.to_token_stream());

                syn::parse_quote!(#pk_ident)
            }
        } else {
            syn::parse_quote!(())
        };

        // Generate secondary key type and impl
        let secondary_key_type: Type = if let Some(first) = secondary_key_fields.first() {
            let (sk_ty, is_manual) = match &first.field_type {
                ModelFieldType::SecondaryKey(ty, manual) => (ty, *manual),
                _ => unreachable!(),
            };

            if is_manual {
                syn::parse_quote!(#sk_ty)
            } else {
                let sk_ident = quote::format_ident!("{}SecondaryKey", ident);

                // Generate the key struct
                generated_items.extend(quote! {
                    #[repr(transparent)]
                    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
                    pub struct #sk_ident(pub #sk_ty);
                });

                // Generate NetabaseKeyItem impl
                let key_plan = KeyPlan {
                    ident: sk_ident.clone(),
                    generics: syn::Generics::default(),
                };
                let key_gen = KeyGenerator::from(key_plan);
                let key_impl = key_gen.generate()?;
                generated_items.extend(key_impl.to_token_stream());

                syn::parse_quote!(#sk_ident)
            }
        } else {
            syn::parse_quote!(())
        };

        // Generate foreign key type and impl
        let foreign_key_type: Type = if let Some(first) = foreign_key_fields.first() {
            let (fk_ty, is_manual) = match &first.field_type {
                ModelFieldType::ForeignKey(ty, manual) => (ty, *manual),
                _ => unreachable!(),
            };

            if is_manual {
                syn::parse_quote!(#fk_ty)
            } else {
                let fk_ident = quote::format_ident!("{}ForeignKey", ident);

                // Generate the key struct
                generated_items.extend(quote! {
                    #[repr(transparent)]
                    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
                    pub struct #fk_ident(pub #fk_ty);
                });

                // Generate NetabaseKeyItem impl
                let key_plan = KeyPlan {
                    ident: fk_ident.clone(),
                    generics: syn::Generics::default(),
                };
                let key_gen = KeyGenerator::from(key_plan);
                let key_impl = key_gen.generate()?;
                generated_items.extend(key_impl.to_token_stream());

                syn::parse_quote!(#fk_ident)
            }
        } else {
            syn::parse_quote!(())
        };

        // Generate primary key accessor
        let primary_key_impl = if let Some(pk) = primary_key_field {
            let pk_ident = &pk.ident;
            quote! {
                fn primary_key(&self) -> &Self::PrimaryKey {
                    &self.#pk_ident
                }
            }
        } else {
            quote! {
                fn primary_key(&self) -> &Self::PrimaryKey {
                    unimplemented!("No primary key defined for this model")
                }
            }
        };

        // Generate secondary keys accessor
        let secondary_keys_impl = if !secondary_key_fields.is_empty() {
            let sk_idents: Vec<_> = secondary_key_fields.iter().map(|f| &f.ident).collect();
            quote! {
                fn secondary_keys(&self) -> Option<Vec<&Self::SecondaryKey>> {
                    Some(vec![#( &self.#sk_idents ),*])
                }
            }
        } else {
            quote! {
                fn secondary_keys(&self) -> Option<Vec<&Self::SecondaryKey>> {
                    None
                }
            }
        };

        // Generate foreign keys accessor
        let foreign_keys_impl = if !foreign_key_fields.is_empty() {
            let fk_idents: Vec<_> = foreign_key_fields.iter().map(|f| &f.ident).collect();
            quote! {
                fn foreign_keys(&self) -> Option<Vec<&Self::ForeignKey>> {
                    Some(vec![#( &self.#fk_idents ),*])
                }
            }
        } else {
            quote! {
                fn foreign_keys(&self) -> Option<Vec<&Self::ForeignKey>> {
                    None
                }
            }
        };

        // Generate subscriptions
        let subscriptions = &self.plan.subscriptions;
        let subscriptions_impl = if !subscriptions.is_empty() {
            quote! {
                fn subscriptions() -> &'static [&'static str] {
                    &[#( stringify!(#subscriptions) ),*]
                }
            }
        } else {
            quote! {
                fn subscriptions() -> &'static [&'static str] {
                    &[]
                }
            }
        };

        // Generate version constant
        let version = self.plan.version.unwrap_or(1);
        let version_impl = quote! {
            const VERSION: u32 = #version;
        };

        generated_items.extend(quote! {
            impl NetabaseModelItem for #ident {
                type PrimaryKey = #primary_key_type;
                type SecondaryKey = #secondary_key_type;
                type ForeignKey = #foreign_key_type;

                #version_impl

                #primary_key_impl
                #secondary_keys_impl
                #foreign_keys_impl
                #subscriptions_impl
            }
        });

        Ok(syn::Item::Verbatim(generated_items))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro_flow::Visited;
    use quote::ToTokens;

    #[test]
    fn test_model_plan_creation() {
        let input: syn::DeriveInput = syn::parse_quote! {
            #[version(1)]
            struct User {
                #[primary_key]
                id: u64,
                #[secondary_key]
                email: String,
                name: String,
            }
        };

        let visited = Visited::<ModelVisitor>::from(&input);
        let plan = ModelPlan::try_from(&visited).unwrap();

        assert_eq!(plan.ident.to_string(), "User");
        assert_eq!(plan.fields.len(), 3);
        assert!(matches!(plan.fields[0].field_type, ModelFieldType::PrimaryKey(_, false)));
        assert!(matches!(plan.fields[1].field_type, ModelFieldType::SecondaryKey(_, false)));
        assert!(matches!(plan.fields[2].field_type, ModelFieldType::Regular));
    }

    #[test]
    fn test_model_generator_with_blob() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct ImageModel {
                #[primary_key]
                id: u64,
                #[blob]
                content: Vec<u8>,
            }
        };

        let visited = Visited::<ModelVisitor>::from(&input);
        let plan = ModelPlan::try_from(&visited).unwrap();
        let generator = ModelGenerator::from(plan);

        let output = ::proc_macro_flow::Generatable::generate(generator).unwrap();
        let code = output.to_token_stream().to_string();

        assert!(code.contains("impl NetabaseModelItem for ImageModel"));
        // Check if anything blob related is generated (it currently isn't, so this is to confirm current state)
    }

    #[test]
    fn test_model_generator() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct SimpleModel {
                #[primary_key]
                id: u64,
            }
        };

        let visited = Visited::<ModelVisitor>::from(&input);
        let plan = ModelPlan::try_from(&visited).unwrap();
        let generator = ModelGenerator::from(plan);

        let output = ::proc_macro_flow::Generatable::generate(generator).unwrap();
        let code = output.to_token_stream().to_string();

        assert!(code.contains("impl NetabaseModelItem for SimpleModel"));
        assert!(code.contains("type PrimaryKey = SimpleModelPrimaryKey"));
        assert!(code.contains("struct SimpleModelPrimaryKey"));
        assert!(code.contains("impl NetabaseKeyItem for SimpleModelPrimaryKey"));
    }
}
