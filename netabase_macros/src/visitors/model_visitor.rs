use syn::{Ident, Path, Token, punctuated::Punctuated, visit::Visit};

use crate::{
    item_info::netabase_model::{ModelKeyInfo, ModelLinkInfo},
    util::extract_fields,
};

#[derive(Default)]
pub struct ModelVisitor<'ast> {
    pub name: Option<&'ast Ident>,
    pub key: Option<ModelKeyInfo<'ast>>,
    pub links: Vec<ModelLinkInfo<'ast>>,
    pub definitions: Vec<Path>,
}

impl<'a> Visit<'a> for ModelVisitor<'a> {
    fn visit_derive_input(&mut self, i: &'a syn::DeriveInput) {
        self.name = Some(&i.ident);
        self.key = match ModelKeyInfo::find_keys(extract_fields(i)) {
            Ok(k) => Some(k),
            Err(e) => panic!("Error parsing Model: {e}"),
        };
        self.definitions = Self::find_definitions(i);
        self.links = ModelLinkInfo::find_link(extract_fields(i)).collect();
    }
}

impl<'a> ModelVisitor<'a> {
    pub fn find_definitions(input: &'a syn::DeriveInput) -> Vec<syn::Path> {
        let attr = input.attrs.iter().find(|a| a.path().is_ident("netabase"));
        if let Some(att) = attr
            && let Ok(list) = att.meta.require_list()
        {
            match list
                .parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)
                .map_err(|e| e.into_compile_error())
            {
                Ok(r) => r.into_iter().collect(),
                Err(_) => vec![],
            }
        } else {
            vec![]
        }
    }
}
