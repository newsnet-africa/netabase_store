use std::{marker::PhantomData, path::Path};

use strum::IntoEnumIterator;

use crate::{
    error::{NetabaseError, StoreError},
    traits::{
        definition::NetabaseDefinition,
        store::{Store, StoreTree},
    },
};

pub struct SledStore<D: NetabaseDefinition> {
    db: sled::Db,
    definitions: Vec<D::Discriminants>,
}

impl<D: NetabaseDefinition> SledStore<D> {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        Ok(Self {
            db: sled::open(path)?,
            definitions: <<D as NetabaseDefinition>::Discriminants as IntoEnumIterator>::iter()
                .collect(),
        })
    }
}

impl<D: NetabaseDefinition> Store<D> for SledStore<D> {
    type StoreError = sled::Error;

    type Tree = sled::Tree;

    fn open_tree<V: crate::traits::model::NetabaseModel<Defined = D>>(
        &self,
        tree_type: <<V as crate::traits::model::NetabaseModel>::Defined as NetabaseDefinition>::Discriminants,
    ) -> Result<Self::Tree, StoreError>
    where
        Self::Tree: StoreTree,
    {
        todo!()
    }

    fn iter(&self) -> impl Iterator<Item = D> {
        self.definitions.iter().filter_map(|d| {
            let tree = match self.open_tree(d) {
                Ok(t) => t.iter().values().map(f),
                Err(_) => return None,
            };
        })
    }
}

impl StoreTree for sled::Tree {
    fn insert<M: crate::traits::model::NetabaseModel>(
        &self,
        key: <M as crate::traits::model::NetabaseModel>::Key,
    ) -> Result<Option<M>, StoreError> {
        todo!()
    }

    fn insert<M: crate::traits::model::NetabaseModel>(
        &self,
        value: M,
    ) -> Result<Option<M>, StoreError> {
        todo!()
    }
}

impl<D: NetabaseDefinition> Iterator for SledStore<D> {
    type Item = D;

    fn next(&mut self) -> Option<Self::Item> {}
}
