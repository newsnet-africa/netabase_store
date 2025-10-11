use std::{marker::PhantomData, path::Path};

use strum::IntoEnumIterator;

use crate::{
    error::{NetabaseError, StoreError},
    traits::{definition::NetabaseDefinition, store::Store},
};

pub struct SledStore<D: NetabaseDefinition> {
    db: sled::Db,
    definitions: Vec<D::Discriminants>,
}

impl<D: NetabaseDefinition> SledStore<D> {
    pub fn new<P: AsRef<Path>>(path: P, definitions: D) -> Result<Self, NetabaseError> {
        Ok(Self {
            db: sled::open(path)?,
            definitions: <<D as NetabaseDefinition>::Discriminants as IntoEnumIterator>::iter(),
        })
    }
}

impl<D: NetabaseDefinition> Store<D> for SledStore<D> {
    type StoreError = NetabaseError;

    type Tree;

    fn open_tree<V: crate::traits::model::NetabaseModel<Defined = D>>(
        &self,
        tree_type: <<V as crate::traits::model::NetabaseModel>::Defined as crate::traits::definition::NetabaseDefinition>::Discriminants,
    ) -> Result<Self::Tree, StoreError>
    where
        Self::Tree: crate::traits::store::StoreTree<Model = V>,
    {
        todo!()
    }
}
