//! Implementation of the `#[netabase_repository]` attribute macro.
//!
//! This macro is applied to a module containing definitions and generates:
//! - Repository marker struct
//! - Definition discriminant enum
//! - Model discriminant enum
//! - `NetabaseRepository` trait implementation
//! - `InRepository` trait implementations for each definition
//! - Schema export and hash methods

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemMod, Result, parse2};

use crate::generators::repository::RepositoryGenerator;
use crate::utils::attributes::{parse_repository_attribute_from_tokens, remove_attribute};
use crate::visitors::repository::RepositoryVisitor;

/// Implementation of the netabase_repository attribute macro
pub fn netabase_repository_attribute(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    // Parse attribute to get repository name
    let config = parse_repository_attribute_from_tokens(attr)?;
    let repo_name = config.name;

    // Parse the module
    let mut module: ItemMod = parse2(item)?;

    // Ensure the module has content
    if module.content.is_none() {
        return Err(syn::Error::new_spanned(
            &module,
            "netabase_repository can only be applied to modules with content (not external modules)",
        ));
    }

    // Create repository visitor and collect information
    let mut visitor = RepositoryVisitor::new(repo_name);
    visitor.visit_module(&module)?;

    // Check for missing definitions (incomplete data graph)
    if let Some(error) = visitor.generate_missing_error() {
        return Err(error);
    }

    // Generate cycle warnings (these are warnings, not errors)
    let cycle_warnings = visitor.generate_cycle_warnings();
    for warning in &cycle_warnings {
        // In proc macros we can't emit warnings directly, so we'll add them as doc comments
        // The user will see these in the generated code documentation
        eprintln!("{}", warning);
    }

    // Generate repository code
    let generator = RepositoryGenerator::new(&visitor);
    let generated_code = generator.generate();

    // Remove the netabase_repository attribute from the module
    remove_attribute(&mut module.attrs, "netabase_repository");

    // Append generated code to the module
    if let Some((ref _brace, ref mut items)) = module.content {
        let file: syn::File = parse2(generated_code).map_err(|e| {
            syn::Error::new(e.span(), format!("Failed to parse repository code: {}", e))
        })?;

        items.extend(file.items.into_iter().map(syn::Item::from));
    }

    Ok(quote! {
        #module
    })
}
