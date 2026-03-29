use proc_macro_flow::FlowVisitor;
use syn::{Attribute, Generics, Ident, Type};

#[derive(Debug, Clone, FlowVisitor)]
pub struct BlobVisitor {
    #[visit(ident)]
    pub(crate) ident: Ident,
    #[visit(generics)]
    pub(crate) generics: Generics,
    #[visit(attrs)]
    pub(crate) attrs: Vec<Attribute>,
    #[visit(attr)]
    pub(crate) kind: BlobItemKind,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum BlobItemKind {
    #[default]
    Unknown,
    Struct {
        fields: Vec<BlobField>,
    },
    Enum {
        variants: Vec<BlobVariant>,
    },
}

impl From<&syn::DeriveInput> for BlobItemKind {
    fn from(input: &syn::DeriveInput) -> Self {
        let mut kind_visitor = KindVisitor {
            kind: BlobItemKind::Unknown,
        };
        syn::visit::Visit::visit_derive_input(&mut kind_visitor, input);
        kind_visitor.kind
    }
}

impl proc_macro_flow::FlowAttribute<syn::DeriveInput> for BlobItemKind {}

struct KindVisitor {
    kind: BlobItemKind,
}

impl<'ast> syn::visit::Visit<'ast> for KindVisitor {
    fn visit_derive_input(&mut self, i: &'ast syn::DeriveInput) {
        syn::visit::visit_data(self, &i.data);
    }

    fn visit_data(&mut self, i: &'ast syn::Data) {
        match i {
            syn::Data::Struct(data_struct) => syn::visit::visit_data_struct(self, data_struct),
            syn::Data::Enum(data_enum) => syn::visit::visit_data_enum(self, data_enum),
            syn::Data::Union(_data_union) => todo!(),
        }
    }

    fn visit_data_struct(&mut self, i: &'ast syn::DataStruct) {
        if let BlobItemKind::Unknown = self.kind {
            self.kind = BlobItemKind::Struct { fields: Vec::new() };
        }
        syn::visit::visit_fields(self, &i.fields);
    }

    fn visit_data_enum(&mut self, i: &'ast syn::DataEnum) {
        if let BlobItemKind::Unknown = self.kind {
            self.kind = BlobItemKind::Enum {
                variants: Vec::new(),
            };
        }
        for variant in &i.variants {
            self.visit_variant(variant);
        }
    }

    fn visit_variant(&mut self, i: &'ast syn::Variant) {
        if let BlobItemKind::Enum { variants, .. } = &mut self.kind {
            variants.push(BlobVariant {
                ident: i.ident.clone(),
                attrs: i.attrs.clone(),
                fields: Vec::new(),
            });
        }
        syn::visit::visit_fields(self, &i.fields);
    }

    fn visit_field(&mut self, i: &'ast syn::Field) {
        let bf = BlobField {
            ident: i.ident.clone(),
            ty: i.ty.clone(),
            attrs: i.attrs.clone(),
        };

        match &mut self.kind {
            BlobItemKind::Struct { fields, .. } => {
                fields.push(bf);
            }
            BlobItemKind::Enum { variants, .. } => {
                if let Some(variant) = variants.last_mut() {
                    variant.fields.push(bf);
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlobVariant {
    pub ident: Ident,
    pub attrs: Vec<Attribute>,
    pub fields: Vec<BlobField>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlobField {
    pub ident: Option<Ident>,
    pub ty: Type,
    pub attrs: Vec<Attribute>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{DeriveInput, parse_quote};

    #[test]
    fn test_blob_visitation() {
        let input: DeriveInput = parse_quote! {
            #[derive(NetabaseBlob)]
            #[chunk_size(1024)]
            struct MyBlob {
                #[chunk_size(512)]
                field1: String,
            }
        };

        let visitor = BlobVisitor::from(&input);

        assert_eq!(visitor.ident, "MyBlob");
        if let BlobItemKind::Struct { fields } = &visitor.kind {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].ident.as_ref().unwrap(), "field1");
            assert_eq!(fields[0].attrs.len(), 1);
        } else {
            panic!("Expected Struct");
        }
    }
}
