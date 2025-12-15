use crate::traits::registery::{
    definition::NetabaseDefinition, 
    models::{
        model::NetabaseModel,
        keys::NetabaseModelKeys,
    },
};
use strum::IntoDiscriminant;

/// Represents a relational link between models, potentially across different database definitions.
/// This allows for cross-definition access and linking with proper type safety and permission checking.
#[derive(Debug, Clone)]
pub struct RelationalLink<FM, OD> 
where
    FM: NetabaseModel<OD>,
    OD: NetabaseDefinition,
{
    /// The actual relational data
    relation: RelationalLinkType<FM, OD>,
}

/// The type of relational link - either a pointer (foreign key) or hydrated (loaded data)
#[derive(Debug, Clone)]
pub enum RelationalLinkType<FM, OD> 
where
    FM: NetabaseModel<OD>,
    OD: NetabaseDefinition,
{
    /// A pointer relation containing only the foreign key
    Pointer(<FM::Keys as NetabaseModelKeys<OD, FM>>::Primary<'static>),
    /// A hydrated relation containing the actual loaded model data
    Hydrated(FM),
}

impl<FM, OD> RelationalLink<FM, OD> 
where
    FM: NetabaseModel<OD>,
    OD: NetabaseDefinition,
{
    /// Create a new pointer relation with just the foreign key
    pub fn new_pointer(key: <FM::Keys as NetabaseModelKeys<OD, FM>>::Primary<'static>) -> Self {
        Self {
            relation: RelationalLinkType::Pointer(key),
        }
    }

    /// Create a new hydrated relation with the loaded model
    pub fn new_hydrated(model: FM) -> Self {
        Self {
            relation: RelationalLinkType::Hydrated(model),
        }
    }

    /// Get the foreign key from the relation
    pub fn get_key(&self) -> &<FM::Keys as NetabaseModelKeys<OD, FM>>::Primary<'static>
    {
        match &self.relation {
            RelationalLinkType::Pointer(key) => key,
            RelationalLinkType::Hydrated(model) => model.get_primary_key(),
        }
    }

    /// Check if this relation is currently hydrated (contains model data)
    pub fn is_hydrated(&self) -> bool {
        matches!(self.relation, RelationalLinkType::Hydrated(_))
    }

    /// Check if this relation is a pointer (contains only foreign key)
    pub fn is_pointer(&self) -> bool {
        matches!(self.relation, RelationalLinkType::Pointer(_))
    }

    /// Get the hydrated model if available, otherwise None
    pub fn get_model(&self) -> Option<&FM> {
        match &self.relation {
            RelationalLinkType::Hydrated(model) => Some(model),
            RelationalLinkType::Pointer(_) => None,
        }
    }

    /// Convert a pointer relation to hydrated by providing the model data
    pub fn hydrate(self, model: FM) -> Self {
        Self::new_hydrated(model)
    }

    /// Convert a hydrated relation back to pointer
    pub fn dehydrate(self) -> Self 
    {
        match &self.relation {
            RelationalLinkType::Hydrated(model) => Self::new_pointer(model.get_primary_key()),
            RelationalLinkType::Pointer(key) => Self::new_pointer(key.clone()),
        }
    }

    /// Get the target definition type information
    pub fn target_definition() -> &'static str {
        std::any::type_name::<OD>()
    }

    /// Get the target model type information
    pub fn target_model() -> &'static str {
        std::any::type_name::<FM>()
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

/// Trait for models that can have relational links
pub trait RelationalModel<D: NetabaseDefinition> 
where
    Self: NetabaseModel<D>,
{
    /// Get all relational links from this model as type-erased references
    fn get_relational_links(&self) -> Vec<&dyn std::any::Any>;
    
    /// Update a relational link by replacing it with a new one
    fn update_relational_link<FM, OD>(
        &mut self, 
        link: RelationalLink<FM, OD>
    ) -> Result<(), RelationalLinkError>
    where
        FM: NetabaseModel<OD>,
        OD: NetabaseDefinition;
}

/// Cross-definition permissions for relational access
#[derive(Debug, Clone)]
pub struct CrossDefinitionPermissions<D1: NetabaseDefinition, D2: NetabaseDefinition> 
where
    D1::ModelTableDefinition: 'static,
    D2::ModelTableDefinition: 'static,
{
    /// List of accessible table definitions from the source definition
    pub source_tables: Vec<D1::ModelTableDefinition>,
    /// List of accessible table definitions from the target definition
    pub target_tables: Vec<D2::ModelTableDefinition>,
    /// Whether read access is allowed
    pub read_allowed: bool,
    /// Whether write access is allowed
    pub write_allowed: bool,
    /// Whether hydration (loading related data) is allowed
    pub hydration_allowed: bool,
}

impl<D1: NetabaseDefinition, D2: NetabaseDefinition> CrossDefinitionPermissions<D1, D2> 
where
    D1::ModelTableDefinition: 'static,
    D2::ModelTableDefinition: 'static,
{
    /// Create new cross-definition permissions
    pub fn new(
        source_tables: Vec<D1::ModelTableDefinition>,
        target_tables: Vec<D2::ModelTableDefinition>,
        read_allowed: bool, 
        write_allowed: bool, 
        hydration_allowed: bool
    ) -> Self {
        Self {
            source_tables,
            target_tables,
            read_allowed,
            write_allowed,
            hydration_allowed,
        }
    }

    /// Create read-only permissions with specified tables
    pub fn read_only(
        source_tables: Vec<D1::ModelTableDefinition>,
        target_tables: Vec<D2::ModelTableDefinition>
    ) -> Self {
        Self::new(source_tables, target_tables, true, false, true)
    }

    /// Create full permissions with specified tables
    pub fn full_access(
        source_tables: Vec<D1::ModelTableDefinition>,
        target_tables: Vec<D2::ModelTableDefinition>
    ) -> Self {
        Self::new(source_tables, target_tables, true, true, true)
    }

    /// Create no permissions
    pub fn no_access() -> Self {
        Self::new(Vec::new(), Vec::new(), false, false, false)
    }

    /// Check if a specific source table is accessible
    pub fn can_access_source_table(&self, table: &D1::ModelTableDefinition) -> bool
    where
        D1::ModelTableDefinition: PartialEq,
    {
        self.source_tables.contains(table)
    }

    /// Check if a specific target table is accessible
    pub fn can_access_target_table(&self, table: &D2::ModelTableDefinition) -> bool
    where
        D2::ModelTableDefinition: PartialEq,
    {
        self.target_tables.contains(table)
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
}