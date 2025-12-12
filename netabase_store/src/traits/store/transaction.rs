use crate::databases::redb_store::RedbNetabaseModelTrait;
use crate::error::NetabaseResult;
use crate::traits::definition::{DiscriminantName, NetabaseDefinition};
use crate::traits::model::{NetabaseModelTrait, ModelTypeContainer, key::NetabaseModelKeyTrait};
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
        Vec<u8>: TryFrom<
            <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey,
        >,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName;

    /// Get a model's primary key by its secondary key
    fn get_pk_by_secondary_key<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: Key + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: for<'a> Borrow<
            <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as Value>::SelfType<'a>,
        >,
        M: for<'a> Value<SelfType<'a> = M>,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName;

    /// Get a model by its secondary key
    fn get_by_secondary_key<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: Key + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: for<'a> Borrow<
            <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as Value>::SelfType<'a>,
        >,
        M: for<'a> Value<SelfType<'a> = M>,
        Vec<u8>: TryFrom<
            <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey,
        >,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName,
    {
        if let Some(pk) = self.get_pk_by_secondary_key::<M>(secondary_key)? {
            self.get::<M>(pk)
        } else {
            Ok(None)
        }
    }

    /// Get a model's primary key by its relational key
    fn get_pk_by_relational_key<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        relational_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: Key + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: for<'a> Borrow<
            <<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as Value>::SelfType<'a>,
        >,
        M: for<'a> Value<SelfType<'a> = M>,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum:
            strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName;

    /// Get a model by its relational key
    fn get_by_relational_key<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        relational_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum,
    ) -> NetabaseResult<Option<M>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: Key + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: for<'a> Borrow<
            <<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as Value>::SelfType<'a>,
        >,
        M: for<'a> Value<SelfType<'a> = M>,
        Vec<u8>: TryFrom<
            <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey,
        >,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum:
            strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName,
    {
        if let Some(pk) = self.get_pk_by_relational_key::<M>(relational_key)? {
            self.get::<M>(pk)
        } else {
            Ok(None)
        }
    }

    /// Get a model's primary key by its hash
    fn get_pk_by_hash<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        hash: M::Hash,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M::Hash: Key + 'static,
        M::Hash: for<'a> Borrow<<M::Hash as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName;

    /// Get a model by its hash
    fn get_by_hash<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        hash: M::Hash,
    ) -> NetabaseResult<Option<M>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M::Hash: Key + 'static,
        M::Hash: for<'a> Borrow<<M::Hash as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
        Vec<u8>: TryFrom<
            <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey,
        >,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName,
    {
        if let Some(pk) = self.get_pk_by_hash::<M>(hash)? {
            self.get::<M>(pk)
        } else {
            Ok(None)
        }
    }

    /// Get the order-independent hash accumulator for a subscription tree
    fn get_subscription_accumulator<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <M as ModelTypeContainer>::Subscriptions,
    ) -> NetabaseResult<([u8; 32], u64)>
    where
        M::PrimaryKey: Key + 'static,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName;

    /// Get all primary keys in a subscription tree
    fn get_subscription_keys<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <M as ModelTypeContainer>::Subscriptions,
    ) -> NetabaseResult<Vec<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName;
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
        M::Hash: Clone + Into<[u8; 32]>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: IntoEnumIterator + bincode::Encode,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>> + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: Debug,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>> + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: Debug + bincode::Encode,
        <D as NetabaseDefinition>::Keys: IntoDiscriminant,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: strum::IntoEnumIterator,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName;

    fn delete<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + Clone + Send>(
        &mut self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<()>
    where
        M::PrimaryKey: Key + 'static + Send + Clone + TryInto<Vec<u8>>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            IntoDiscriminant + Clone + Debug + Send + Key + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>> + TryInto<Vec<u8>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum:
            IntoDiscriminant + Clone + Debug + Send + Key + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>> + TryInto<Vec<u8>>,
        <D as NetabaseDefinition>::Keys: IntoDiscriminant,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            strum::IntoEnumIterator,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum:
            strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName,
        <M as ModelTypeContainer>::Subscriptions: DiscriminantName;

    fn commit(self) -> NetabaseResult<()>;
}
