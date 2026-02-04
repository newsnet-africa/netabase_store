//! `schema.toml` and definition schema TOML specification.
//!
//! This module is a normative, human-readable reference for the TOML produced
//! by [`NetabaseDefinition::export_toml`](crate::traits::registry::definition::NetabaseDefinition::export_toml)
//! and consumed by `#[netabase_definition(from_file = "schema.toml")]`.
//! It documents every top-level table, field, and nested structure that can
//! appear in a **definition schema** TOML file.
//!
//! The actual serialization is implemented by
//! [`crate::traits::registry::definition::schema`]; this document mirrors that
//! Rust API but in TOML-centric terms.
//!
//! # Top-Level Structure (`DefinitionSchema`)
//!
//! A `schema.toml` file describes a single *definition* and is the TOML
//! representation of [`DefinitionSchema`](crate::traits::registry::definition::schema::DefinitionSchema).
//!
//! At the top level it contains:
//!
//! - `schema_format_version` — **integer**, schema format version (currently `2`).
//!   - Optional on disk; when omitted, it is treated as `1` for backwards
//!     compatibility.
//! - `name` — **string**, the Rust identifier of the definition (e.g. `"BlogApp"`).
//! - `models` — **array of tables**, `[[models]]`, one entry per current model.
//! - `structs` — **optional array of tables**, `[[structs]]`, used for
//!   additional helper structs that appear in the schema.
//! - `subscriptions` — **array of strings**, all subscription *topic* names
//!   registered on this definition.
//! - `model_history` — **array of tables**, `[[model_history]]`, version
//!   history for models that participate in migration.
//! - `schema_hash` — **optional string**, a 64-bit hash of the schema encoded as
//!   a decimal string (to avoid TOML integer limits).
//! - `config` — **optional table** `[config]`, global configuration for the
//!   definition.
//!
//! ## Example (simplified)
//!
//! ```toml
//! schema_format_version = 2
//! name = "BlogApp"
//!
//! [[models]]
//! name = "Post"
//!
//!   [[models.fields]]
//!   name = "id"
//!   type_name = "PostID"
//!   kind = "Primary"
//!
//!   [[models.fields]]
//!   name = "title"
//!   type_name = "String"
//!   kind = "Regular"
//!
//! [[model_history]]
//! family = "Post"
//! current_version = 1
//!
//!   [[model_history.versions]]
//!   version = 1
//!   struct_name = "Post"
//!   fields = []
//!   subscriptions = []
//!   subscription_immutable = false
//!   version_hash = "1234567890" # string-encoded u64
//!   supports_downgrade = false
//!   supports_upgrade = true
//!   is_libp2p_enabled = false
//!   is_content_addressed = false
//! ```
//!
//! # `[[models]]` — `ModelSchema`
//!
//! Each `[[models]]` entry corresponds to a current model (struct) in the
//! definition and mirrors
//! [`ModelSchema`](crate::traits::registry::definition::schema::ModelSchema).
//!
//! Required fields:
//!
//! - `name` — **string**, Rust struct name (e.g. `"User"`).
//! - `fields` — **array of tables**, `[[models.fields]]`, fields on this model.
//!
//! Optional / inferred fields:
//!
//! - `subscriptions` — **array of strings**, topic names this model subscribes
//!   to (from `#[subscribe(...)]`).
//! - `subscription_immutable` — **bool**, whether subscriptions are immutable
//!   (`#[subscribe(immutable, ...)]`). Defaults to `false` when omitted.
//! - `family` — **optional string**, migration family name
//!   (`#[netabase_version(family = "User", ...)]`).
//! - `version` — **optional integer**, version number within the family.
//! - `is_current` — **bool**, whether this is the current version in the
//!   family. Defaults to `false`.
//! - `is_libp2p_enabled` — **bool**, whether this model is exposed over libp2p.
//!   Defaults to `false`.
//! - `is_content_addressed` — **bool**, whether the primary key is derived from
//!   content (immutable, hash-based IDs). Defaults to `false`.
//! - `content_addressed_config` — **optional table**, see below.
//!
//! ## `[[models.fields]]` — `FieldSchema`
//!
//! Each `[[models.fields]]` entry represents a single struct field and matches
//! [`FieldSchema`](crate::traits::registry::definition::schema::FieldSchema).
//!
//! Common fields:
//!
//! - `name` — **string**, field name as it appears in Rust.
//! - `type_name` — **string**, fully-qualified or simplified Rust type name as
//!   captured by the macro (e.g. `"String"`, `"UserID"`, `"Option<String>"`).
//! - `kind` — **string**, discriminant describing the *role* of the field.
//!
//! The `kind` field comes from
//! [`KeyTypeSchema`](crate::traits::registry::definition::schema::KeyTypeSchema)
//! and can take the following values:
//!
//! - `"Primary"` — primary key field (`#[primary_key]`).
//! - `"Secondary"` — secondary index field (`#[secondary_key]`).
//! - `"Relational"` — relational foreign key (`#[link(OtherDef, Model)]`).
//! - `"Blob"` — blob payload field (`#[blob]`).
//! - `"Regular"` — non-key, non-blob field.
//!
//! For `kind = "Relational"`, an additional `details` table is present:
//!
//! ```toml
//! [[models.fields]]
//! name = "author"
//! type_name = "AuthorID"
//! kind = "Relational"
//!
//!   [models.fields.details]
//!   definition = "UserDef"   # target definition name
//!   model = "User"           # target model name
//! ```
//!
//! For other kinds (`Primary`, `Secondary`, `Blob`, `Regular`), the `details`
//! table is omitted.
//!
//! # `[[model_history]]` — `ModelVersionHistory`
//!
//! Version history is stored under `[[model_history]]` and mirrors
//! [`ModelVersionHistory`](crate::traits::registry::definition::schema::ModelVersionHistory).
//!
//! Required fields:
//!
//! - `family` — **string**, model family identifier (e.g. `"User"`).
//! - `current_version` — **integer**, latest known version number.
//! - `versions` — **array of tables**, `[[model_history.versions]]`.
//!
//! Optional fields:
//!
//! - `migration_paths` — **array of tables**, `[[model_history.migration_paths]]`,
//!   describing upgrade/downgrade steps between versions.
//!
//! ## `[[model_history.versions]]` — `VersionedModelSchema`
//!
//! Each entry describes one concrete version of a model:
//!
//! - `version` — **integer**, version number.
//! - `struct_name` — **string**, Rust struct name at this version (e.g.
//!   `"UserV1"`, `"User"`).
//! - `fields` — **array of tables**, same shape as `[[models.fields]]`.
//! - `subscriptions` — **array of strings**, topics this version subscribes to.
//! - `subscription_immutable` — **bool**, immutability flag for subscriptions.
//! - `version_hash` — **string**, schema hash for this version (string-encoded
//!   `u64`).
//! - `supports_downgrade` — **bool**, whether `MigrateTo` is implemented.
//! - `supports_upgrade` — **bool**, whether `MigrateFrom` from the previous
//!   version is implemented. Defaults to `true` when omitted.
//! - `is_libp2p_enabled` — **bool**, whether this version participates in
//!   libp2p synchronization.
//! - `is_content_addressed` — **bool**, whether primary key is derived from
//!   content.
//! - `content_addressed_config` — **optional table**, see below.
//!
//! ## `[[model_history.migration_paths]]` — `MigrationPathSchema`
//!
//! Each migration path entry specifies how to move between two versions:
//!
//! - `from_version` — **integer**, source version.
//! - `to_version` — **integer**, target version.
//! - `may_lose_data` — **bool**, whether migration may drop data.
//! - `field_changes` — **array of tables**, `[[model_history.migration_paths.field_changes]]`.
//!
//! ### `[[...field_changes]]` — `FieldChangeSchema`
//!
//! The `change_type` tag selects the variant, with additional fields:
//!
//! - `change_type = "Added"` with:
//!   - `name` — **string**
//!   - `type_name` — **string**
//!   - `has_default` — **bool**
//! - `change_type = "Removed"` with:
//!   - `name` — **string**
//!   - `type_name` — **string**
//! - `change_type = "Renamed"` with:
//!   - `old_name` — **string**
//!   - `new_name` — **string**
//! - `change_type = "TypeChanged"` with:
//!   - `name` — **string**
//!   - `old_type` — **string**
//!   - `new_type` — **string**
//!
//! # `[config]` — `DefinitionConfig`
//!
//! The optional `[config]` table controls definition-wide behavior and mirrors
//! [`DefinitionConfig`](crate::traits::registry::definition::schema::DefinitionConfig).
//!
//! Fields:
//!
//! - `retention_days` — **optional integer**, how long to retain old model
//!   versions.
//! - `compression_enabled` — **bool**, whether blob fields are compressed.
//!   Defaults to `false` when omitted.
//! - `max_blob_size` — **optional integer**, maximum allowed blob size in bytes.
//! - `auto_migration` — **bool**, whether automatic migration is enabled.
//!   Defaults to `true` when omitted.
//! - `metadata` — **table**, `[config.metadata]`, arbitrary string key/value
//!   metadata used by tooling.
//!
//! ## `[...content_addressed_config]` — `ContentAddressedConfig`
//!
//! When `is_content_addressed = true` on a model or version entry, an optional
//! `content_addressed_config` table may be present:
//!
//! - `hasher` — **string**, logical name of the hasher (e.g. `"Sha256"`).
//! - `function` — **string**, function path used to compute the hash.
//! - `key_type` — **optional string**, custom key type (defaults to `[u8; 32]`).
//!
//! # Repository-Level Schema (`RepositorySchema`)
//!
//! While `schema.toml` describes a single definition, repository-level schema
//! uses [`RepositorySchema`](crate::traits::registry::definition::schema::RepositorySchema)
//! and is typically written to `repository.toml` by repository macros.
//!
//! Top-level fields:
//!
//! - `schema_format_version` — **integer**, format version (defaults to `1` when
//!   omitted for older files).
//! - `name` — **string**, repository name.
//! - `definitions` — **array of tables**, `[[definitions]]`, each embedding a
//!   full `DefinitionSchema` as described above.
//!
//! In practice, repository TOML is usually generated and consumed by
//! `netabase_macros` (`schema_toml()` / `write_schema_toml()` and
//! `infer_netabase_definition!`), but the structure mirrors this spec exactly so
//! you can inspect and manipulate it with standard TOML tooling.
