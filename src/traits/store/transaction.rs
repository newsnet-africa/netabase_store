use crate::traits::definition::{NetabaseDefinition, DiscriminantName};
use crate::traits::model::{NetabaseModelTrait, RedbNetabaseModelTrait, key::NetabaseModelKeyTrait};
use crate::error::NetabaseResult;
use strum::{IntoDiscriminant, IntoEnumIterator};
use redb::{Key, Value};
use std::borrow::Borrow;
use std::fmt::Debug;

pub trait ReadTransaction<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant: IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName,
{
    fn get<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(&self, key: M::PrimaryKey) -> NetabaseResult<Option<M>>
    where 
        M::PrimaryKey: Key + 'static,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>;
}

pub trait WriteTransaction<D: NetabaseDefinition>: ReadTransaction<D>
where
    <D as IntoDiscriminant>::Discriminant: IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName,
{
    fn put<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + Send + Clone>(&mut self, model: M) -> NetabaseResult<()>
    where 
        M::PrimaryKey: Key + 'static + Send + Clone,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Borrow<<M as Value>::SelfType<'a>>,
        M::SecondaryKeys: Iterator<Item = <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: IntoDiscriminant + Clone + Debug + Send + Key,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: Debug,
        M::RelationalKeys: Iterator<Item = <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: IntoDiscriminant + Clone + Debug + Send + Key,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: Debug;
    
    fn delete<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + Clone + Send>(&mut self, key: M::PrimaryKey) -> NetabaseResult<()>
    where 
        M::PrimaryKey: Key + 'static + Send + Clone,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Borrow<<M as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: IntoDiscriminant + Clone + Debug + Send + Key,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: IntoDiscriminant + Clone + Debug + Send + Key;

    fn commit(self) -> NetabaseResult<()>;
}