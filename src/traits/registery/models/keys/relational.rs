use crate::traits::registery::definition::NetabaseDefinition;
use crate::traits::registery::models::StoreKeyMarker;
use crate::traits::registery::models::keys::NetabaseModelKeys;
use crate::traits::registery::models::keys::primary::NetabaseModelPrimaryKey;
use crate::traits::registery::models::model::NetabaseModelMarker;

/// Marker trait for relational keys
/// This creates a type-safe intermediary between NetabaseModelKeys and the relational key functionality
pub trait NetabaseModelRelationalKey<
    'a,
    D: NetabaseDefinition,        // Source definition
    M: NetabaseModelMarker<D>,    // Source model  
    K: NetabaseModelKeys<D, M>,   // Source keys
>: StoreKeyMarker<D> + Clone
where
    D::Discriminant: 'static,
{
}

/// Trait for relational keys that reference foreign models
/// 
/// This trait extends the marker trait with actual functionality
pub trait NetabaseModelRelationalKeyForeign<
    'a,
    D: NetabaseDefinition,        // Source definition
    M: NetabaseModelMarker<D>,    // Source model
    K: NetabaseModelKeys<D, M>,   // Source keys
    FD: NetabaseDefinition,       // Foreign definition  
    FM: NetabaseModelMarker<FD>,  // Foreign model
    FK: NetabaseModelKeys<FD, FM>, // Foreign keys
>: NetabaseModelRelationalKey<'a, D, M, K>
where
    D::Discriminant: 'static,
    FD::Discriminant: 'static,
    FK::Secondary<'a>: StoreKeyMarker<FD>,
    FK::Relational<'a>: StoreKeyMarker<FD>,
{
    /// The foreign primary key type this relational key references
    type ForeignPrimaryKey: NetabaseModelPrimaryKey<'a, FD, FM, FK>;
    
    /// Get the foreign primary key value
    fn foreign_key(&self) -> &Self::ForeignPrimaryKey;
    
    /// Create a relational key from a foreign primary key
    fn from_foreign_key(key: Self::ForeignPrimaryKey) -> Self;
}
