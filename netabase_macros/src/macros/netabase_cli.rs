//! CLI generation macro for Store schemas

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Result};

use crate::generators::cli;

/// Generate CLI for a definition or repository
pub fn generate_cli_macro(_input: TokenStream) -> Result<TokenStream> {
    // Parse the input to extract definition/repository information
    // For now, we'll create a simplified version that works with the inferred definitions

    Ok(quote! {
        // CLI generation will be integrated with infer_netabase_definition
        compile_error!("Use #[generate_cli] with infer_netabase_definition!");
    })
}

/// Generate CLI for an inferred definition (used internally)
pub fn generate_cli_for_definition(def_name: &Ident, models: &[String]) -> TokenStream {
    let model_idents: Vec<Ident> = models
        .iter()
        .map(|m| quote::format_ident!("{}", m))
        .collect();

    cli::generate_store_cli(def_name, &[(def_name.clone(), model_idents)])
}
