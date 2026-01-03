use crate::traits::registery::definition::NetabaseDefinition;
use crate::traits::registery::models::StoreKeyMarker;
use crate::traits::registery::models::keys::primary::NetabaseModelPrimaryKey;
use crate::traits::registery::models::model::NetabaseModelMarker;

/// Marker trait for relational keys.
/// 
/// This is a simple marker trait without the K parameter to avoid
/// early/late-bound lifetime issues with GATs.
pub trait NetabaseModelRelationalKey<
    D: NetabaseDefinition,        // Source definition
    M: NetabaseModelMarker<D>,    // Source model  
>: StoreKeyMarker<D> + Clone
where
    D::Discriminant: 'static + std::fmt::Debug,
{
}

/// Trait for relational keys that reference foreign models
/// 
/// This trait extends the marker trait with actual functionality
pub trait NetabaseModelRelationalKeyForeign<
    D: NetabaseDefinition,        // Source definition
    M: NetabaseModelMarker<D>,    // Source model
    FD: NetabaseDefinition,       // Foreign definition  
    FM: NetabaseModelMarker<FD>,  // Foreign model
>: NetabaseModelRelationalKey<D, M>
where
    D::Discriminant: 'static + std::fmt::Debug,
    FD::Discriminant: 'static + std::fmt::Debug,
{
    /// The foreign primary key type this relational key references
    type ForeignPrimaryKey: NetabaseModelPrimaryKey<FD, FM>;
    
    /// Get the foreign primary key value
    fn foreign_key(&self) -> &Self::ForeignPrimaryKey;
    
    /// Create a relational key from a foreign primary key
    fn from_foreign_key(key: Self::ForeignPrimaryKey) -> Self;
}
