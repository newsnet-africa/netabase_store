use serde::{Deserialize, Serialize};

/// Schema format version for forwards/backwards compatibility.
pub const SCHEMA_FORMAT_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DefinitionSchema {
    /// Schema format version (for parsing old TOML files).
    #[serde(default = "default_schema_format_version")]
    pub schema_format_version: u32,
    pub name: String,
    pub models: Vec<ModelSchema>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structs: Vec<StructSchema>,
    pub subscriptions: Vec<String>,
    /// Model version history for migration support.
    /// Contains all previous versions of models, not just the current ones.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub model_history: Vec<ModelVersionHistory>,
    /// Schema hash for quick P2P comparison.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_hash: Option<u64>,
}

fn default_schema_format_version() -> u32 {
    1 // Old schemas without version field are v1
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
    /// Schema hash for this specific version.
    pub version_hash: u64,
    /// Whether this version implements MigrateTo (can downgrade).
    #[serde(default)]
    pub supports_downgrade: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelSchema {
    pub name: String,
    pub fields: Vec<FieldSchema>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subscriptions: Vec<String>,
    /// The model family this belongs to (for versioning).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    /// Version number within the family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    /// Whether this is the current (latest) version.
    #[serde(default)]
    pub is_current: bool,
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

impl DefinitionSchema {
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

    /// Get all versions of a model family.
    pub fn all_versions(&self, family: &str) -> Vec<u32> {
        self.model_history
            .iter()
            .find(|h| h.family == family)
            .map(|h| h.versions.iter().map(|v| v.version).collect())
            .unwrap_or_default()
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
                        if let (Some(sv), Some(ov)) = (self_ver, other_ver) {
                            if sv.version_hash != ov.version_hash {
                                conflicts.push((history.family.clone(), history.current_version));
                            }
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

impl ModelVersionHistory {
    /// Get the schema for a specific version.
    pub fn get_version(&self, version: u32) -> Option<&VersionedModelSchema> {
        self.versions.iter().find(|v| v.version == version)
    }

    /// Check if we can migrate from a given version.
    pub fn can_migrate_from(&self, version: u32) -> bool {
        version <= self.current_version && self.versions.iter().any(|v| v.version == version)
    }

    /// Get the migration path from one version to another.
    pub fn migration_path(&self, from: u32, to: u32) -> Option<Vec<u32>> {
        if from == to {
            return Some(vec![]);
        }

        let versions: Vec<u32> = self.versions.iter().map(|v| v.version).collect();

        if from < to {
            // Upgrade path
            let path: Vec<u32> = versions
                .iter()
                .copied()
                .filter(|&v| v > from && v <= to)
                .collect();
            if path.is_empty() || *path.last()? != to {
                None
            } else {
                Some(path)
            }
        } else {
            // Downgrade path
            let path: Vec<u32> = versions
                .iter()
                .copied()
                .rev()
                .filter(|&v| v < from && v >= to)
                .collect();
            if path.is_empty() || *path.last()? != to {
                None
            } else {
                Some(path)
            }
        }
    }
}
