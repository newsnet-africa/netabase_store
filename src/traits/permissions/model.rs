use crate::traits::registery::definition::NetabaseDefinition;
use crate::relational::GlobalDefinitionEnum;
use super::traits::AccessType;

/// Access level for a specific model or relation
/// This structure is const-compatible and can be used in const contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessLevel {
    pub read: bool,
    pub create: bool,
    pub update: bool,
    pub delete: bool,
    pub hydrate: bool,
}

impl AccessLevel {
    pub const NONE: Self = Self {
        read: false,
        create: false,
        update: false,
        delete: false,
        hydrate: false,
    };

    pub const READ_ONLY: Self = Self {
        read: true,
        create: false,
        update: false,
        delete: false,
        hydrate: true,
    };

    pub const FULL: Self = Self {
        read: true,
        create: true,
        update: true,
        delete: true,
        hydrate: true,
    };

    pub const fn new(read: bool, create: bool, update: bool, delete: bool, hydrate: bool) -> Self {
        Self {
            read,
            create,
            update,
            delete,
            hydrate,
        }
    }

    /// Check if the given access type is allowed
    pub const fn allows(&self, access: AccessType) -> bool {
        match access {
            AccessType::Read => self.read,
            AccessType::Create => self.create,
            AccessType::Update => self.update,
            AccessType::Delete => self.delete,
        }
    }

    /// Check if hydration is allowed
    pub const fn allows_hydrate(&self) -> bool {
        self.hydrate
    }
}

/// Access level for cross-definition permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrossAccessLevel {
    pub read: bool,
    pub hydrate: bool,
}

impl CrossAccessLevel {
    pub const NONE: Self = Self {
        read: false,
        hydrate: false,
    };

    pub const READ: Self = Self {
        read: true,
        hydrate: true,
    };

    pub const fn new(read: bool, hydrate: bool) -> Self {
        Self { read, hydrate }
    }

    pub const fn allows_read(&self) -> bool {
        self.read
    }

    pub const fn allows_hydrate(&self) -> bool {
        self.hydrate
    }
}

/// Model-level permissions using discriminants for type safety
/// This structure is const-compatible and follows the same pattern as TREE_NAMES
///
/// Permission hierarchy:
/// - `outbound` and `inbound`: Same-definition access (Model → Model within same Definition)
/// - `cross_definition`: Cross-definition access (Model in Definition1 → Model in Definition2)
///   Uses GlobalKeys which specify both the target definition AND the target model
pub struct ModelPermissions<'a, D: NetabaseDefinition + GlobalDefinitionEnum>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
    /// Outbound: Which models this model can access within the same definition
    pub outbound: &'a [(D::Discriminant, AccessLevel)],

    /// Inbound: Which models can access this model within the same definition
    pub inbound: &'a [(D::Discriminant, AccessLevel)],

    /// Cross-definition: Models from other definitions using GlobalKeys
    /// GlobalKeys specifies both the target definition and the target model
    /// Example: GlobalKeys::DefinitionTwo(DefinitionTwoDiscriminants::Article)
    pub cross_definition: &'a [(D::GlobalKeys, CrossAccessLevel)],
}

impl<'a, D: NetabaseDefinition + GlobalDefinitionEnum> ModelPermissions<'a, D>
where
    D::Discriminant: 'static + std::fmt::Debug + PartialEq,
    D::GlobalKeys: PartialEq,
{
    /// Create a new ModelPermissions with no permissions
    pub const fn none() -> Self {
        Self {
            outbound: &[],
            inbound: &[],
            cross_definition: &[],
        }
    }

    /// Check if this model can access a target model with the given access type
    pub fn can_access_model(&self, discriminant: D::Discriminant, access: AccessType) -> bool {
        self.outbound
            .iter()
            .find(|(disc, _)| *disc == discriminant)
            .map(|(_, level)| level.allows(access))
            .unwrap_or(false)
    }

    /// Check if this model can hydrate a relation to the target model
    pub fn can_hydrate_relation(&self, discriminant: D::Discriminant) -> bool {
        self.outbound
            .iter()
            .find(|(disc, _)| *disc == discriminant)
            .map(|(_, level)| level.allows_hydrate())
            .unwrap_or(false)
    }

    /// Get the access level for a specific model discriminant
    pub fn get_access_level(&self, discriminant: D::Discriminant) -> Option<AccessLevel> {
        self.outbound
            .iter()
            .find(|(disc, _)| *disc == discriminant)
            .map(|(_, level)| *level)
    }

    /// Check if this model allows cross-definition access to a target
    pub fn can_access_cross_definition(&self, global_key: &D::GlobalKeys) -> bool {
        self.cross_definition
            .iter()
            .find(|(key, _)| key == global_key)
            .map(|(_, level)| level.allows_read())
            .unwrap_or(false)
    }

    /// Check if this model can hydrate a cross-definition relation
    pub fn can_hydrate_cross_definition(&self, global_key: &D::GlobalKeys) -> bool {
        self.cross_definition
            .iter()
            .find(|(key, _)| key == global_key)
            .map(|(_, level)| level.allows_hydrate())
            .unwrap_or(false)
    }

    /// Check if a source model (via discriminant) is allowed to access this model
    pub fn allows_inbound_access(&self, source_discriminant: D::Discriminant, access: AccessType) -> bool {
        self.inbound
            .iter()
            .find(|(disc, _)| *disc == source_discriminant)
            .map(|(_, level)| level.allows(access))
            .unwrap_or(false)
    }
}

// Debug implementation for better error messages
impl<'a, D: NetabaseDefinition + GlobalDefinitionEnum> std::fmt::Debug for ModelPermissions<'a, D>
where
    D::Discriminant: 'static + std::fmt::Debug,
    D::GlobalKeys: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelPermissions")
            .field("outbound", &self.outbound)
            .field("inbound", &self.inbound)
            .field("cross_definition", &self.cross_definition)
            .finish()
    }
}
