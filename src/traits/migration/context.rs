//! Version context for postcard deserialization.
//!
//! This module provides a context type for version-aware deserialization.
//! When receiving data from an older node or reading from an old database,
//! the context triggers automatic migration during deserialization.
//!
//! # Architecture
//!
//! The version context is used during deserialization to detect version mismatches
//! and trigger migrations. This enables transparent upgrades when reading data
//! serialized with older schema versions.
//!
//! # Wire Format Compatibility
//!
//! The 4-byte version prefix is compatible with postcard's wire format because:
//! 1. It's prepended before serialization
//! 2. It's stripped before passing to postcard deserializer
//! 3. The version detection happens at a layer above postcard
//!
//! # Performance
//!
//! Version checking adds minimal overhead (single u32 comparison per deserialization).
//! The migration chain is inlined and optimized by the compiler when versions
//! are known at compile time.

/// Version context for migration-aware deserialization.
///
/// This context enables automatic migration during deserialization.
///
/// # Wire Format
///
/// When version context is enabled, the wire format is:
/// ```text
/// [version: u32 (4 bytes)][payload: remaining bytes]
/// ```
///
/// The deserializer reads the version, and if it differs from the expected
/// version, it deserializes into the appropriate old type and applies the
/// migration chain.
///
/// # Example
///
/// ```
/// use netabase_store::traits::migration::VersionContext;
///
/// // Default context: auto-migrates, not strict
/// let ctx = VersionContext::default();
/// assert!(ctx.auto_migrate);
/// assert!(!ctx.strict);
///
/// // Strict context: fails on version mismatch
/// let strict = VersionContext::strict(3);
/// assert_eq!(strict.expected_version, 3);
/// assert!(!strict.auto_migrate);
/// assert!(strict.strict);
///
/// // Custom context
/// let custom = VersionContext::new(2).with_auto_migrate(false);
/// assert_eq!(custom.expected_version, 2);
/// assert!(!custom.auto_migrate);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionContext {
    /// The expected (current) version for this model family.
    pub expected_version: u32,
    /// The actual version found in the data (set during deserialization).
    pub actual_version: Option<u32>,
    /// Whether to automatically migrate on version mismatch.
    pub auto_migrate: bool,
    /// Whether to fail on version mismatch if auto_migrate is false.
    pub strict: bool,
}

impl Default for VersionContext {
    fn default() -> Self {
        Self {
            expected_version: 0,
            actual_version: None,
            auto_migrate: true,
            strict: false,
        }
    }
}

impl VersionContext {
    /// Create a new version context expecting a specific version.
    pub fn new(expected_version: u32) -> Self {
        Self {
            expected_version,
            auto_migrate: true,
            ..Default::default()
        }
    }

    /// Create a strict context that fails on version mismatch.
    pub fn strict(expected_version: u32) -> Self {
        Self {
            expected_version,
            auto_migrate: false,
            strict: true,
            ..Default::default()
        }
    }

    /// Enable or disable automatic migration.
    pub fn with_auto_migrate(mut self, auto_migrate: bool) -> Self {
        self.auto_migrate = auto_migrate;
        self
    }

    /// Check if migration is needed.
    pub fn needs_migration(&self) -> bool {
        match self.actual_version {
            Some(actual) => actual != self.expected_version,
            None => false,
        }
    }

    /// Get the version delta (positive = upgrade needed, negative = downgrade).
    pub fn version_delta(&self) -> i32 {
        match self.actual_version {
            Some(actual) => self.expected_version as i32 - actual as i32,
            None => 0,
        }
    }
}

/// Trait for types that can be decoded with version awareness.
///
/// This trait supports version-aware deserialization.
/// The macro system generates implementations that read the version header and
/// apply migrations as needed.
pub trait VersionedDecode: Sized {
    /// Decode from bytes with version context.
    ///
    /// # Arguments
    /// * `data` - The serialized bytes (with version header)
    /// * `ctx` - Version context for migration decisions
    ///
    /// # Returns
    /// The decoded value, possibly migrated from an older version.
    fn decode_versioned(data: &[u8], ctx: &VersionContext) -> Result<Self, postcard::Error>;

    /// Decode from bytes without version header (legacy format).
    ///
    /// Assumes the data is in the expected version format.
    fn decode_unversioned(data: &[u8]) -> Result<Self, postcard::Error>;
}

/// Trait for types that can be encoded with version header.
pub trait VersionedEncode: Sized {
    /// Encode to bytes with version header.
    fn encode_versioned(&self) -> Vec<u8>;

    /// Encode to bytes for a specific target version (for P2P downgrade).
    fn encode_for_version(&self, target_version: u32) -> Option<Vec<u8>>;
}

/// Header prepended to versioned data.
///
/// The version header uses a magic byte sequence "NV" followed by a 4-byte
/// version number to identify versioned data and enable format detection.
///
/// # Wire Format
///
/// ```text
/// +--------+--------+---------+---------+---------+---------+
/// | Magic1 | Magic2 | Ver[0]  | Ver[1]  | Ver[2]  | Ver[3]  |
/// | 'N'    | 'V'    | u32 little-endian              |
/// +--------+--------+---------+---------+---------+---------+
///   1 byte   1 byte   4 bytes total = 6 bytes header
/// ```
///
/// # Example
///
/// ```
/// use netabase_store::traits::migration::VersionHeader;
///
/// // Create and serialize a header
/// let header = VersionHeader::new(42);
/// assert_eq!(header.version, 42);
/// let bytes = header.to_bytes();
/// assert_eq!(bytes.len(), 6);
/// assert_eq!(bytes[0], b'N');
/// assert_eq!(bytes[1], b'V');
///
/// // Parse from bytes
/// let parsed = VersionHeader::from_bytes(&bytes).unwrap();
/// assert_eq!(parsed.version, 42);
///
/// // Check if data is versioned
/// assert!(VersionHeader::is_versioned(&bytes));
///
/// let unversioned = vec![0u8, 1, 2, 3];
/// assert!(!VersionHeader::is_versioned(&unversioned));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct VersionHeader {
    /// Magic bytes to identify versioned format: "NV" (Netabase Versioned)
    pub magic: [u8; 2],
    /// Version number
    pub version: u32,
}

impl VersionHeader {
    /// Magic bytes for versioned format.
    pub const MAGIC: [u8; 2] = [b'N', b'V'];

    /// Size of the header in bytes.
    pub const SIZE: usize = 6; // 2 (magic) + 4 (version)

    /// Create a new version header.
    pub fn new(version: u32) -> Self {
        Self {
            magic: Self::MAGIC,
            version,
        }
    }

    /// Check if bytes start with a valid version header.
    pub fn is_versioned(data: &[u8]) -> bool {
        data.len() >= Self::SIZE && data[0] == Self::MAGIC[0] && data[1] == Self::MAGIC[1]
    }

    /// Parse header from bytes.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        if data[0] != Self::MAGIC[0] || data[1] != Self::MAGIC[1] {
            return None;
        }
        let version = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
        Some(Self {
            magic: Self::MAGIC,
            version,
        })
    }

    /// Serialize header to bytes.
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let version_bytes = self.version.to_le_bytes();
        [
            self.magic[0],
            self.magic[1],
            version_bytes[0],
            version_bytes[1],
            version_bytes[2],
            version_bytes[3],
        ]
    }
}

/// Schema version information stored in the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaVersion {
    /// Definition name
    pub definition: String,
    /// Model family versions: (family_name, current_version)
    pub model_versions: Vec<(String, u32)>,
    /// Schema hash for quick comparison
    pub schema_hash: u64,
    /// Timestamp when schema was last updated
    pub updated_at: u64,
}

/// Comparison result for schema versions (for P2P conflict resolution).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaComparison {
    /// Schemas are identical.
    Identical,
    /// Local schema is newer.
    LocalNewer {
        local_versions: Vec<(String, u32)>,
        remote_versions: Vec<(String, u32)>,
    },
    /// Remote schema is newer.
    RemoteNewer {
        local_versions: Vec<(String, u32)>,
        remote_versions: Vec<(String, u32)>,
    },
    /// Schemas diverged (requires conflict resolution).
    Diverged {
        local_only: Vec<(String, u32)>,
        remote_only: Vec<(String, u32)>,
        conflicts: Vec<(String, u32, u32)>, // (family, local_version, remote_version)
    },
}

impl SchemaVersion {
    /// Compare with another schema version.
    pub fn compare(&self, other: &SchemaVersion) -> SchemaComparison {
        if self.schema_hash == other.schema_hash {
            return SchemaComparison::Identical;
        }

        let mut local_newer = false;
        let mut remote_newer = false;
        let mut conflicts = Vec::new();

        for (family, local_ver) in &self.model_versions {
            if let Some((_, remote_ver)) = other.model_versions.iter().find(|(f, _)| f == family) {
                match local_ver.cmp(remote_ver) {
                    std::cmp::Ordering::Greater => local_newer = true,
                    std::cmp::Ordering::Less => remote_newer = true,
                    std::cmp::Ordering::Equal => {}
                }
                if local_ver != remote_ver {
                    conflicts.push((family.clone(), *local_ver, *remote_ver));
                }
            }
        }

        match (local_newer, remote_newer) {
            (true, true) => SchemaComparison::Diverged {
                local_only: vec![],
                remote_only: vec![],
                conflicts,
            },
            (true, false) => SchemaComparison::LocalNewer {
                local_versions: self.model_versions.clone(),
                remote_versions: other.model_versions.clone(),
            },
            (false, true) => SchemaComparison::RemoteNewer {
                local_versions: self.model_versions.clone(),
                remote_versions: other.model_versions.clone(),
            },
            (false, false) => SchemaComparison::Identical,
        }
    }
}
