use crate::error::NetabaseResult;
use crate::traits::definition::{DiscriminantName, NetabaseDefinition};
use crate::traits::model::{NetabaseModelTrait, key::NetabaseModelKeyTrait};
use crate::databases::redb_store::RedbNetabaseModelTrait;
use redb::{Key, Value};
use std::borrow::Borrow;
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

pub trait ReadTransaction<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Get a model by its primary key
    fn get<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<Option<M>>
    where
        M::PrimaryKey: Key + 'static,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>;

    /// Get a model's primary key by its secondary key
    /// Returns the primary key which can then be used to fetch the full model
    fn get_pk_by_secondary_key<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: Key + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: for<'a> Borrow<<<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>;

    /// Get a model by its secondary key (convenience method that combines lookup and fetch)
    fn get_by_secondary_key<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: Key + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: for<'a> Borrow<<<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>,
    {
        if let Some(pk) = self.get_pk_by_secondary_key::<M>(secondary_key)? {
            self.get::<M>(pk)
        } else {
            Ok(None)
        }
    }

    /// Get the order-independent hash accumulator for a subscription tree
    /// Uses XOR to accumulate all hashes, making it independent of insertion order
    /// Returns the accumulated hash and the count of items
    fn get_subscription_accumulator<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <<M as NetabaseModelTrait<D>>::SubscriptionEnum as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<([u8; 32], u64)>
    where
        M::PrimaryKey: Key + 'static;

    /// Get all primary keys in a subscription tree
    /// Useful for syncing differences between stores
    fn get_subscription_keys<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <<M as NetabaseModelTrait<D>>::SubscriptionEnum as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<Vec<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>;
}

pub trait WriteTransaction<D: NetabaseDefinition>: ReadTransaction<D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    fn put<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + Send + Clone>(&mut self, model: M) -> NetabaseResult<()>
    where
        M::PrimaryKey: Key + 'static + Send + Clone + TryInto<Vec<u8>>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>,
        Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum>,
        Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Borrow<<M as Value>::SelfType<'a>>,
        M::Hash: Clone + Into<[u8; 32]>, // Add the Hash constraint to trait too
        M::SecondaryKeys: Iterator<Item = <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: IntoEnumIterator + bincode::Encode,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: Debug,
        M::RelationalKeys: Iterator<Item = <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: Debug + bincode::Encode;

    fn delete<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + Clone + Send>(
        &mut self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<()>
    where
        M::PrimaryKey: Key + 'static + Send + Clone + TryInto<Vec<u8>>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Borrow<<M as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            IntoDiscriminant + Clone + Debug + Send + Key,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum:
            IntoDiscriminant + Clone + Debug + Send + Key;

    fn commit(self) -> NetabaseResult<()>;
}
