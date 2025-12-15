use crate::traits::registery::{
    definition::NetabaseDefinition, 
};

/// Trait for types that can be converted to/from a global definition enum
pub trait IntoGlobalDefinition {
    type GlobalEnum: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug;
    
    fn into_global(self) -> Self::GlobalEnum;
    fn from_global(global: Self::GlobalEnum) -> Option<Self> where Self: Sized;
}

/// Trait for managing global definition collections
pub trait GlobalDefinitionCollection {
    type DefinitionType;
    type GlobalEnum;
    
    fn add_definition(&mut self, def: Self::DefinitionType);
    fn get_definition(&self, global: &Self::GlobalEnum) -> Option<&Self::DefinitionType>;
    fn remove_definition(&mut self, global: &Self::GlobalEnum) -> Option<Self::DefinitionType>;
}

/// Trait that enables any definition to be part of a global enum system
/// This should be implemented by macro for all NetabaseDefinition types
pub trait GlobalDefinitionEnum: NetabaseDefinition 
where
    <Self as strum::IntoDiscriminant>::Discriminant: 'static,
{
    type GlobalDefinition: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug;
    type GlobalKeys: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug;
    
    fn into_global_definition(self) -> Self::GlobalDefinition;
    fn from_global_definition(global: Self::GlobalDefinition) -> Option<Self> where Self: Sized;
    
    fn discriminant_into_global(discriminant: Self::Discriminant) -> Self::GlobalKeys;
    fn discriminant_from_global(global: Self::GlobalKeys) -> Option<Self::Discriminant>;
}

/// A relational link to a foreign key in another definition
/// Uses generics instead of global enums for type safety
#[derive(Debug, Clone)]
pub struct RelationalLink<D: GlobalDefinitionEnum> 
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// The foreign key pointing to another definition
    pub foreign_key: D::GlobalKeys,
    /// Whether the relation is a pointer or hydrated
    pub relation_type: RelationType<D>,
}

/// The type of relation - either a pointer or hydrated data
#[derive(Debug, Clone)]
pub enum RelationType<D: GlobalDefinitionEnum> 
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Just the key pointer to the foreign data
    Pointer,
    /// Hydrated with the actual foreign data
    Hydrated(D::GlobalDefinition),
}

// Manual PartialEq implementation to avoid derive issues with lifetime bounds
impl<D: GlobalDefinitionEnum> PartialEq for RelationalLink<D> 
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn eq(&self, other: &Self) -> bool {
        // For now, just compare discriminants of foreign keys
        // TODO: Implement proper comparison when we have conversion traits
        std::mem::discriminant(&self.foreign_key) == std::mem::discriminant(&other.foreign_key)
    }
}

impl<D: GlobalDefinitionEnum> PartialEq for RelationType<D> 
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RelationType::Pointer, RelationType::Pointer) => true,
            (RelationType::Hydrated(_), RelationType::Hydrated(_)) => {
                // For now just compare discriminants
                // TODO: Implement proper comparison when we have the trait methods
                true
            }
            _ => false,
        }
    }
}

impl<D: GlobalDefinitionEnum> RelationalLink<D> 
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Create a new pointer relation with just the foreign key
    pub fn new_pointer(key: D::GlobalKeys) -> Self {
        Self {
            foreign_key: key,
            relation_type: RelationType::Pointer,
        }
    }

    /// Create a new hydrated relation with the loaded model
    pub fn new_hydrated(model: D::GlobalDefinition, key: D::GlobalKeys) -> Self {
        Self {
            foreign_key: key,
            relation_type: RelationType::Hydrated(model),
        }
    }

    /// Get the foreign key from the relation
    pub fn get_key(&self) -> &D::GlobalKeys {
        &self.foreign_key
    }

    /// Check if this relation is currently hydrated (contains model data)
    pub fn is_hydrated(&self) -> bool {
        matches!(self.relation_type, RelationType::Hydrated(_))
    }

    /// Check if this relation is a pointer (contains only foreign key)
    pub fn is_pointer(&self) -> bool {
        matches!(self.relation_type, RelationType::Pointer)
    }

    /// Get the hydrated model if available, otherwise None
    pub fn get_model(&self) -> Option<&D::GlobalDefinition> {
        match &self.relation_type {
            RelationType::Hydrated(model) => Some(model),
            RelationType::Pointer => None,
        }
    }

    /// Convert a pointer relation to hydrated by providing the model data
    pub fn hydrate(mut self, model: D::GlobalDefinition) -> Self {
        self.relation_type = RelationType::Hydrated(model);
        self
    }

    /// Convert a hydrated relation back to pointer
    pub fn dehydrate(mut self) -> Self {
        self.relation_type = RelationType::Pointer;
        self
    }
}

/// Errors that can occur during relational operations
#[derive(Debug, thiserror::Error)]
pub enum RelationalLinkError {
    #[error("Key mismatch: the provided model's primary key doesn't match the stored foreign key")]
    KeyMismatch,
    
    #[error("Permission denied: insufficient permissions to access related definition")]
    PermissionDenied,
    
    #[error("Not found: the related model could not be found")]
    NotFound,
    
    #[error("Cross-definition access error")]
    CrossDefinitionError,
}

/// Cross-definition permissions for relational access
/// Uses strongly typed table definitions instead of strings
#[derive(Debug, Clone)]
pub struct CrossDefinitionPermissions<D: GlobalDefinitionEnum> 
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// List of accessible table definitions (strongly typed)
    pub accessible_tables: Vec<D::GlobalKeys>,
    /// Whether read access is allowed
    pub read_allowed: bool,
    /// Whether write access is allowed
    pub write_allowed: bool,
    /// Whether hydration (loading related data) is allowed
    pub hydration_allowed: bool,
}

impl<D: GlobalDefinitionEnum> CrossDefinitionPermissions<D> {
    /// Create new cross-definition permissions
    pub fn new(
        accessible_tables: Vec<D::GlobalKeys>,
        read_allowed: bool, 
        write_allowed: bool, 
        hydration_allowed: bool
    ) -> Self {
        Self {
            accessible_tables,
            read_allowed,
            write_allowed,
            hydration_allowed,
        }
    }

    /// Create read-only permissions with specified tables
    pub fn read_only(accessible_tables: Vec<D::GlobalKeys>) -> Self {
        Self::new(accessible_tables, true, false, true)
    }

    /// Create full permissions with specified tables
    pub fn full_access(accessible_tables: Vec<D::GlobalKeys>) -> Self {
        Self::new(accessible_tables, true, true, true)
    }

    /// Create no permissions
    pub fn no_access() -> Self 
    where 
        <D as strum::IntoDiscriminant>::Discriminant: 'static,
    {
        Self::new(Vec::new(), false, false, false)
    }

    /// Check if the given operation is allowed
    pub fn can_read(&self) -> bool {
        self.read_allowed
    }

    pub fn can_write(&self) -> bool {
        self.write_allowed
    }

    pub fn can_hydrate(&self) -> bool {
        self.hydration_allowed
    }

    /// Check if a specific table key is accessible
    pub fn can_access_table(&self, key: &D::GlobalKeys) -> bool {
        self.accessible_tables.contains(key)
    }
}