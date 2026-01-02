//! Core migration traits for model version conversion.

/// Trait for upgrading from an older version to a newer version.
///
/// Implement this trait to define how data migrates forward from an older schema.
/// The migration system will automatically chain these implementations when
/// upgrading across multiple versions.
///
/// # Compiler Optimization
///
/// When versions are known at compile time, the Rust compiler will inline and
/// optimize chained conversions through monomorphization. For example, a chain
/// `V1 -> V2 -> V3` will be optimized to a direct `V1 -> V3` conversion in
/// release builds with LTO enabled.
///
/// # Example
///
/// ```
/// use netabase_store::traits::migration::MigrateFrom;
///
/// struct UserV1 {
///     id: u64,
///     name: String,
/// }
///
/// struct UserV2 {
///     id: u64,
///     name: String,
///     email: String,
/// }
///
/// // Manual implementation
/// impl MigrateFrom<UserV1> for UserV2 {
///     fn migrate_from(old: UserV1) -> Self {
///         UserV2 {
///             id: old.id,
///             name: old.name,
///             email: String::from("unknown@example.com"),
///         }
///     }
/// }
///
/// // Or use From which auto-implements MigrateFrom
/// impl From<UserV1> for UserV2 {
///     fn from(old: UserV1) -> Self {
///         UserV2 {
///             id: old.id,
///             name: old.name,
///             email: String::from("unknown@example.com"),
///         }
///     }
/// }
///
/// let v1 = UserV1 { id: 1, name: "Alice".to_string() };
/// let v2 = UserV2::migrate_from(v1);
/// assert_eq!(v2.id, 1);
/// assert_eq!(v2.name, "Alice");
/// assert_eq!(v2.email, "unknown@example.com");
/// ```
pub trait MigrateFrom<OldVersion>: Sized {
    /// Convert from an older version to this version.
    fn migrate_from(old: OldVersion) -> Self;
}

/// Trait for downgrading to an older version (for P2P compatibility).
///
/// Implement this trait when you need to send data to nodes running older
/// versions of the schema. This is optional but recommended for P2P systems.
///
/// # Data Loss Warning
///
/// Downgrading may result in data loss if the new version has fields that
/// don't exist in the old version. Document any data loss in your implementation.
///
/// # Example
///
/// ```
/// use netabase_store::traits::migration::MigrateTo;
///
/// #[derive(Clone)]
/// struct UserV1 {
///     id: u64,
///     name: String,
/// }
///
/// #[derive(Clone)]
/// struct UserV2 {
///     id: u64,
///     name: String,
///     email: String,
/// }
///
/// impl MigrateTo<UserV1> for UserV2 {
///     fn migrate_to(&self) -> UserV1 {
///         UserV1 {
///             id: self.id,
///             name: self.name.clone(),
///             // email field is lost during downgrade
///         }
///     }
///
///     fn would_lose_data(&self) -> bool {
///         !self.email.is_empty()
///     }
/// }
///
/// let v2 = UserV2 {
///     id: 1,
///     name: "Bob".to_string(),
///     email: "bob@example.com".to_string(),
/// };
///
/// assert!(v2.would_lose_data());
/// let v1 = v2.migrate_to();
/// assert_eq!(v1.name, "Bob");
/// ```
pub trait MigrateTo<NewerVersion>: Sized {
    /// Convert this version to an older version.
    ///
    /// Returns `Some` if the conversion is lossless, `None` if data would be lost
    /// and the caller should handle gracefully.
    fn migrate_to(&self) -> NewerVersion;

    /// Check if the conversion would result in data loss.
    fn would_lose_data(&self) -> bool {
        false
    }
}

/// Marker trait for the current (latest) version of a model family.
///
/// The macro system automatically implements this for the highest-versioned
/// model in each family. This is used by the definition macro to select
/// which version to compile into the database schema.
pub trait CurrentVersion: Sized {
    /// The model family name (e.g., "User", "Post").
    const FAMILY: &'static str;

    /// The version number of this model.
    const VERSION: u32;

    /// The schema hash for this version (for P2P comparison).
    fn schema_hash() -> u64;
}

/// Trait for models that belong to a versioned family.
///
/// This is implemented by the macro for all versioned models, not just
/// the current version.
pub trait VersionedModel: Sized {
    /// The model family name.
    const FAMILY: &'static str;

    /// The version number.
    const VERSION: u32;

    /// Whether this is the current (latest) version.
    const IS_CURRENT: bool;
}

/// Trait for types that can be migrated through a chain of versions.
///
/// This is automatically implemented when there's a valid migration path
/// from one version to another, even through intermediate versions.
pub trait MigrateChain<Target>: Sized {
    /// Migrate through the chain to reach the target version.
    fn migrate_chain(self) -> Target;

    /// The number of migration steps in the chain.
    const CHAIN_LENGTH: usize;
}

// Reflexive implementation: any type can migrate to itself
impl<T> MigrateChain<T> for T {
    fn migrate_chain(self) -> T {
        self
    }

    const CHAIN_LENGTH: usize = 0;
}

// Note: Single-step implementations are generated by the macro system
// to avoid orphan rule issues. The macro generates:
// impl MigrateChain<ModelV2> for ModelV1 { ... }
// when ModelV2: MigrateFrom<ModelV1>

/// Metadata about a migration path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationPath {
    /// The source version number.
    pub from_version: u32,
    /// The target version number.
    pub to_version: u32,
    /// The model family name.
    pub family: &'static str,
    /// Number of intermediate steps.
    pub steps: usize,
    /// Whether any step may lose data.
    pub may_lose_data: bool,
}

/// Result of a migration operation.
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Number of records migrated.
    pub records_migrated: usize,
    /// Number of records that failed migration.
    pub records_failed: usize,
    /// Detailed errors if any.
    pub errors: Vec<MigrationError>,
    /// The migration path taken.
    pub path: MigrationPath,
}

/// Error during migration.
#[derive(Debug, Clone)]
pub struct MigrationError {
    /// The primary key of the record that failed (as string).
    pub record_key: String,
    /// Description of the error.
    pub error: String,
    /// The version where the error occurred.
    pub at_version: u32,
}
