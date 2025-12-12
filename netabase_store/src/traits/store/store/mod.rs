use crate::traits::definition::{DiscriminantName, NetabaseDefinition};
use crate::databases::redb_store::RedbNetabaseModelTrait;
use crate::databases::sled_store::SledNetabaseModelTrait;
use crate::traits::model::key::NetabaseModelKeyTrait;
use crate::traits::store::transaction::{ReadTransaction, WriteTransaction};
use crate::{error::NetabaseResult, traits::model::NetabaseModelTrait, traits::model::ModelTypeContainer};
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

    fn get_one<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<Option<M>>
    where
        M::PrimaryKey: Key + 'static + bincode::Encode + bincode::Decode<()>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M> + bincode::Decode<()>,
        [u8; 32]: From<<M as NetabaseModelTrait<D>>::Hash>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName,
    {
        self.read(|txn| txn.get::<M>(key))
    }

    fn put_one<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + SledNetabaseModelTrait<D> + Send + Clone>(
        &self,
        model: M,
    ) -> NetabaseResult<()>
    where
        M::PrimaryKey: Key + 'static + Send + Clone + TryInto<Vec<u8>> + bincode::Encode + bincode::Decode<()>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>,
        Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum>,
        Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Borrow<<M as Value>::SelfType<'a>> + bincode::Encode + bincode::Decode<()>, // Added bincode::Decode<()>
        M::Hash: Clone + Into<[u8; 32]>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: IntoEnumIterator + bincode::Encode + bincode::Decode<()>, // Added bincode::Decode<()>
        <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::SecondaryEnum: IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>> + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <<M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: Debug,
        <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::RelationalEnum: IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>> + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <<M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: Debug + IntoEnumIterator + bincode::Encode + bincode::Decode<()>, // Added bincode::Decode<()>
        [u8; 32]: From<<M as NetabaseModelTrait<D>>::Hash>,
        <D as NetabaseDefinition>::Keys: IntoDiscriminant,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: strum::IntoEnumIterator,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName,
    {
        self.write(|txn| txn.put(model))
    }

    fn get_many<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        keys: Vec<M::PrimaryKey>,
    ) -> NetabaseResult<Vec<Option<M>>>
    where
        M::PrimaryKey: Key + 'static + bincode::Encode + bincode::Decode<()>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M> + bincode::Decode<()>,
        [u8; 32]: From<<M as NetabaseModelTrait<D>>::Hash>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName,
    {
        self.read(|txn| {
            let mut results = Vec::with_capacity(keys.len());
            for key in keys {
                results.push(txn.get::<M>(key)?);
            }
            Ok(results)
        })
    }

    fn put_many<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + SledNetabaseModelTrait<D> + Send + Clone>(
        &self,
        models: Vec<M>,
    ) -> NetabaseResult<()>
    where
        M::PrimaryKey: Key + 'static + Send + Clone + TryInto<Vec<u8>> + bincode::Encode + bincode::Decode<()>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>,
        Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum>,
        Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Borrow<<M as Value>::SelfType<'a>> + bincode::Encode + bincode::Decode<()>, // Added bincode::Decode<()>
        M::Hash: Clone + Into<[u8; 32]>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: IntoEnumIterator + bincode::Encode + bincode::Decode<()>, // Added bincode::Decode<()>
        <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::SecondaryEnum: IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>> + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <<M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: Debug,
        <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::RelationalEnum: IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>> + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <<M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: Debug + IntoEnumIterator + bincode::Encode + bincode::Decode<()>, // Added bincode::Decode<()>
        [u8; 32]: From<<M as NetabaseModelTrait<D>>::Hash>,
        <D as NetabaseDefinition>::Keys: IntoDiscriminant,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: strum::IntoEnumIterator,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + bincode::Encode + bincode::Decode<()> + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName,
    {
        self.write(|txn| {
            for model in models {
                txn.put(model)?;
            }
            Ok(())
        })
    }
}