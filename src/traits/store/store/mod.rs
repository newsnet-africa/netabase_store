use crate::traits::definition::{DiscriminantName, NetabaseDefinition};
use crate::databases::redb_store::RedbNetabaseModelTrait;
use crate::traits::model::key::NetabaseModelKeyTrait;
use crate::traits::store::transaction::{ReadTransaction, WriteTransaction};
use crate::{error::NetabaseResult, traits::model::NetabaseModelTrait};
use redb::{Key, Value};
use std::borrow::Borrow;
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

pub trait StoreTrait<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant: IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    type ReadTxn<'a>: ReadTransaction<D> where Self: 'a;
    type WriteTxn: WriteTransaction<D>;

    fn read<'a, F, R>(&'a self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::ReadTxn<'a>) -> NetabaseResult<R>;

    fn write<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&mut Self::WriteTxn) -> NetabaseResult<R>;

    fn get_one<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<Option<M>>
    where
        M::PrimaryKey: Key + 'static,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
        [u8; 32]: From<<M as NetabaseModelTrait<D>>::Hash>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>,
    {
        self.read(|txn| txn.get::<M>(key))
    }

    fn put_one<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + Send + Clone>(
        &self,
        model: M,
    ) -> NetabaseResult<()>
    where
        M::PrimaryKey: Key + 'static + Send + Clone + TryInto<Vec<u8>>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>,
        Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum>,
        Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Borrow<<M as Value>::SelfType<'a>>,
        M::Hash: Clone + Into<[u8; 32]>,
        M::SecondaryKeys: Iterator<Item = <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::SecondaryEnum>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: IntoEnumIterator + bincode::Encode,
        <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::SecondaryEnum: IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>>,
        <<M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: Debug,
        M::RelationalKeys: Iterator<Item = <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::RelationalEnum>,
        <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::RelationalEnum: IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>>,
        <<M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: Debug + IntoEnumIterator + bincode::Encode,
        [u8; 32]: From<<M as NetabaseModelTrait<D>>::Hash>,
    {
        self.write(|txn| txn.put(model))
    }

    fn get_many<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        keys: Vec<M::PrimaryKey>,
    ) -> NetabaseResult<Vec<Option<M>>>
    where
        M::PrimaryKey: Key + 'static,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>,
        [u8; 32]: From<<M as NetabaseModelTrait<D>>::Hash>,
    {
        self.read(|txn| {
            let mut results = Vec::with_capacity(keys.len());
            for key in keys {
                results.push(txn.get::<M>(key)?);
            }
            Ok(results)
        })
    }

    fn put_many<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + Send + Clone>(
        &self,
        models: Vec<M>,
    ) -> NetabaseResult<()>
    where
        M::PrimaryKey: Key + 'static + Send + Clone + TryInto<Vec<u8>>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>,
        Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum>,
        Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Borrow<<M as Value>::SelfType<'a>>,
        M::Hash: Clone + Into<[u8; 32]>,
        M::SecondaryKeys: Iterator<Item = <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::SecondaryEnum>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: IntoEnumIterator + bincode::Encode,
        <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::SecondaryEnum: IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>>,
        <<M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: Debug,
        M::RelationalKeys: Iterator<Item = <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::RelationalEnum>,
        <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::RelationalEnum: IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>>,
        <<M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: Debug + IntoEnumIterator + bincode::Encode,
        [u8; 32]: From<<M as NetabaseModelTrait<D>>::Hash>,
    {
        self.write(|txn| {
            for model in models {
                txn.put(model)?;
            }
            Ok(())
        })
    }
}
