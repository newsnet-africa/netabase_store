use super::toml_types::*;
use super::DefinitionManager;
use crate::error::NetabaseResult;
use crate::traits::definition::{DiscriminantName, NetabaseDefinition};
use crate::traits::manager::DefinitionManagerTrait;
use crate::traits::permission::PermissionEnumTrait;
use chrono::Utc;
use std::fmt::Debug;
use std::fs;
use strum::{IntoDiscriminant, IntoEnumIterator};

impl<R, D, P, B> DefinitionManager<R, D, P, B>
where
    R: DefinitionManagerTrait<Definition = D, Permissions = P, Backend = B>,
    D: NetabaseDefinition,
    P: PermissionEnumTrait,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <P as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    /// Generate the root TOML file for this manager
    ///
    /// This creates or updates the file at:
    /// `<root_path>/<manager_name>.root.netabase.toml`
    ///
    /// The file contains:
    /// - Manager metadata (name, version, paths)
    /// - List of all available definitions
    /// - Currently loaded definitions
    /// - Warm-on-access hints
    ///
    /// # Returns
    /// * `Ok(())` - If the file was written successfully
    /// * `Err(IoError)` - If file writing failed
    /// * `Err(TomlError)` - If TOML serialization failed
    pub fn generate_root_toml(&self) -> NetabaseResult<()> {
        let now = Utc::now();

        // Collect all definition discriminants
        let all_definitions: Vec<String> = <D as IntoDiscriminant>::Discriminant::iter()
            .map(|disc| disc.name().to_string())
            .collect();

        // Collect currently loaded definitions
        let loaded_definitions: Vec<String> = self
            .loaded_definitions()
            .iter()
            .map(|disc| disc.name().to_string())
            .collect();

        // Collect warm-on-access definitions
        let warm_definitions: Vec<String> = self
            .warm_on_access
            .iter()
            .map(|disc| disc.name().to_string())
            .collect();

        // Create the root TOML structure
        let root_toml = RootToml {
            manager: ManagerSection {
                name: R::manager_name().to_string(),
                version: "1".to_string(),
                root_path: self.root_path.display().to_string(),
                created_at: now,
                updated_at: now,
            },
            definitions: DefinitionsSection {
                all: all_definitions,
                loaded: loaded_definitions,
                warm_on_access: warm_definitions,
            },
            permissions: vec![], // TODO: Extract permission roles from P in future phases
        };

        // Serialize to TOML string
        let toml_string = toml::to_string_pretty(&root_toml)?;

        // Write to file
        let toml_path = self.root_path.join(format!(
            "{}.root.netabase.toml",
            R::manager_name()
        ));
        fs::write(&toml_path, toml_string)?;

        Ok(())
    }

    /// Generate a definition TOML file for a specific definition
    ///
    /// This creates or updates the file at:
    /// `<root_path>/<definition_name>/<definition_name>.netabase.toml`
    ///
    /// The file contains:
    /// - Definition metadata (name, version)
    /// - Tree structure (main, secondary, relational, subscription)
    /// - Permission information
    /// - Schema hash for version tracking
    ///
    /// # Arguments
    /// * `discriminant` - Which definition to generate TOML for
    ///
    /// # Returns
    /// * `Ok(())` - If the file was written successfully
    /// * `Err(IoError)` - If file writing failed
    /// * `Err(TomlError)` - If TOML serialization failed
    pub fn generate_definition_toml(
        &self,
        discriminant: &<D as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<()> {
        let now = Utc::now();
        let def_name = discriminant.name();

        // Create the definition directory if it doesn't exist
        let def_dir = self.root_path.join(def_name);
        fs::create_dir_all(&def_dir)?;

        // TODO: Extract actual tree names from the definition
        // For now, use placeholder values based on naming conventions
        let main_tree = def_name.to_string();
        let secondary_trees = vec![]; // Would extract from D::TreeManager
        let relational_trees = vec![]; // Would extract from D::TreeManager
        let subscription_trees = vec![]; // Would extract from D::TreeManager

        // Calculate schema hash
        let schema_hash = self.calculate_schema_hash(discriminant);

        // Create the definition TOML structure
        let def_toml = DefinitionToml {
            definition: DefinitionSection {
                name: def_name.to_string(),
                discriminant: def_name.to_string(),
                version: "1".to_string(),
            },
            trees: TreesSection {
                main: main_tree,
                secondary: secondary_trees,
                relational: relational_trees,
                subscription: subscription_trees,
            },
            permissions: DefinitionPermissionsSection {
                can_reference: vec![],
                references: vec![],
            },
            metadata: DefinitionMetadataSection {
                created_at: now,
                updated_at: now,
                schema_hash,
            },
        };

        // Serialize to TOML string
        let toml_string = toml::to_string_pretty(&def_toml)?;

        // Write to file
        let toml_path = def_dir.join(format!("{}.netabase.toml", def_name));
        fs::write(&toml_path, toml_string)?;

        Ok(())
    }

    /// Generate TOML files for all definitions
    ///
    /// This is a convenience method that generates definition TOML files
    /// for every definition in the system.
    ///
    /// # Returns
    /// * `Ok(count)` - Number of definition TOML files generated
    /// * `Err(...)` - If any file generation failed
    pub fn generate_all_definition_tomls(&self) -> NetabaseResult<usize> {
        let mut count = 0;

        for discriminant in <D as IntoDiscriminant>::Discriminant::iter() {
            self.generate_definition_toml(&discriminant)?;
            count += 1;
        }

        Ok(count)
    }

    /// Calculate a schema hash for a definition
    ///
    /// This creates a blake3 hash of the definition's structure to detect
    /// schema changes between versions.
    ///
    /// # Arguments
    /// * `discriminant` - The definition to hash
    ///
    /// # Returns
    /// A string in the format "blake3:<hex_hash>"
    fn calculate_schema_hash(&self, discriminant: &<D as IntoDiscriminant>::Discriminant) -> String {
        // For now, create a simple hash based on the discriminant name
        // In a full implementation, this would hash the complete schema structure
        let def_name = discriminant.name();
        let hash = blake3::hash(def_name.as_bytes());
        format!("blake3:{}", hash.to_hex())
    }

    /// Read and parse the root TOML file
    ///
    /// # Returns
    /// * `Ok(RootToml)` - If the file exists and was parsed successfully
    /// * `Err(IoError)` - If the file doesn't exist or can't be read
    /// * `Err(TomlDeError)` - If parsing failed
    pub fn read_root_toml(&self) -> NetabaseResult<RootToml> {
        let toml_path = self.root_path.join(format!(
            "{}.root.netabase.toml",
            R::manager_name()
        ));
        let toml_string = fs::read_to_string(toml_path)?;
        let root_toml: RootToml = toml::from_str(&toml_string)?;
        Ok(root_toml)
    }

    /// Read and parse a definition TOML file
    ///
    /// # Arguments
    /// * `discriminant` - Which definition to read
    ///
    /// # Returns
    /// * `Ok(DefinitionToml)` - If the file exists and was parsed successfully
    /// * `Err(IoError)` - If the file doesn't exist or can't be read
    /// * `Err(TomlDeError)` - If parsing failed
    pub fn read_definition_toml(
        &self,
        discriminant: &<D as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<DefinitionToml> {
        let def_name = discriminant.name();
        let toml_path = self
            .root_path
            .join(def_name)
            .join(format!("{}.netabase.toml", def_name));
        let toml_string = fs::read_to_string(toml_path)?;
        let def_toml: DefinitionToml = toml::from_str(&toml_string)?;
        Ok(def_toml)
    }

    /// Check if a definition's schema has changed
    ///
    /// Compares the current schema hash with the one stored in the TOML file.
    ///
    /// # Arguments
    /// * `discriminant` - Which definition to check
    ///
    /// # Returns
    /// * `Ok(true)` - If the schema has changed
    /// * `Ok(false)` - If the schema is unchanged
    /// * `Err(...)` - If the TOML file can't be read
    pub fn has_schema_changed(
        &self,
        discriminant: &<D as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<bool> {
        let def_toml = self.read_definition_toml(discriminant)?;
        let current_hash = self.calculate_schema_hash(discriminant);
        Ok(def_toml.metadata.schema_hash != current_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_hash_generation() {
        // Test that schema hashes are consistent
        let hash1 = blake3::hash(b"TestDefinition");
        let hash2 = blake3::hash(b"TestDefinition");
        assert_eq!(hash1, hash2);

        let hash_str = format!("blake3:{}", hash1.to_hex());
        assert!(hash_str.starts_with("blake3:"));
        assert_eq!(hash_str.len(), 7 + 64); // "blake3:" + 64 hex chars
    }
}
