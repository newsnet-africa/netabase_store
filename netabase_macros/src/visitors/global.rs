use crate::visitors::definition::DefinitionVisitor;
use syn::{ItemMod, Result};

/// Information about all definitions at the global level
pub struct GlobalVisitor {
    pub global_name: syn::Ident,
    pub definitions: Vec<DefinitionVisitor>,
}

impl GlobalVisitor {
    pub fn new(global_name: syn::Ident) -> Self {
        Self {
            global_name,
            definitions: Vec::new(),
        }
    }

    /// Visit items in the global module and collect definition information
    pub fn visit_module(&mut self, module: &ItemMod) -> Result<()> {
        if let Some((_, items)) = &module.content {
            for item in items {
                if let syn::Item::Mod(nested_mod) = item {
                    // Check for netabase_definition attribute
                    if nested_mod
                        .attrs
                        .iter()
                        .any(|a| a.path().is_ident("netabase_definition"))
                    {
                        self.visit_definition(nested_mod)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn visit_definition(&mut self, def_mod: &ItemMod) -> Result<()> {
        let attr = def_mod
            .attrs
            .iter()
            .find(|a| a.path().is_ident("netabase_definition"))
            .unwrap();

        let config = crate::utils::attributes::parse_definition_attribute(attr)?;

        let def_name_ident = crate::utils::naming::path_last_segment(&config.definition)
            .ok_or_else(|| syn::Error::new_spanned(&config.definition, "Invalid definition name"))?
            .clone();

        let mut def_visitor =
            DefinitionVisitor::new(def_name_ident, config.subscriptions, config.repositories);
        def_visitor.visit_module(def_mod)?;

        self.definitions.push(def_visitor);

        Ok(())
    }
}
