use js_sys::Iter;
use strum::IntoEnumIterator;

use crate::{
    error::StoreError,
    traits::{
        definition::{NetabaseDefinition, NetabaseDefinitionDiscriminants},
        model::{NetabaseModel, NetabaseModelKey},
    },
};

pub trait Store<D: NetabaseDefinition>: Iterator<Item = D> {
    type StoreError: std::error::Error;
    type Tree: StoreTree;

    fn get_definitions(&self) -> <D::Discriminants as IntoEnumIterator>::Iterator {
        D::Discriminants::iter()
    }

    fn open_tree<V: NetabaseModel<Defined = D>>(
        &self,
        tree_type: <<V as NetabaseModel>::Defined as NetabaseDefinition>::Discriminants,
    ) -> Result<Self::Tree, StoreError>
    where
        Self::Tree: StoreTree<Model = V>;

    fn get<V: NetabaseModel<Defined = D>>(&self, key: V::Key) -> Result<Option<V>, StoreError>
    where
        Self::Tree: StoreTree<Model = V>,
    {
        let tree = self.open_tree::<V>(V::DISCRIMINANT)?;
        tree.get(key)
    }
    fn put<V: NetabaseModel<Defined = D>>(&self, value: V) -> Result<Option<V>, StoreError>
    where
        Self::Tree: StoreTree<Model = V>,
    {
        let tree = self.open_tree::<V>(V::DISCRIMINANT)?;
        tree.put(value)
    }
}

pub trait StoreTree {
    type Model: NetabaseModel;

    fn get(
        &self,
        key: <Self::Model as NetabaseModel>::Key,
    ) -> Result<Option<Self::Model>, StoreError>;

    fn put(&self, value: Self::Model) -> Result<Option<Self::Model>, StoreError>;
}
