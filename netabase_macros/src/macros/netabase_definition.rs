use proc_macro2::TokenStream;
use quote::quote;
use std::fs;
use std::path::PathBuf;
use syn::{ItemMod, Result, parse2, visit_mut::VisitMut};

use crate::generators::definition::{DefinitionEnumGenerator, DefinitionTraitGenerator};
use crate::generators::model::{
    KeyEnumGenerator, MigrationGenerator, SerializationGenerator, WrapperTypeGenerator,
};
use crate::generators::structure::StructureGenerator;
use crate::utils::attributes::{parse_definition_attribute_from_tokens, remove_attribute};
use crate::utils::naming::path_last_segment;
use crate::utils::schema::DefinitionSchema;
use crate::visitors::definition::DefinitionVisitor;
use crate::visitors::model::ModelMutator;

/// Implementation of the netabase_definition attribute macro
pub fn netabase_definition_attribute(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    // Parse attribute to get definition name, subscriptions, and repositories
    let config = parse_definition_attribute_from_tokens(attr)?;

    let definition_name = path_last_segment(&config.definition)
        .ok_or_else(|| syn::Error::new_spanned(&config.definition, "Invalid definition name"))?
        .clone();

    // Parse the module
    let mut module: ItemMod = parse2(item)?;

    // Ensure the module has content
    if module.content.is_none() {
        return Err(syn::Error::new_spanned(
            module,
            "netabase_definition can only be applied to modules with content (not external modules)",
        ));
    }

    // Handle file import if specified
    if let Some(file_path) = config.from_file {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").map_err(|e| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to get CARGO_MANIFEST_DIR: {}", e),
            )
        })?;
        let path = PathBuf::from(manifest_dir).join(file_path);

        let content = fs::read_to_string(&path).map_err(|e| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to read schema file at {:?}: {}", path, e),
            )
        })?;

        let schema: DefinitionSchema = toml::from_str(&content).map_err(|e| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to parse schema TOML: {}", e),
            )
        })?;

        // Generate structs from schema
        let generated_structs = StructureGenerator::generate(&schema);

        // Parse generated code into items and inject into module
        let file: syn::File = parse2(generated_structs)?;
        if let Some((_, items)) = &mut module.content {
            items.extend(file.items.into_iter().map(syn::Item::from));
        }
    }

    // 1. Create visitor and collect information (Read-only pass)
    let mut visitor = DefinitionVisitor::new(
        definition_name.clone(),
        config.subscriptions.clone(),
        config.repositories.clone(),
    );
    visitor.visit_module(&module)?;

    // Group models by family for versioning support
    visitor.group_model_families();

    // 2. Generate Definition-level code
    let enum_generator = DefinitionEnumGenerator::new(&visitor);
    let definition_enum = enum_generator.generate_definition_enum();
    let subscriptions_enum = enum_generator.generate_subscriptions_enum();
    let definition_keys_enum = enum_generator.generate_definition_keys_enum();
    let definition_tree_names_enum = enum_generator.generate_definition_tree_names_enum();

    let def_trait_generator = DefinitionTraitGenerator::new(&visitor);
    let def_trait_impls = def_trait_generator.generate();

    // 3. Generate Model-level code for each collected model
    let mut model_generated_code = Vec::new();

    for model_info in &visitor.models {
        let model_visitor = &model_info.visitor;

        // Wrapper Types (ID, wrappers)
        let wrappers = WrapperTypeGenerator::new(model_visitor).generate();
        model_generated_code.push(wrappers);

        // Key Enums
        let keys = KeyEnumGenerator::new(model_visitor).generate();
        model_generated_code.push(keys);

        // Traits (NetabaseModel, NetabaseModelKeys) are handled by DefinitionTraitGenerator

        // Serialization (redb::Value, redb::Key, NetabaseBlobItem)
        let ser_gen = SerializationGenerator::new(model_visitor);
        let model_ser = ser_gen.generate_model_value_key();
        let key_ser = ser_gen.generate_key_enum_value_key();
        let blob_traits = ser_gen.generate_blob_traits();
        model_generated_code.push(model_ser);
        model_generated_code.push(key_ser);
        model_generated_code.push(blob_traits);
    }

    // 3.5. Generate Migration-related code if we have versioned models
    let mut migration_code = TokenStream::new();
    if visitor.has_versioned_models() {
        let migration_gen = MigrationGenerator::new(&visitor);
        migration_code = migration_gen.generate();
    }

    // 4. Mutate the module content (Transform structs)
    let mut mutator = ModelMutator::new(definition_name.clone());
    mutator.visit_item_mod_mut(&mut module);

    // Remove the netabase_definition attribute from the module
    remove_attribute(&mut module.attrs, "netabase_definition");

    // 5. Append generated code to the module
    if let Some((ref _brace, ref mut items)) = module.content {
        // Add definition-level items
        // Parse them first to ensure validity and separate items
        let def_items_tokens = quote! {
            #definition_enum
            #subscriptions_enum
            #definition_keys_enum
            #definition_tree_names_enum
            #def_trait_impls
        };

        let def_file: syn::File = parse2(def_items_tokens).map_err(|e| {
            syn::Error::new(e.span(), format!("Failed to parse definition items: {}", e))
        })?;

        items.extend(def_file.items.into_iter().map(syn::Item::from));

        // Add model-level items
        for code in model_generated_code {
            let file: syn::File = parse2(code).map_err(|e| {
                syn::Error::new(e.span(), format!("Failed to parse model items: {}", e))
            })?;
            items.extend(file.items.into_iter().map(syn::Item::from));
        }

        // Add migration-related items if we have versioned models
        if !migration_code.is_empty() {
            let migration_file: syn::File = parse2(migration_code).map_err(|e| {
                syn::Error::new(e.span(), format!("Failed to parse migration items: {}", e))
            })?;

            items.extend(migration_file.items.into_iter().map(syn::Item::from));
        }
    }

    Ok(quote! {
        #module
    })
}
