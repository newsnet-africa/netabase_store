use crate::visitors::constants::model::{self, *};
use proc_macro_flow::{AttributeExt, FlowVisitor, FlowAttribute};
use syn::visit::Visit;
use syn::{FieldMutability, Ident, Path, Type};

pub mod key;

use self::key::KeyVisitor;

#[derive(FlowVisitor)]
pub struct ModelVisitor<'a> {
    #[visit(attr)]
    pub(crate) key: KeyVisitor,
    #[visit(attr)]
    pub(crate) fields: ModelFields,
    #[visit(extract = extract_mutability)]
    pub(crate) mutability: &'a FieldMutability,
    pub(crate) subscriptions: Vec<Path>,
    pub(crate) primary_key_hasher: Option<Option<Path>>, // Some(None) means default hasher, Some(Some(path)) means custom
    pub(crate) version: Option<u32>,
}

#[derive(Debug, Clone, FlowVisitor, FlowAttribute)]
#[visit(traversal)]
pub struct ModelFields {
    #[visit(extract = empty_vec)]
    pub fields: Vec<FieldVisitor>,
}

fn empty_vec(_input: &syn::DeriveInput) -> Vec<FieldVisitor> {
    Vec::new()
}

impl<'ast> syn::visit::Visit<'ast> for ModelFields {
    fn visit_field(&mut self, i: &'ast syn::Field) {
        if let Some(ident) = &i.ident {
            let mut field_visitor = FieldVisitor {
                ident: ident.clone(),
                ty: i.ty.clone(),
                field_type: FieldType::None,
            };
            field_visitor.visit_field(i);
            self.fields.push(field_visitor);
        }
    }
}

#[allow(unused_variables)]
fn extract_mutability(_input: &syn::DeriveInput) -> &FieldMutability {
    // Standard DeriveInput doesn't have mutability at struct level,
    // it's usually inferred or hardcoded to None for visitors
    &FieldMutability::None
}

#[derive(Debug, Clone)]
pub struct FieldVisitor {
    pub ident: Ident,
    pub ty: Type,
    pub field_type: FieldType,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    None,
    PrimaryKey(Type, bool), // bool = manual
    SecondaryKey(Type, bool),
    ForeignKey(Type, bool),
    Blob(Type),
}

impl<'ast> syn::visit::Visit<'ast> for FieldVisitor {
    fn visit_field(&mut self, i: &'ast syn::Field) {
        let field_type = i.attrs.find_attribute(
            &[PRIMARY_KEY, SECONDARY_KEY, FOREIGN_KEY, BLOB],
            |target, attr| {
                let mut is_manual = false;
                if let syn::Meta::List(list) = &attr.meta {
                    let _ = list.parse_nested_meta(|meta| {
                        if meta.path.is_ident("manual") {
                            is_manual = true;
                        }
                        Ok(())
                    });
                }

                let ft = match model::Attribute::from(target) {
                    model::Attribute::PrimaryKey => FieldType::PrimaryKey(i.ty.clone(), is_manual),
                    model::Attribute::SecondaryKey => FieldType::SecondaryKey(i.ty.clone(), is_manual),
                    model::Attribute::ForeignKey => FieldType::ForeignKey(i.ty.clone(), is_manual),
                    model::Attribute::Blob => FieldType::Blob(i.ty.clone()),
                    _ => return Ok(None),
                };
                Ok(Some(ft))
            },
        );

        if let Ok(Some(ft)) = field_type {
            self.field_type = ft;
        }
    }
}

impl<'a> Visit<'a> for ModelVisitor<'a> {
    fn visit_derive_input(&mut self, i: &'a syn::DeriveInput) {
        let _ = i
            .attrs
            .find_attribute(&[SUBSCRIBE, VERSION, PRIMARY_KEY], |target, attr| {
                match model::Attribute::from(target) {
                    model::Attribute::Subscribe => {
                        if let syn::Meta::List(list) = &attr.meta {
                            let path: Path = list.parse_args()?;
                            self.subscriptions.push(path);
                        }
                    }
                    model::Attribute::Version => {
                        if let syn::Meta::List(list) = &attr.meta {
                            let lit: syn::LitInt = list.parse_args()?;
                            self.version = Some(lit.base10_parse()?);
                        }
                    }
                    model::Attribute::PrimaryKey => match &attr.meta {
                        syn::Meta::Path(_) => {
                            self.primary_key_hasher = Some(None);
                        }
                        syn::Meta::List(list) => {
                            let path: Path = list.parse_args()?;
                            self.primary_key_hasher = Some(Some(path));
                        }
                        _ => {}
                    },
                    _ => {}
                }
                Ok(None::<()>)
            });
    }
}
