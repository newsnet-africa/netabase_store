use syn::{ItemMod, ItemStruct, Path, Result};
use crate::visitors::model::ModelFieldVisitor;

/// Information about a model within a definition
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: syn::Ident,
    pub visitor: ModelFieldVisitor,
}

/// Information about a regular struct (not a model)
#[derive(Debug, Clone)]
pub struct RegularStructInfo {
    pub name: syn::Ident,
    pub fields: Vec<(Option<syn::Ident>, syn::Type)>,
    pub is_tuple: bool,
}

/// Information about subscription topics defined at the definition level
#[derive(Debug, Clone)]
pub struct DefinitionSubscriptions {
    pub topics: Vec<Path>,
}

/// Visitor that collects information about models within a definition
pub struct DefinitionVisitor {
    pub definition_name: syn::Ident,
    pub models: Vec<ModelInfo>,
    pub regular_structs: Vec<RegularStructInfo>,
    pub subscriptions: DefinitionSubscriptions,
    pub nested_definitions: Vec<DefinitionVisitor>,
}

impl DefinitionVisitor {
    pub fn new(definition_name: syn::Ident, subscriptions: Vec<Path>) -> Self {
        Self {
            definition_name,
            models: Vec::new(),
            regular_structs: Vec::new(),
            subscriptions: DefinitionSubscriptions { topics: subscriptions },
            nested_definitions: Vec::new(),
        }
    }

    /// Visit items in the definition module and collect model information
    pub fn visit_module(&mut self, module: &ItemMod) -> Result<()> {
        if let Some((_, items)) = &module.content {
            for item in items {
                match item {
                    syn::Item::Struct(item_struct) => {
                        self.visit_struct(item_struct)?;
                    }
                    syn::Item::Mod(nested_mod) => {
                        // Check for nested netabase_definition attribute
                        if let Some(attr) = nested_mod.attrs.iter()
                            .find(|a| a.path().is_ident("netabase_definition"))
                        {
                            self.visit_nested_definition(nested_mod)?;
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn visit_struct(&mut self, item_struct: &ItemStruct) -> Result<()> {
        // Only process structs with NetabaseModel derive
        let has_netabase_model = item_struct.attrs.iter().any(|attr| {
            if let syn::Meta::List(meta_list) = &attr.meta {
                if meta_list.path.is_ident("derive") {
                    return meta_list.tokens.to_string().contains("NetabaseModel");
                }
            }
            false
        });

        if !has_netabase_model {
            self.visit_regular_struct(item_struct)?;
            return Ok(());
        }

        let mut visitor = ModelFieldVisitor::new(item_struct.ident.clone());

        // Visit model attributes (for subscribe)
        visitor.visit_model_attributes(&item_struct.attrs)?;

        // Visit fields
        if let syn::Fields::Named(fields) = &item_struct.fields {
            for field in &fields.named {
                visitor.visit_field(field)?;
            }
        }

        // Validate
        visitor.validate()?;

        self.models.push(ModelInfo {
            name: item_struct.ident.clone(),
            visitor,
        });

        Ok(())
    }

    fn visit_regular_struct(&mut self, item_struct: &ItemStruct) -> Result<()> {
        let mut fields_info = Vec::new();
        let mut is_tuple = false;

        match &item_struct.fields {
            syn::Fields::Named(fields) => {
                for field in &fields.named {
                    let name = field.ident.clone();
                    let ty = field.ty.clone();
                    fields_info.push((name, ty));
                }
            }
            syn::Fields::Unnamed(fields) => {
                is_tuple = true;
                for field in &fields.unnamed {
                    let ty = field.ty.clone();
                    fields_info.push((None, ty));
                }
            }
            syn::Fields::Unit => {}
        }

        self.regular_structs.push(RegularStructInfo {
            name: item_struct.ident.clone(),
            fields: fields_info,
            is_tuple,
        });
        Ok(())
    }

    fn visit_nested_definition(&mut self, nested_mod: &ItemMod) -> Result<()> {
        // Parse nested definition attribute
        let attr = nested_mod.attrs.iter()
            .find(|a| a.path().is_ident("netabase_definition"))
            .unwrap();

        let (def_name, subscriptions, _) = crate::utils::attributes::parse_definition_attribute(attr)?;

        let def_name_ident = crate::utils::naming::path_last_segment(&def_name)
            .ok_or_else(|| syn::Error::new_spanned(&def_name, "Invalid definition name"))?
            .clone();

        let mut nested_visitor = DefinitionVisitor::new(def_name_ident, subscriptions);
        nested_visitor.visit_module(nested_mod)?;

        self.nested_definitions.push(nested_visitor);

        Ok(())
    }
}
