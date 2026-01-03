//! Global macro for grouping multiple definitions.
//!
//! The `#[netabase]` macro is used to create a top-level enum that wraps
//! multiple definition enums. This is useful for applications that need
//! to work with multiple definitions polymorphically.
//!
//! # Generated Code
//!
//! Given multiple `#[netabase_definition]` modules, `#[netabase(MyGlobal)]`
//! generates:
//! - `MyGlobal` enum with variants for each definition
//! - Conversions between definitions and the global enum
//! - Strum derives for discriminant matching
//!
//! # Example
//!
//! ```rust,ignore
//! #[netabase(AppData)]
//! mod app {
//!     #[netabase_definition(Users)]
//!     mod users { /* ... */ }
//!     
//!     #[netabase_definition(Posts)]
//!     mod posts { /* ... */ }
//! }
//!
//! // Generated:
//! // enum AppData {
//! //     Users(Users),
//! //     Posts(Posts),
//! // }
//! ```
//!
//! # Use Cases
//!
//! - Multi-definition applications
//! - Plugin systems with dynamic schema
//! - Generic database utilities that work across definitions

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemMod, Path, Result, parse2};

use crate::generators::global::GlobalEnumGenerator;
use crate::utils::attributes::remove_attribute;
use crate::utils::naming::path_last_segment;
use crate::visitors::global::GlobalVisitor;

/// Implementation of the netabase attribute macro.
///
/// This macro processes a module containing multiple `#[netabase_definition]`
/// modules and generates a global enum to wrap them all.
pub fn netabase_attribute(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    // Parse attribute to get global name
    let global_path: Path = parse2(attr)?;

    let global_name = path_last_segment(&global_path)
        .ok_or_else(|| syn::Error::new_spanned(&global_path, "Invalid global name"))?
        .clone();

    // Parse the module
    let mut module: ItemMod = parse2(item)?;

    // Ensure the module has content
    if module.content.is_none() {
        return Err(syn::Error::new_spanned(
            module,
            "netabase can only be applied to modules with content (not external modules)",
        ));
    }

    // Create visitor and collect information
    let mut visitor = GlobalVisitor::new(global_name.clone());
    visitor.visit_module(&module)?;

    // Remove the netabase attribute from the module
    remove_attribute(&mut module.attrs, "netabase");

    // Generate code using generator
    let enum_generator = GlobalEnumGenerator::new(&visitor);
    let global_enum = enum_generator.generate_global_enum();

    // Add generated code to the module
    if let Some((ref _brace, ref mut items)) = module.content {
        // Parse the generated items and add them to the module
        let generated_items: syn::File = parse2(quote! {
            #global_enum
        })?;

        items.extend(generated_items.items.into_iter().map(syn::Item::from));
    }

    Ok(quote! {
        #module
    })
}
