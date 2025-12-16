use crate::traits::registery::definition::NetabaseDefinition;

/// Access level for an entire model (all its tables)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelAccessLevel {
    /// Model can only be read
    ReadOnly,
    /// Model can be read and written
    ReadWrite,
    /// Model cannot be accessed
    NoAccess,
}

impl ModelAccessLevel {
    /// Check if this access level allows writes
    pub const fn allows_write(&self) -> bool {
        matches!(self, ModelAccessLevel::ReadWrite)
    }

    /// Check if this access level allows reads
    pub const fn allows_read(&self) -> bool {
        matches!(
            self,
            ModelAccessLevel::ReadOnly | ModelAccessLevel::ReadWrite
        )
    }
}

/// Cross-definition access configuration (definition-to-definition level)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrossDefinitionAccess {
    /// Whether read access is allowed to this definition
    pub read: bool,
    /// Whether write access is allowed to this definition
    pub write: bool,
}

impl CrossDefinitionAccess {
    pub const NONE: Self = Self {
        read: false,
        write: false,
    };

    pub const READ_ONLY: Self = Self {
        read: true,
        write: false,
    };

    pub const READ_WRITE: Self = Self {
        read: true,
        write: true,
    };

    pub const fn new(read: bool, write: bool) -> Self {
        Self { read, write }
    }
}

/// Definition-level permissions using discriminants
/// Specifies per-model access levels within this definition
pub struct DefinitionPermissions<'a, D: NetabaseDefinition>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
    /// Per-model access levels within this definition (using model discriminants)
    pub model_access: &'a [(D::Discriminant, ModelAccessLevel)],
}

impl<'a, D: NetabaseDefinition> DefinitionPermissions<'a, D>
where
    D::Discriminant: 'static + std::fmt::Debug + PartialEq,
{
    /// Create a new DefinitionPermissions with no models accessible
    pub const fn none() -> Self {
        Self { model_access: &[] }
    }

    /// Get the access level for a specific model by its discriminant
    pub fn get_model_access(&self, discriminant: D::Discriminant) -> ModelAccessLevel {
        self.model_access
            .iter()
            .find(|(disc, _)| *disc == discriminant)
            .map(|(_, level)| *level)
            .unwrap_or(ModelAccessLevel::NoAccess)
    }

    /// Check if any model requires write access
    /// Used to determine if database should be opened as read-write
    pub fn requires_write_access(&self) -> bool {
        self.model_access
            .iter()
            .any(|(_, level)| level.allows_write())
    }

    /// Check if a model is accessible at all
    pub fn is_model_accessible(&self, discriminant: D::Discriminant) -> bool {
        self.get_model_access(discriminant) != ModelAccessLevel::NoAccess
    }

    /// Get all models with read-only access
    pub fn get_readonly_models(&self) -> impl Iterator<Item = &D::Discriminant> {
        self.model_access
            .iter()
            .filter(|(_, level)| matches!(level, ModelAccessLevel::ReadOnly))
            .map(|(disc, _)| disc)
    }

    /// Get all models with read-write access
    pub fn get_readwrite_models(&self) -> impl Iterator<Item = &D::Discriminant> {
        self.model_access
            .iter()
            .filter(|(_, level)| matches!(level, ModelAccessLevel::ReadWrite))
            .map(|(disc, _)| disc)
    }
}

// Debug implementation
impl<'a, D: NetabaseDefinition> std::fmt::Debug for DefinitionPermissions<'a, D>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefinitionPermissions")
            .field("model_access", &self.model_access)
            .finish()
    }
}
