use quote::ToTokens;
use syn::{ItemStruct, parse_quote};

use crate::{
    util::append_ident,
    visitors::uniffi_visitor::{self, UniffiVisitor},
};
impl<'a> UniffiVisitor<'a> {
    pub fn generate_uniffi_type(&self) -> ItemStruct {
        let name = append_ident(
            self.name.expect("Visitor has not found a name"),
            "UniffiType",
        );

        let fields = self.fields.expect("Visitor has not found a name");
        let stripped_fields = strip_field_attrs(fields);

        let fields_ts = match stripped_fields {
            syn::Fields::Named(fields_named) => fields_named.to_token_stream(),
            syn::Fields::Unnamed(fields_unnamed) => fields_unnamed.to_token_stream(),
            syn::Fields::Unit => unreachable!("Model should have fields"),
        };

        parse_quote! {
            #[derive(::netabase_store::netabase_deps::uniffi::Record)]
            pub struct #name
                #fields_ts
        }
    }
}

fn strip_field_attrs(fields: &syn::Fields) -> syn::Fields {
    match fields {
        syn::Fields::Named(fields_named) => {
            let new_fields = fields_named.named.iter().map(|f| {
                let mut f2 = f.clone();
                f2.attrs.clear();
                f2
            });
            syn::Fields::Named(syn::FieldsNamed {
                brace_token: fields_named.brace_token,
                named: new_fields.collect(),
            })
        }
        syn::Fields::Unnamed(fields_unnamed) => {
            let new_fields = fields_unnamed.unnamed.iter().map(|f| {
                let mut f2 = f.clone();
                f2.attrs.clear();
                f2
            });
            syn::Fields::Unnamed(syn::FieldsUnnamed {
                paren_token: fields_unnamed.paren_token,
                unnamed: new_fields.collect(),
            })
        }
        syn::Fields::Unit => syn::Fields::Unit,
    }
}
