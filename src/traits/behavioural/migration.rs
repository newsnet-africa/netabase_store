use crate::traits::behavioural::metadata::MetadataProvider;
use crate::traits::structural::definition::NetabaseDefinition;
use crate::traits::structural::metadata::ModelMetadata;
use crate::traits::structural::models::NetabaseModel;
use crate::traits::structural::tables::{NetabaseModelShardTable, NetabaseModelTables};

pub trait VersionedModel<D: NetabaseDefinition, Prev>: NetabaseModel<D> {
    const VERSION: u32;
    fn migrate(prev: Prev) -> Self;
}

pub trait ModelFamily<D: NetabaseDefinition>:
    Sized + 'static + MetadataProvider<Metadata = ModelMetadata>
{
    type Latest: NetabaseModel<D>;

    fn family_name() -> &'static str;
    fn latest_version() -> u32;
    fn versions() -> Vec<u32>;
    fn migrate_to_latest() -> crate::results::NetabaseResult<()>;
    fn verify_family() -> crate::results::NetabaseResult<bool>;
}

pub trait VersionedModelTable<D: NetabaseDefinition, M: NetabaseModel<D>, PrevM: NetabaseModel<D>>:
    NetabaseModelShardTable<D, M>
{
    fn migrate_from_prev() -> crate::results::NetabaseResult<()>;
    fn migrate_all() -> crate::results::NetabaseResult<()>;
    fn migrate_range(
        start: Option<&Self::ModelTableKeys>,
        end: Option<&Self::ModelTableKeys>,
    ) -> crate::results::NetabaseResult<()>;
    fn verify() -> crate::results::NetabaseResult<bool>;
}

pub trait VersionedModelTables<D: NetabaseDefinition, M: NetabaseModel<D>>:
    NetabaseModelTables<D, M>
{
    fn migrate_all() -> crate::results::NetabaseResult<()>;
    fn verify_all() -> crate::results::NetabaseResult<bool>;
}
