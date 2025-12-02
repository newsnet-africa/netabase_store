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
    pub errors: Vec<syn::Error>,
    // Generics support removed - not yet implemented
    // pub generics: Option<&'ast Generics>,
}

impl<'a> Visit<'a> for ModelVisitor<'a> {
    fn visit_derive_input(&mut self, i: &'a syn::DeriveInput) {
        self.name = Some(&i.ident);
        // Generics support removed - not yet implemented
        // self.generics = Some(&i.generics);
        // Extract fields with error handling
        let fields = match extract_fields(i) {
            Ok(fields) => fields,
            Err(e) => {
                let error = syn::Error::new_spanned(
                    &i.ident,
                    format!(
                        "NetabaseModel Derive Error in struct `{}`:\n\n{}",
                        i.ident, e
                    ),
                );
                self.errors.push(error);
                return; // Early return on field extraction error
            }
        };

        self.key = match ModelKeyInfo::find_keys(fields) {
            Ok(k) => Some(k),
            Err(e) => {
                let error = syn::Error::new_spanned(
                    &i.ident,
                    format!(
                        "NetabaseModel Derive Error in struct `{}`:\n\n{}",
                        i.ident, e
                    ),
                );
                self.errors.push(error);
                None
            }
        };
        self.definitions = Self::find_definitions(i);
        self.links = ModelLinkInfo::find_link(fields).collect();
    }
}

impl<'a> ModelVisitor<'a> {
    pub fn find_definitions(input: &'a syn::DeriveInput) -> Vec<syn::Path> {
        // Use extremely defensive programming to avoid any potential panics during attribute parsing
        // Wrap everything in a catch-all to ensure we never panic during attribute processing
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            match input.attrs.iter().find(|a| {
                // Even the path checking could potentially panic in edge cases
                match a.path().is_ident("netabase") {
                    true => true,
                    false => false,
                }
            }) {
                Some(attr) => {
                    match attr.meta.require_list() {
                        Ok(list) => {
                            match list.parse_args_with(
                                Punctuated::<syn::Path, Token![,]>::parse_terminated,
                            ) {
                                Ok(paths) => paths.into_iter().collect(),
                                Err(_) => {
                                    // Parsing failed - return empty vec instead of panicking
                                    // This could happen if the attribute syntax is invalid or
                                    // if the referenced types don't exist yet (macro ordering)
                                    vec![]
                                }
                            }
                        }
                        Err(_) => {
                            // Meta is not a list - return empty vec
                            vec![]
                        }
                    }
                }
                None => {
                    // No netabase attribute found
                    vec![]
                }
            }
        }))
        .unwrap_or_else(|_| {
            // If any panic occurred during attribute parsing, return empty vec
            // This ensures the macro doesn't crash but will trigger the missing attribute error later
            vec![]
        })
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn into_compile_errors(self) -> Vec<proc_macro2::TokenStream> {
        self.errors
            .into_iter()
            .map(|e| e.into_compile_error())
            .collect()
    }
}
