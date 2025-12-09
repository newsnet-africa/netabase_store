use crate::traits::store::tree_manager::TreeManager;
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

pub mod key;

/// Trait for converting discriminants to string names safely
/// Uses strum's AsRefStr for prefix matching support
pub trait DiscriminantName: AsRef<str> {
    fn name(&self) -> &str {
        self.as_ref()
    }
}

pub trait NetabaseDefinitionTrait: IntoDiscriminant + TreeManager<Self> + Sized
where
    <Self as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    type Keys: IntoDiscriminant;
    /// Unified enum that wraps all model-associated types under one generic type
    /// This eliminates the need for opaque Vec<u8> and String types
    type ModelAssociatedTypes: Clone + Debug + Send + ModelAssociatedTypesExt<Self> + crate::databases::redb_store::RedbModelAssociatedTypesExt<Self>;
}

/// Extension trait for ModelAssociatedTypes to provide type conversion methods
pub trait ModelAssociatedTypesExt<D>
where
    D: NetabaseDefinitionTrait + crate::traits::store::tree_manager::TreeManager<D>,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Wrap a primary key of any model into the unified type
    fn from_primary_key<M: crate::traits::model::NetabaseModelTrait<D>>(key: M::PrimaryKey) -> Self;
    
    /// Wrap a model instance into the unified type
    fn from_model<M: crate::traits::model::NetabaseModelTrait<D>>(model: M) -> Self;
    
    /// Wrap a secondary key into the unified type
    fn from_secondary_key<M: crate::traits::model::NetabaseModelTrait<D>>(
        key: <<M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant
    ) -> Self;
    
    /// Wrap a relational key discriminant into the unified type
    fn from_relational_key_discriminant<M: crate::traits::model::NetabaseModelTrait<D>>(
        key: <<M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant
    ) -> Self;
    
    /// Wrap a secondary key data into the unified type
    fn from_secondary_key_data<M: crate::traits::model::NetabaseModelTrait<D>>(
        key: <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::SecondaryEnum
    ) -> Self;
    
    /// Wrap a relational key data into the unified type
    fn from_relational_key_data<M: crate::traits::model::NetabaseModelTrait<D>>(
        key: <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::RelationalEnum
    ) -> Self;

    /// Wrap a subscription key discriminant into the unified type
    fn from_subscription_key_discriminant<M: crate::traits::model::NetabaseModelTrait<D>>(
        key: <<M as crate::traits::model::NetabaseModelTrait<D>>::SubscriptionEnum as IntoDiscriminant>::Discriminant
    ) -> Self;
}

pub trait NetabaseDefinition: NetabaseDefinitionTrait + TreeManager<Self> + Sized
where
    <Self as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
}

impl<T: NetabaseDefinitionTrait + TreeManager<T>> NetabaseDefinition for T where
    <T as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone
{
}
