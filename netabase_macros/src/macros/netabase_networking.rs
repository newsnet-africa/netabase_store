use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemMod, Result, parse2};

use crate::generators::definition::networking::subscription_capability::SubscriptionCapabilityGenerator;
use crate::utils::attributes::find_attribute;
use crate::utils::attributes::parse_definition_attribute;
use crate::visitors::definition::DefinitionVisitor;

pub fn netabase_networking_attribute(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    // Parse the module
    let mut module: ItemMod = parse2(item)?;

    // Ensure the module has content
    if module.content.is_none() {
        return Err(syn::Error::new_spanned(
            module,
            "netabase_networking can only be applied to modules with content (not external modules)",
        ));
    }

    // Try to find netabase_definition attribute to get configuration
    let def_attr = find_attribute(&module.attrs, "netabase_definition").ok_or_else(|| {
        syn::Error::new_spanned(
            &module,
            "netabase_networking requires #[netabase_definition] attribute on the same module to infer configuration",
        )
    })?;

    let config = parse_definition_attribute(def_attr)?;

    // Create visitor and collect information
    let mut visitor = DefinitionVisitor::new(
        crate::utils::naming::path_last_segment(&config.definition)
            .expect("Invalid definition name")
            .clone(),
        config.subscriptions,
        config.repositories,
    );
    visitor.visit_module(&module)?;

    // Group models by family (needed for proper model info)
    visitor.group_model_families();

    // Generate Capabilities
    let capability_generator = SubscriptionCapabilityGenerator::new(&visitor);
    let capabilities = capability_generator.generate_all_subscription_capabilities();

    // Append generated code to the module
    if let Some((_, items)) = &mut module.content {
        let cap_file: syn::File = parse2(capabilities).map_err(|e| {
            syn::Error::new(e.span(), format!("Failed to parse capability items: {}", e))
        })?;
        items.extend(cap_file.items.into_iter().map(syn::Item::from));
    }

    Ok(quote! {
        #module
    })
}
