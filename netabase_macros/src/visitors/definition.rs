use crate::visitors::model::ModelFieldVisitor;
use crate::visitors::model::ModelVersionInfo;
use std::collections::HashMap;
use syn::{ItemMod, ItemStruct, Path, Result};

/// Information about a model within a definition
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: syn::Ident,
    pub visitor: ModelFieldVisitor,
}

impl ModelInfo {
    /// Get the version info if this model is versioned.
    pub fn version_info(&self) -> Option<&ModelVersionInfo> {
        self.visitor.version_info.as_ref()
    }

    /// Get the family name if versioned, otherwise use the struct name.
    pub fn family_name(&self) -> String {
        self.version_info()
            .map(|v| v.family.clone())
            .unwrap_or_else(|| self.name.to_string())
    }

    /// Get the version number if versioned, otherwise 1.
    pub fn version(&self) -> u32 {
        self.version_info().map(|v| v.version).unwrap_or(1)
    }
}

/// A family of versioned models.
#[derive(Debug, Clone)]
pub struct ModelFamily {
    /// The family name.
    pub family: String,
    /// All versions of this model, ordered by version number.
    pub versions: Vec<ModelInfo>,
    /// The current (latest) version.
    pub current_version: u32,
    /// Index of the current model in the versions vec.
    pub current_index: usize,
}

impl ModelFamily {
    /// Get the current (latest) model.
    pub fn current_model(&self) -> &ModelInfo {
        &self.versions[self.current_index]
    }

    /// Get a model by version number.
    pub fn get_version(&self, version: u32) -> Option<&ModelInfo> {
        self.versions.iter().find(|m| m.version() == version)
    }

    /// Get all version numbers.
    pub fn all_versions(&self) -> Vec<u32> {
        self.versions.iter().map(|m| m.version()).collect()
    }
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
#[derive(Debug, Clone)]
pub struct DefinitionVisitor {
    pub definition_name: syn::Ident,
    pub models: Vec<ModelInfo>,
    pub regular_structs: Vec<RegularStructInfo>,
    pub subscriptions: DefinitionSubscriptions,
    pub nested_definitions: Vec<DefinitionVisitor>,
    /// Repository identifiers this definition belongs to
    pub repositories: Vec<syn::Ident>,
    /// Model families grouped by family name (populated after visiting).
    pub model_families: HashMap<String, ModelFamily>,
}

impl DefinitionVisitor {
    pub fn new(
        definition_name: syn::Ident,
        subscriptions: Vec<Path>,
        repositories: Vec<syn::Ident>,
    ) -> Self {
        Self {
            definition_name,
            models: Vec::new(),
            regular_structs: Vec::new(),
            subscriptions: DefinitionSubscriptions {
                topics: subscriptions,
            },
            nested_definitions: Vec::new(),
            repositories,
            model_families: HashMap::new(),
        }
    }

    /// Check if this definition belongs to a specific repository
    pub fn belongs_to_repository(&self, repo_name: &syn::Ident) -> bool {
        self.repositories.iter().any(|r| r == repo_name)
    }

    /// Get repository discriminant names as strings
    pub fn repository_discriminant_names(&self) -> Vec<String> {
        self.repositories.iter().map(|r| r.to_string()).collect()
    }

    /// Group models by their family name and determine current versions.
    /// Call this after visiting all models.
    pub fn group_model_families(&mut self) {
        let mut families: HashMap<String, Vec<ModelInfo>> = HashMap::new();

        // Group models by family
        for model in &self.models {
            let family = model.family_name();
            families.entry(family).or_default().push(model.clone());
        }

        // Build ModelFamily for each group
        for (family, mut versions) in families {
            // Sort by version number
            versions.sort_by_key(|m| m.version());

            // Find current version: either explicitly marked or highest version
            let current_index = versions
                .iter()
                .enumerate()
                .find(|(_, m)| {
                    m.version_info()
                        .map(|v| v.is_current == Some(true))
                        .unwrap_or(false)
                })
                .map(|(i, _)| i)
                .unwrap_or(versions.len() - 1); // Default to highest version

            let current_version = versions[current_index].version();

            self.model_families.insert(
                family.clone(),
                ModelFamily {
                    family,
                    versions,
                    current_version,
                    current_index,
                },
            );
        }
    }

    /// Get the current (latest) models for compilation.
    /// Returns only one model per family (the current version).
    pub fn current_models(&self) -> Vec<&ModelInfo> {
        self.model_families
            .values()
            .map(|f| f.current_model())
            .collect()
    }

    /// Get all versioned families.
    pub fn versioned_families(&self) -> Vec<&ModelFamily> {
        self.model_families
            .values()
            .filter(|f| {
                f.versions.len() > 1
                    || f.versions
                        .first()
                        .map(|m| m.version_info().is_some())
                        .unwrap_or(false)
            })
            .collect()
    }

    /// Check if this definition has any versioned models.
    pub fn has_versioned_models(&self) -> bool {
        self.models.iter().any(|m| m.version_info().is_some())
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
                        if let Some(attr) = nested_mod
                            .attrs
                            .iter()
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
        let attr = nested_mod
            .attrs
            .iter()
            .find(|a| a.path().is_ident("netabase_definition"))
            .unwrap();

        let config = crate::utils::attributes::parse_definition_attribute(attr)?;

        let def_name_ident = crate::utils::naming::path_last_segment(&config.definition)
            .ok_or_else(|| syn::Error::new_spanned(&config.definition, "Invalid definition name"))?
            .clone();

        let mut nested_visitor =
            DefinitionVisitor::new(def_name_ident, config.subscriptions, config.repositories);
        nested_visitor.visit_module(nested_mod)?;

        self.nested_definitions.push(nested_visitor);

        Ok(())
    }
}
