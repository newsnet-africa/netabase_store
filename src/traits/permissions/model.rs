use super::traits::AccessType;
use crate::traits::registery::{
    definition::NetabaseDefinition, models::treenames::DiscriminantTableName,
};

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

/// Model-level permissions using ModelTreeNames for type safety
/// This structure is const-compatible and follows the same pattern as TREE_NAMES
///
/// Permission hierarchy:
/// - `outbound`: Which models this model can access within the same definition
///   Uses D::TreeNames (DefinitionTreeNames enum) which holds ModelTreeNames for each model
///   This allows recursive traversal of relational tables
pub struct ModelPermissions<'a, D: NetabaseDefinition>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
    /// Outbound: Which models this model can access within the same definition
    /// Stores (DefinitionTreeNames, AccessLevel) pairs
    /// DefinitionTreeNames is an enum holding ModelTreeNames for each model in the definition
    pub outbound: &'a [(D::TreeNames, AccessLevel)],
}

impl<'a, D: NetabaseDefinition> ModelPermissions<'a, D>
where
    D::Discriminant: 'static + std::fmt::Debug + PartialEq,
    D::TreeNames: PartialEq,
{
    /// Create a new ModelPermissions with no permissions
    pub const fn none() -> Self {
        Self { outbound: &[] }
    }

    /// Check if this model can access a target model (by TreeNames) with the given access type
    pub fn can_access_model(
        &self,
        tree_names: &DiscriminantTableName<D>,
        access: AccessType,
    ) -> bool {
        self.outbound
            .iter()
            .find(|(names, acc)| names.table_name == tree_names.table_name)
            .map(|(_, level)| level.allows(access))
            .unwrap_or(false)
    }

    /// Check if this model can hydrate a relation to the target model
    pub fn can_hydrate_relation(&self, tree_names: &DiscriminantTableName<D>) -> bool {
        self.outbound
            .iter()
            .find(|(names, acc)| names.table_name == tree_names.table_name)
            .map(|(_, level)| level.allows_hydrate())
            .unwrap_or(false)
    }

    /// Get the access level for a specific model by TreeNames
    pub fn get_access_level(&self, tree_names: &DiscriminantTableName<D>) -> Option<AccessLevel> {
        self.outbound
            .iter()
            .find(|(names, acc)| names.table_name == tree_names.table_name)
            .map(|(_, level)| *level)
    }

    /// Get all accessible model tree names
    pub fn get_accessible_models(&self) -> impl Iterator<Item = &DiscriminantTableName<D>> {
        self.outbound.iter().map(|(names, _)| names)
    }
}

// Debug implementation for better error messages
impl<'a, D: NetabaseDefinition> std::fmt::Debug for ModelPermissions<'a, D>
where
    D::Discriminant: 'static + std::fmt::Debug,
    D::TreeNames: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelPermissions").finish()
    }
}
