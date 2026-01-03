//! Repository visitor for collecting definitions that belong to a repository.
//!
//! This visitor scans a module for definitions with `repos(...)` containing
//! the target repository name, and builds a dependency graph for complete
//! data graph validation.

use std::collections::{HashMap, HashSet};
use syn::{ItemMod, Result};

use crate::utils::attributes::parse_definition_attribute;
use crate::visitors::definition::DefinitionVisitor;
use crate::visitors::model::field::FieldKeyType;

/// Information about a definition within a repository
#[derive(Debug, Clone)]
pub struct RepositoryDefinitionInfo {
    pub name: syn::Ident,
    pub visitor: DefinitionVisitor,
    /// Definitions that this definition links to (via relational links)
    pub link_targets: Vec<syn::Ident>,
}

/// Information about a relational link that escapes repository boundary
#[derive(Debug, Clone)]
pub struct EscapedLink {
    pub source_definition: syn::Ident,
    pub source_model: syn::Ident,
    pub source_field: syn::Ident,
    pub target_definition: syn::Ident,
    pub target_model: syn::Ident,
}

/// Result of cycle detection in the dependency graph
#[derive(Debug, Clone)]
pub struct CycleInfo {
    pub definitions: Vec<syn::Ident>,
}

/// Visitor that collects definitions belonging to a repository
pub struct RepositoryVisitor {
    /// The repository name
    pub repository_name: syn::Ident,
    /// Definitions that belong to this repository
    pub definitions: Vec<RepositoryDefinitionInfo>,
    /// All definition names scanned (including those not in this repository)
    pub all_definition_names: HashSet<String>,
    /// Links that escape the repository boundary
    pub escaped_links: Vec<EscapedLink>,
    /// Cycles detected in the dependency graph
    pub cycles: Vec<CycleInfo>,
    /// Missing definitions required by relational links
    pub missing_definitions: Vec<(syn::Ident, syn::Ident)>, // (source_def, target_def)
    /// External definitions explicitly listed in the repository attribute
    pub external_definitions: Vec<syn::Ident>,
}

impl RepositoryVisitor {
    pub fn new(repository_name: syn::Ident) -> Self {
        Self {
            repository_name,
            definitions: Vec::new(),
            all_definition_names: HashSet::new(),
            escaped_links: Vec::new(),
            cycles: Vec::new(),
            missing_definitions: Vec::new(),
            external_definitions: Vec::new(),
        }
    }

    /// Add external definitions from attribute
    pub fn add_external_definitions(&mut self, defs: Vec<syn::Ident>) {
        for def in defs {
            self.all_definition_names.insert(def.to_string());
            // Create a minimal definition info for external definitions
            self.definitions.push(RepositoryDefinitionInfo {
                name: def.clone(),
                visitor: crate::visitors::definition::DefinitionVisitor::new(
                    def.clone(),
                    vec![],
                    vec![],
                ),
                link_targets: vec![],
            });
            self.external_definitions.push(def);
        }
    }

    /// Visit all items in the module and collect definitions belonging to this repository
    pub fn visit_module(&mut self, module: &ItemMod) -> Result<()> {
        if let Some((_, items)) = &module.content {
            // First pass: collect all definition names and those in this repository
            for item in items {
                if let syn::Item::Mod(nested_mod) = item {
                    if let Some(attr) = nested_mod
                        .attrs
                        .iter()
                        .find(|a| a.path().is_ident("netabase_definition"))
                    {
                        self.visit_definition_module(nested_mod, attr)?;
                    }
                }
            }

            // Second pass: validate complete data graph
            self.validate_data_graph()?;

            // Third pass: detect cycles
            self.detect_cycles();
        }

        Ok(())
    }

    fn visit_definition_module(&mut self, module: &ItemMod, attr: &syn::Attribute) -> Result<()> {
        let config = parse_definition_attribute(attr)?;

        let def_name = crate::utils::naming::path_last_segment(&config.definition)
            .ok_or_else(|| syn::Error::new_spanned(&config.definition, "Invalid definition name"))?
            .clone();

        self.all_definition_names.insert(def_name.to_string());

        // Check if this definition belongs to our repository
        if config.belongs_to_repository(&self.repository_name) {
            // Create a definition visitor to collect model information
            let mut def_visitor = DefinitionVisitor::new(
                def_name.clone(),
                config.subscriptions.clone(),
                config.repositories.clone(),
            );
            def_visitor.visit_module(module)?;

            // Collect link targets from all models
            let link_targets = self.collect_link_targets(&def_visitor);

            self.definitions.push(RepositoryDefinitionInfo {
                name: def_name,
                visitor: def_visitor,
                link_targets,
            });
        }

        Ok(())
    }

    fn collect_link_targets(&self, def_visitor: &DefinitionVisitor) -> Vec<syn::Ident> {
        let mut targets = Vec::new();

        for model_info in &def_visitor.models {
            for field_info in &model_info.visitor.relational_keys {
                // Extract target definition from the link path in FieldKeyType::Relational
                if let FieldKeyType::Relational { definition, .. } = &field_info.key_type {
                    if let Some(target_def) = crate::utils::naming::path_last_segment(definition) {
                        targets.push(target_def.clone());
                    }
                }
            }
        }

        targets
    }

    /// Validate that all linked definitions are present in the repository
    fn validate_data_graph(&mut self) -> Result<()> {
        let repo_def_names: HashSet<String> = self
            .definitions
            .iter()
            .map(|d| d.name.to_string())
            .collect();

        for def_info in &self.definitions {
            for target in &def_info.link_targets {
                let target_name = target.to_string();
                if !repo_def_names.contains(&target_name) {
                    self.missing_definitions
                        .push((def_info.name.clone(), target.clone()));
                }
            }
        }

        Ok(())
    }

    /// Detect cycles in the definition dependency graph using DFS
    fn detect_cycles(&mut self) {
        let def_names: Vec<String> = self
            .definitions
            .iter()
            .map(|d| d.name.to_string())
            .collect();

        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
        for def_info in &self.definitions {
            let edges: Vec<String> = def_info
                .link_targets
                .iter()
                .map(|t| t.to_string())
                .filter(|t| def_names.contains(t))
                .collect();
            adjacency.insert(def_info.name.to_string(), edges);
        }

        let mut visited: HashSet<String> = HashSet::new();
        let mut rec_stack: HashSet<String> = HashSet::new();
        let mut path: Vec<String> = Vec::new();

        for def_name in &def_names {
            if !visited.contains(def_name) {
                self.dfs_detect_cycle(
                    def_name,
                    &adjacency,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                );
            }
        }
    }

    fn dfs_detect_cycle(
        &mut self,
        node: &str,
        adjacency: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = adjacency.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_detect_cycle(neighbor, adjacency, visited, rec_stack, path);
                } else if rec_stack.contains(neighbor) {
                    // Cycle detected - extract cycle from path
                    if let Some(start_idx) = path.iter().position(|n| n == neighbor) {
                        let cycle_defs: Vec<syn::Ident> = path[start_idx..]
                            .iter()
                            .map(|n| syn::Ident::new(n, proc_macro2::Span::call_site()))
                            .collect();

                        // Only add if cycle is not already recorded
                        let cycle_key: HashSet<String> =
                            cycle_defs.iter().map(|i| i.to_string()).collect();
                        let already_exists = self.cycles.iter().any(|c| {
                            let existing_key: HashSet<String> =
                                c.definitions.iter().map(|i| i.to_string()).collect();
                            existing_key == cycle_key
                        });

                        if !already_exists {
                            self.cycles.push(CycleInfo {
                                definitions: cycle_defs,
                            });
                        }
                    }
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
    }

    /// Generate compile error for missing definitions
    pub fn generate_missing_error(&self) -> Option<syn::Error> {
        if self.missing_definitions.is_empty() {
            return None;
        }

        let missing_list: Vec<String> = self.missing_definitions
            .iter()
            .map(|(src, target)| {
                format!(
                    "  - Definition `{}` required by relational link in `{}` - add `repos({}, ...)` to `{}`'s #[netabase_definition]",
                    target, src, self.repository_name, target
                )
            })
            .collect();

        let error_msg = format!(
            "Repository `{}` has incomplete data graph. Missing definitions:\n{}",
            self.repository_name,
            missing_list.join("\n")
        );

        Some(syn::Error::new_spanned(&self.repository_name, error_msg))
    }

    /// Generate warnings for detected cycles
    pub fn generate_cycle_warnings(&self) -> Vec<String> {
        self.cycles
            .iter()
            .map(|cycle| {
                let cycle_str: Vec<String> = cycle.definitions
                    .iter()
                    .map(|d| d.to_string())
                    .collect();
                format!(
                    "Warning: Circular definition dependencies detected in repository `{}`: {} → {}",
                    self.repository_name,
                    cycle_str.join(" → "),
                    cycle_str.first().unwrap_or(&"?".to_string())
                )
            })
            .collect()
    }

    /// Get all definition names in this repository
    pub fn definition_names(&self) -> Vec<&syn::Ident> {
        self.definitions.iter().map(|d| &d.name).collect()
    }

    /// Get all model names across all definitions in this repository
    pub fn model_names(&self) -> Vec<(syn::Ident, syn::Ident)> {
        let mut models = Vec::new();
        for def_info in &self.definitions {
            for model_info in &def_info.visitor.models {
                models.push((def_info.name.clone(), model_info.name.clone()));
            }
        }
        models
    }
}
