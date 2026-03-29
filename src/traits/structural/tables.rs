use crate::traits::behavioural::conversion::IntoDiscriminant;
use crate::traits::structural::definition::NetabaseDefinition;
use crate::traits::structural::models::NetabaseModel;
use crate::traits::structural::keys::{
    NetabaseKey, NetabaseModelPrimaryKey, NetabaseModelSecondaryKey, NetabaseModelBlobKey, NetabaseModelSubscriptionKey, ModelRelationalKey 
};
use crate::traits::structural::values::{StoreValue, BlobValue, ModelHash};
use std::fmt::Debug;

pub trait NetabaseTableName: Sized + Debug + Copy + Eq + std::hash::Hash {
    type Id: Debug + Copy + Eq + std::hash::Hash;
    type Inner: AsRef<str>;
    fn as_str(&self) -> &str {
        self.inner().as_ref()
    }
    fn inner(&self) -> &Self::Inner;
    fn id(&self) -> Self::Id;
}

pub trait NetabaseModelTableNames<D: NetabaseDefinition, M: NetabaseModel<D>>: Sized + Clone + NetabaseTableName {
    fn primary() -> <M::Tables as NetabaseModelTables<D, M>>::Primary;
    fn secondary<N: Into<<<M::Tables as NetabaseModelTables<D, M>>::Secondary as NetabaseModelShardTable<D,M>>::TableName>>(id: N) -> <M::Tables as NetabaseModelTables<D, M>>::Secondary;
    fn blob<N: Into<<<M::Tables as NetabaseModelTables<D, M>>::Blob as NetabaseModelShardTable<D,M>>::TableName>>(id: N) -> <M::Tables as NetabaseModelTables<D, M>>::Blob;
    fn relational<N: Into<<<M::Tables as NetabaseModelTables<D, M>>::Relational as NetabaseModelShardTable<D,M>>::TableName>>(id: N) -> <M::Tables as NetabaseModelTables<D, M>>::Relational;
    fn subscription<N: Into<<<M::Tables as NetabaseModelTables<D, M>>::Subscription as NetabaseModelShardTable<D,M>>::TableName>>(id: N) -> <M::Tables as NetabaseModelTables<D, M>>::Subscription;
}

pub trait NetabaseModelShardTable<D: NetabaseDefinition, M: NetabaseModel<D>> {
    type TableName = M::ModelName;
    type ModelTableKeys: NetabaseKey<D, M, Self::ModelTableValue>;
    type ModelTableValue: StoreValue;
}

pub trait NetabaseModelTables<D: NetabaseDefinition, M: NetabaseModel<D>> 
where
    M: StoreValue,
    Self: IntoDiscriminant,
    <<Self as NetabaseModelTables<D, M>>::Primary as NetabaseModelShardTable<D, M>>::ModelTableKeys: NetabaseModelPrimaryKey<D, M>,
    <<Self as NetabaseModelTables<D, M>>::Secondary as NetabaseModelShardTable<D, M>>::ModelTableKeys: NetabaseModelSecondaryKey<D, M>,
    <<Self as NetabaseModelTables<D, M>>::Blob as NetabaseModelShardTable<D, M>>::ModelTableKeys: NetabaseModelBlobKey<D, M>,
    <<Self as NetabaseModelTables<D, M>>::Relational as NetabaseModelShardTable<D, M>>::ModelTableKeys: NetabaseKey<D, M, <<Self as NetabaseModelTables<D, M>>::Relational as NetabaseModelShardTable<D, M>>::ModelTableValue> + ModelRelationalKey<D, M>,
    <<Self as NetabaseModelTables<D, M>>::Subscription as NetabaseModelShardTable<D, M>>::ModelTableKeys: NetabaseModelSubscriptionKey<D, M>,
{
    type Primary: ModelPrimaryTable<D, M>;
    type Secondary: ModelSecondaryTable<D, M>;
    type Blob: ModelBlobTable<D, M>;
    type Relational: ModelRelationalTable<D, M>;
    type Subscription: ModelSubscriptionTable<D, M>;
}

pub trait ModelPrimaryTable<D: NetabaseDefinition, M: NetabaseModel<D>>:
    NetabaseModelShardTable<D, M, ModelTableValue = M>
where
    Self::ModelTableKeys: NetabaseModelPrimaryKey<D, M>,
{
}

pub trait ModelSecondaryTable<D: NetabaseDefinition, M: NetabaseModel<D>>:
    NetabaseModelShardTable<D, M, ModelTableValue = <M::Keys as crate::traits::structural::keys::NetabaseModelKeys<D, M>>::Primary>
where
    Self::ModelTableKeys: NetabaseModelSecondaryKey<D, M>,
{
}

pub trait ModelBlobTable<D: NetabaseDefinition, M: NetabaseModel<D>>:
    NetabaseModelShardTable<D, M, ModelTableValue = BlobValue>
where
    <Self as NetabaseModelShardTable<D, M>>::ModelTableKeys: NetabaseModelBlobKey<D, M>,
{
}

pub trait ModelRelationalTable<D: NetabaseDefinition, M: NetabaseModel<D>>:
    NetabaseModelShardTable<D, M>
where
    <Self as NetabaseModelShardTable<D, M>>::ModelTableKeys: NetabaseKey<D, M, <Self as NetabaseModelShardTable<D, M>>::ModelTableValue> + ModelRelationalKey<D, M>,
{
}

pub trait ModelSubscriptionTable<D: NetabaseDefinition, M: NetabaseModel<D>>:
    NetabaseModelShardTable<D, M, ModelTableValue = ModelHash<M>>
where
    <Self as NetabaseModelShardTable<D, M>>::ModelTableKeys: NetabaseModelSubscriptionKey<D, M>,
{
}
