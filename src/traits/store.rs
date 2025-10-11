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
        tree.insert(value)
    }
}

pub trait StoreTree {
    fn get<M: NetabaseModel>(
        &self,
        key: <M as NetabaseModel>::Key,
    ) -> Result<Option<M>, StoreError>;

    fn insert<M: NetabaseModel>(&self, value: M) -> Result<Option<M>, StoreError>;
}
