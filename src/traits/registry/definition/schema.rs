//! Schema types for exporting and versioning definitions.
//!
//! This module provides types for representing database schemas in a serializable format,
//! primarily for TOML export/import and schema versioning. These types mirror the runtime
//! structure but with serialization support.

#[cfg(feature = "schema_export")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "schema_export")]
/// Serde module for serializing Option<u64> hashes as strings to avoid TOML i64::MAX limitation
mod hash_as_string {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => serializer.serialize_str(&v.to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => s.parse().map(Some).map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

#[cfg(feature = "schema_export")]
/// Serde module for serializing u64 hashes as strings to avoid TOML i64::MAX limitation
mod hash_as_u64_string {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Schema format version for forwards/backwards compatibility.
pub const SCHEMA_FORMAT_VERSION: u32 = 2;

#[cfg(feature = "schema_export")]
fn default_schema_format_version() -> u32 {
    1 // Old schemas without version field are v1
}

/// Repository schema containing multiple definitions
#[cfg_attr(feature = "schema_export", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct RepositorySchema {
    /// Schema format version
    #[cfg_attr(feature = "schema_export", serde(default = "default_schema_format_version"))]
    pub schema_format_version: u32,
    /// Repository name
    pub name: String,
    /// All definitions in this repository
    pub definitions: Vec<DefinitionSchema>,
}

impl RepositorySchema {
    /// Convert the repository schema to a TOML string
    ///
    /// # Feature Requirements
    ///
    /// Requires the `schema_export` feature to be enabled.
    ///
    /// # Example
    ///
    /// ```
    /// # use netabase_store::traits::registry::definition::schema::RepositorySchema;
    /// # let schema = RepositorySchema {
    /// #     schema_format_version: 2,
    /// #     name: "MyRepo".to_string(),
    /// #     definitions: vec![],
    /// # };
    /// let toml_string = schema.to_toml();
    /// assert!(toml_string.contains("MyRepo"));
    /// ```
    #[cfg(feature = "schema_export")]
    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self)
            .unwrap_or_else(|e| format!("# Error serializing repository to TOML: {}", e))
    }

    /// Parse a repository schema from a TOML string
    ///
    /// # Feature Requirements
    ///
    /// Requires the `schema_export` feature to be enabled.
    ///
    /// # Arguments
    ///
    /// * `toml_str` - TOML string containing the repository schema
    ///
    /// # Returns
    ///
    /// Returns `Ok(RepositorySchema)` if parsing succeeds, or an error message if it fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use netabase_store::traits::registry::definition::schema::RepositorySchema;
    /// let toml = r#"
    /// schema_format_version = 2
    /// name = "TestRepo"
    /// definitions = []
    /// "#;
    /// 
    /// let schema = RepositorySchema::from_toml(toml).unwrap();
    /// assert_eq!(schema.name, "TestRepo");
    /// assert_eq!(schema.schema_format_version, 2);
    /// ```
    #[cfg(feature = "schema_export")]
    pub fn from_toml(toml_str: &str) -> Result<Self, String> {
        toml::from_str(toml_str)
            .map_err(|e| format!("Failed to parse repository TOML: {}", e))
    }

    /// Read a repository schema from a TOML file
    ///
    /// # Feature Requirements
    ///
    /// Requires the `schema_export` feature to be enabled.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TOML file
    ///
    /// # Returns
    ///
    /// Returns `Ok(RepositorySchema)` if reading and parsing succeed, or an error message if it fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use netabase_store::traits::registry::definition::schema::RepositorySchema;
    /// # use std::path::Path;
    /// let schema = RepositorySchema::from_toml_file(Path::new("schema.toml")).unwrap();
    /// println!("Loaded repository: {}", schema.name);
    /// ```
    #[cfg(feature = "schema_export")]
    pub fn from_toml_file(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read TOML file {}: {}", path.display(), e))?;
        Self::from_toml(&content)
    }

    /// Compute a hash of the entire repository schema
    pub fn compute_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        for def in &self.definitions {
            def.name.hash(&mut hasher);
            for model in &def.models {
                model.name.hash(&mut hasher);
                for field in &model.fields {
                    field.name.hash(&mut hasher);
                    field.type_name.hash(&mut hasher);
                }
            }
        }
        hasher.finish()
    }
}

#[cfg(feature = "schema_export")]
fn default_auto_migration() -> bool {
    true
}

#[cfg_attr(feature = "schema_export", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct DefinitionSchema {
    /// Schema format version (for parsing old TOML files).
    #[cfg_attr(feature = "schema_export", serde(default = "default_schema_format_version"))]
    pub schema_format_version: u32,
    pub name: String,
    pub models: Vec<ModelSchema>,
    #[cfg_attr(feature = "schema_export", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub structs: Vec<StructSchema>,
    pub subscriptions: Vec<String>,
    /// Model version history for migration support.
    /// Contains all previous versions of models, not just the current ones.
    #[cfg_attr(feature = "schema_export", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub model_history: Vec<ModelVersionHistory>,
    /// Schema hash for quick P2P comparison.
    #[cfg_attr(feature = "schema_export", serde(default, skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "schema_export", serde(with = "hash_as_string"))]
    pub schema_hash: Option<u64>,
    /// Configuration options for this definition.
    #[cfg_attr(feature = "schema_export", serde(default, skip_serializing_if = "Option::is_none"))]
    pub config: Option<DefinitionConfig>,
}

/// Configuration options for a definition.
#[cfg_attr(feature = "schema_export", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct DefinitionConfig {
    /// Retention policy for old model versions (in days).
    #[cfg_attr(feature = "schema_export", serde(default, skip_serializing_if = "Option::is_none"))]
    pub retention_days: Option<u32>,
    /// Whether to enable compression for blob fields.
    #[cfg_attr(feature = "schema_export", serde(default))]
    pub compression_enabled: bool,
    /// Maximum blob size in bytes.
    #[cfg_attr(feature = "schema_export", serde(default, skip_serializing_if = "Option::is_none"))]
    pub max_blob_size: Option<u64>,
    /// Whether to enable automatic migration.
    #[cfg_attr(feature = "schema_export", serde(default = "default_auto_migration"))]
    pub auto_migration: bool,
    /// Custom metadata fields.
    #[cfg_attr(feature = "schema_export", serde(default, skip_serializing_if = "std::collections::HashMap::is_empty"))]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Version history for a single model family.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelVersionHistory {
    /// The model family name (groups all versions of a model).
    pub family: String,
    /// Current version number.
    pub current_version: u32,
    /// All known versions with their schema snapshots.
    pub versions: Vec<VersionedModelSchema>,
    /// Detected migration paths between versions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub migration_paths: Vec<MigrationPathSchema>,
}

/// A snapshot of a model at a specific version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionedModelSchema {
    /// Version number.
    pub version: u32,
    /// Struct name at this version (e.g., "UserV1", "UserV2").
    pub struct_name: String,
    /// Fields at this version.
    pub fields: Vec<FieldSchema>,
    /// Subscriptions at this version.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subscriptions: Vec<String>,
    /// Whether subscriptions are immutable.
    #[serde(default)]
    pub subscription_immutable: bool,
    /// Schema hash for this specific version.
    #[serde(with = "hash_as_u64_string")]
    pub version_hash: u64,
    /// Whether this version implements MigrateTo (can downgrade).
    #[serde(default)]
    pub supports_downgrade: bool,
    /// Whether this version implements MigrateFrom the previous version.
    #[serde(default = "default_true")]
    pub supports_upgrade: bool,
    /// Whether this version supports Libp2p features.
    #[serde(default)]
    pub is_libp2p_enabled: bool,
    /// Whether this model is content-addressed (immutable, hash-based ID).
    #[serde(default)]
    pub is_content_addressed: bool,
    /// Configuration for content-addressed models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_addressed_config: Option<ContentAddressedConfig>,
}

fn default_true() -> bool {
    true
}

/// Migration path information for schema comparison.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationPathSchema {
    /// Source version.
    pub from_version: u32,
    /// Target version.
    pub to_version: u32,
    /// Whether the migration may lose data.
    #[serde(default)]
    pub may_lose_data: bool,
    /// Field changes in this migration step.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub field_changes: Vec<FieldChangeSchema>,
}

/// Describes a field change between versions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "change_type")]
pub enum FieldChangeSchema {
    Added {
        name: String,
        type_name: String,
        has_default: bool,
    },
    Removed {
        name: String,
        type_name: String,
    },
    Renamed {
        old_name: String,
        new_name: String,
    },
    TypeChanged {
        name: String,
        old_type: String,
        new_type: String,
    },
}

impl DefinitionSchema {
    /// Convert the schema to a TOML string.
    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self)
            .unwrap_or_else(|e| format!("# Error serializing to TOML: {}", e))
    }

    /// Compute a hash of the entire schema for quick comparison.
    pub fn compute_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        for model in &self.models {
            model.name.hash(&mut hasher);
            for field in &model.fields {
                field.name.hash(&mut hasher);
                field.type_name.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// Get the current version of a model family.
    pub fn current_version(&self, family: &str) -> Option<u32> {
        self.model_history
            .iter()
            .find(|h| h.family == family)
            .map(|h| h.current_version)
    }

    /// Compare with another schema for P2P conflict resolution.
    pub fn compare(&self, other: &DefinitionSchema) -> SchemaComparisonResult {
        let self_hash = self.schema_hash.unwrap_or_else(|| self.compute_hash());
        let other_hash = other.schema_hash.unwrap_or_else(|| other.compute_hash());

        if self_hash == other_hash {
            return SchemaComparisonResult::Identical;
        }

        let mut local_newer = Vec::new();
        let mut remote_newer = Vec::new();
        let mut conflicts = Vec::new();

        for history in &self.model_history {
            if let Some(other_history) = other
                .model_history
                .iter()
                .find(|h| h.family == history.family)
            {
                match history.current_version.cmp(&other_history.current_version) {
                    std::cmp::Ordering::Greater => {
                        local_newer.push((
                            history.family.clone(),
                            history.current_version,
                            other_history.current_version,
                        ));
                    }
                    std::cmp::Ordering::Less => {
                        remote_newer.push((
                            history.family.clone(),
                            history.current_version,
                            other_history.current_version,
                        ));
                    }
                    std::cmp::Ordering::Equal => {
                        // Same version but different hash = conflict
                        let self_ver = history
                            .versions
                            .iter()
                            .find(|v| v.version == history.current_version);
                        let other_ver = other_history
                            .versions
                            .iter()
                            .find(|v| v.version == other_history.current_version);
                        if let (Some(sv), Some(ov)) = (self_ver, other_ver)
                            && sv.version_hash != ov.version_hash {
                                conflicts.push((history.family.clone(), history.current_version));
                            }
                    }
                }
            }
        }

        if !conflicts.is_empty() {
            SchemaComparisonResult::Conflict {
                families: conflicts,
            }
        } else if !local_newer.is_empty() && remote_newer.is_empty() {
            SchemaComparisonResult::LocalNewer {
                families: local_newer,
            }
        } else if local_newer.is_empty() && !remote_newer.is_empty() {
            SchemaComparisonResult::RemoteNewer {
                families: remote_newer,
            }
        } else if !local_newer.is_empty() && !remote_newer.is_empty() {
            SchemaComparisonResult::Mixed {
                local_newer,
                remote_newer,
            }
        } else {
            SchemaComparisonResult::Identical
        }
    }
}

/// Result of comparing two schemas.
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaComparisonResult {
    /// Schemas are identical.
    Identical,
    /// Local schema is newer for all differing families.
    LocalNewer { families: Vec<(String, u32, u32)> },
    /// Remote schema is newer for all differing families.
    RemoteNewer { families: Vec<(String, u32, u32)> },
    /// Some families newer locally, some remotely.
    Mixed {
        local_newer: Vec<(String, u32, u32)>,
        remote_newer: Vec<(String, u32, u32)>,
    },
    /// Same version numbers but different hashes (diverged).
    Conflict { families: Vec<(String, u32)> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelSchema {
    pub name: String,
    pub fields: Vec<FieldSchema>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subscriptions: Vec<String>,
    /// Whether subscriptions are immutable (from #[subscribe(immutable, ...)]).
    #[serde(default)]
    pub subscription_immutable: bool,
    /// The model family this belongs to (for versioning).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    /// Version number within the family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    /// Whether this is the current (latest) version.
    #[serde(default)]
    pub is_current: bool,
    /// Whether this model supports Libp2p features.
    #[serde(default)]
    pub is_libp2p_enabled: bool,
    /// Whether this model is content-addressed (immutable, hash-based ID).
    #[serde(default)]
    pub is_content_addressed: bool,
    /// Configuration for content-addressed models.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_addressed_config: Option<ContentAddressedConfig>,
}

/// Configuration for content-addressed models.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentAddressedConfig {
    /// The hasher type (e.g., "Sha256").
    pub hasher: String,
    /// The hash function path (e.g., "my_hash_fn").
    pub function: String,
    /// Optional custom key type (defaults to [u8; 32]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructSchema {
    pub name: String,
    pub fields: Vec<StructFieldSchema>,
    #[serde(default)]
    pub is_tuple: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructFieldSchema {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldSchema {
    pub name: String,
    pub type_name: String,
    #[serde(flatten)]
    pub key_type: KeyTypeSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "details")]
pub enum KeyTypeSchema {
    Primary,
    Secondary,
    Relational { definition: String, model: String },
    Blob,
    Regular,
}
