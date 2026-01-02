use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr, parse::{Parse, ParseStream}, Token, Ident};
use std::fs;
use std::path::PathBuf;
use serde::Deserialize;

struct ImportInput {
    file_path: LitStr,
    module_name: Option<Ident>,
}

impl Parse for ImportInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let file_path: LitStr = input.parse()?;
        let module_name = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(input.parse::<Ident>()?)
        } else {
            None
        };
        Ok(ImportInput { file_path, module_name })
    }
}

#[proc_macro]
pub fn infer_netabase_definition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ImportInput);
    let file_path_lit = input.file_path;
    let file_path_str = file_path_lit.value();
    
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let full_path = PathBuf::from(manifest_dir).join(&file_path_str);
    
    let content = match fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(e) => return syn::Error::new_spanned(
            file_path_lit, 
            format!("Failed to read file at {:?}: {}", full_path, e)
        ).to_compile_error().into(),
    };
    
    #[derive(Deserialize)]
    struct FullSchema {
        name: String,
        subscriptions: Vec<String>,
        #[serde(flatten)]
        _other: toml::Table,
    }

    let schema: FullSchema = match toml::from_str(&content) {
        Ok(s) => s,
        Err(e) => return syn::Error::new_spanned(
            file_path_lit,
            format!("Failed to parse TOML: {}", e)
        ).to_compile_error().into(),
    };
    
    let def_name = syn::Ident::new(&schema.name, proc_macro2::Span::call_site());
    let subs: Vec<syn::Ident> = schema.subscriptions.iter()
        .map(|s| syn::Ident::new(s, proc_macro2::Span::call_site()))
        .collect();
        
    let module_name = input.module_name.unwrap_or_else(|| def_name.clone());
        
    let output = quote! {
        #[netabase_macros::netabase_definition(
            #def_name,
            subscriptions(#(#subs),*),
            from_file = #file_path_str
        )]
        pub mod #module_name {
            use super::*;
        }
    };
    
    output.into()
}