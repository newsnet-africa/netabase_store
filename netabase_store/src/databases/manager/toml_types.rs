use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Root TOML file structure for a DefinitionManager
///
/// This file is stored at: `<root_path>/<manager_name>.root.netabase.toml`
///
/// It contains metadata about the manager itself and tracks which definitions
/// are available and currently loaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootToml {
    pub manager: ManagerSection,
    pub definitions: DefinitionsSection,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<PermissionRoleSection>,
}

/// Manager metadata section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerSection {
    /// Name of the manager (e.g., "RestaurantManager")
    pub name: String,

    /// Schema version for this manager
    #[serde(default = "default_version")]
    pub version: String,

    /// Root path where all definition databases are stored
    pub root_path: String,

    /// Timestamp when this manager was created
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,

    /// Timestamp of last update to manager metadata
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

/// Definitions section tracking all definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionsSection {
    /// All available definitions (discriminant names)
    pub all: Vec<String>,

    /// Definitions currently loaded in memory
    #[serde(default)]
    pub loaded: Vec<String>,

    /// Definitions marked for warm-on-access (kept loaded)
    #[serde(default)]
    pub warm_on_access: Vec<String>,
}

/// Permission role definition in root TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRoleSection {
    /// Role name (e.g., "Manager", "Waiter", "Customer")
    pub name: String,

    /// Permission level for this role
    pub level: String,

    /// Definitions this role can access
    pub definitions: Vec<String>,
}

/// Definition TOML file structure
///
/// This file is stored at: `<root_path>/<definition_name>/<definition_name>.netabase.toml`
///
/// It contains metadata about a specific definition including its tree structure,
/// permissions, and schema hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionToml {
    pub definition: DefinitionSection,
    pub trees: TreesSection,
    #[serde(default)]
    pub permissions: DefinitionPermissionsSection,
    pub metadata: DefinitionMetadataSection,
}

/// Definition metadata section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionSection {
    /// Human-readable name of the definition
    pub name: String,

    /// Discriminant name (should match the enum variant)
    pub discriminant: String,

    /// Schema version for this definition
    #[serde(default = "default_version")]
    pub version: String,
}

/// Trees section describing the database structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreesSection {
    /// Main tree name (primary storage)
    pub main: String,

    /// Secondary index tree names
    #[serde(default)]
    pub secondary: Vec<String>,

    /// Relational link tree names
    #[serde(default)]
    pub relational: Vec<String>,

    /// Subscription tree names
    #[serde(default)]
    pub subscription: Vec<String>,
}

/// Permissions section for a definition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DefinitionPermissionsSection {
    /// Other definitions that can reference this one
    #[serde(default)]
    pub can_reference: Vec<String>,

    /// Definitions this one can reference
    #[serde(default)]
    pub references: Vec<String>,
}

/// Metadata section for a definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionMetadataSection {
    /// Timestamp when this definition was created
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,

    /// Timestamp of last schema update
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,

    /// Blake3 hash of the schema structure
    /// Format: "blake3:<hex_hash>"
    pub schema_hash: String,
}

fn default_version() -> String {
    "1".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_toml_serialization() {
        let root = RootToml {
            manager: ManagerSection {
                name: "TestManager".to_string(),
                version: "1".to_string(),
                root_path: "/test/path".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            definitions: DefinitionsSection {
                all: vec!["User".to_string(), "Product".to_string()],
                loaded: vec!["User".to_string()],
                warm_on_access: vec![],
            },
            permissions: vec![],
        };

        let toml_str = toml::to_string_pretty(&root).unwrap();
        assert!(toml_str.contains("TestManager"));
        assert!(toml_str.contains("User"));

        // Verify round-trip
        let parsed: RootToml = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.manager.name, "TestManager");
        assert_eq!(parsed.definitions.all.len(), 2);
    }

    #[test]
    fn test_definition_toml_serialization() {
        let def = DefinitionToml {
            definition: DefinitionSection {
                name: "User".to_string(),
                discriminant: "User".to_string(),
                version: "1".to_string(),
            },
            trees: TreesSection {
                main: "User".to_string(),
                secondary: vec!["User_Email".to_string(), "User_Name".to_string()],
                relational: vec!["User_rel_Products".to_string()],
                subscription: vec!["User_sub_Updates".to_string()],
            },
            permissions: DefinitionPermissionsSection {
                can_reference: vec!["Product".to_string()],
                references: vec![],
            },
            metadata: DefinitionMetadataSection {
                created_at: Utc::now(),
                updated_at: Utc::now(),
                schema_hash: "blake3:abcd1234".to_string(),
            },
        };

        let toml_str = toml::to_string_pretty(&def).unwrap();
        assert!(toml_str.contains("User"));
        assert!(toml_str.contains("User_Email"));
        assert!(toml_str.contains("blake3:abcd1234"));

        // Verify round-trip
        let parsed: DefinitionToml = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.definition.name, "User");
        assert_eq!(parsed.trees.secondary.len(), 2);
    }
}
