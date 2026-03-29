use crate::traits::structural::definition::NetabaseDefinition;
use crate::traits::structural::models::NetabaseModel;
use crate::traits::structural::values::{BlobValue, ModelHash, StoreValue};
use std::fmt::Debug;

pub trait NetabaseKey<D, M, V>: Sized
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    V: StoreValue,
{
    type Raw: ?Sized;
    type StoreKey: ?Sized;
    type StoreValue: ?Sized;
    fn model_label() -> <M as NetabaseModel<D>>::ModelName;
}

pub trait NetabaseModelKeys<D: NetabaseDefinition, M: NetabaseModel<D>> {
    type Primary: NetabaseModelPrimaryKey<D, M>;
    type Secondary: NetabaseModelSecondaryKey<D, M>;
    type Blob: NetabaseModelBlobKey<D, M>;
    type Subscription: NetabaseModelSubscriptionKey<D, M>;
    type Relational: ModelRelationalKey<D, M>;
}

pub trait ModelRelationalKey<D: NetabaseDefinition, M: NetabaseModel<D>> {}

pub trait SegmentedKey<D, M, V>: NetabaseKey<D, M, V>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    V: StoreValue,
{
    type Directory: DirectorySegment;
    type Final: FinalSegment;
    fn split_ref(&self) -> (&[Self::Directory], &Self::Final);
}

pub trait DirectorySegment {}
pub trait FinalSegment {}

pub trait NetabaseModelPrimaryKey<D: NetabaseDefinition, M: NetabaseModel<D>>:
    NetabaseKey<D, M, M> + StoreValue
{
}

pub trait NetabaseModelSecondaryKey<D: NetabaseDefinition, M: NetabaseModel<D>>:
    NetabaseKey<D, M, Self::PrimaryKey>
where
    Self::PrimaryKey: StoreValue,
{
    type PrimaryKey: NetabaseModelPrimaryKey<D, M>;
}

pub trait NetabaseModelBlobKey<D: NetabaseDefinition, M: NetabaseModel<D>>:
    NetabaseKey<D, M, BlobValue>
{
}

pub trait NetabaseModelSubscriptionKey<D: NetabaseDefinition, M: NetabaseModel<D>>:
    NetabaseKey<D, M, ModelHash<M>>
{
}

pub trait RelationalTargetModel<
    FromD: NetabaseDefinition,
    FromM: NetabaseModel<FromD>,
    ToD: NetabaseDefinition,
>: Sized + NetabaseModel<ToD>
{
}

pub trait NetabaseModelRelationalKey<
    FromD: NetabaseDefinition,
    FromM: NetabaseModel<FromD>,
    ToD: NetabaseDefinition,
    ToM: RelationalTargetModel<FromD, FromM, ToD>,
>: NetabaseKey<FromD, FromM, ToM> + ModelRelationalKey<FromD, FromM> where
    <ToM::Keys as NetabaseModelKeys<ToD, ToM>>::Primary: StoreValue,
{
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationshipLevel {
    SameModel,
    SameDefinition,
    SameRepository,
}
