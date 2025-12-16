use crate::{
    errors::NetabaseResult,
    relational::{CrossDefinitionPermissions, GlobalDefinitionEnum, RelationalLink},
    traits::registery::{
        definition::NetabaseDefinition,
        models::{keys::NetabaseModelKeys, model::NetabaseModel},
    },
};
use strum::IntoDiscriminant;

pub trait NBTransaction<'db, D: NetabaseDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    type ReadTransaction;
    type WriteTransaction;

    fn create(&self, definition: &D) -> NetabaseResult<()>;

    fn read(&self, key: &D::DefKeys) -> NetabaseResult<Option<D>>;

    fn update(&self, definition: &D) -> NetabaseResult<()>;

    fn delete(&self, key: &D::DefKeys) -> NetabaseResult<()>;

    fn create_many(&self, definitions: &[D]) -> NetabaseResult<()>;

    fn read_if<F>(&self, predicate: F) -> NetabaseResult<Vec<D>>
    where
        F: Fn(&D) -> bool;

    fn read_range(&self, range: std::ops::Range<D::DefKeys>) -> NetabaseResult<Vec<D>>;

    fn update_range<F>(&self, range: std::ops::Range<D::DefKeys>, updater: F) -> NetabaseResult<()>
    where
        F: Fn(&mut D);

    fn update_if<P, U>(&self, predicate: P, updater: U) -> NetabaseResult<()>
    where
        P: Fn(&D) -> bool,
        U: Fn(&mut D);

    fn delete_many(&self, keys: &[D::DefKeys]) -> NetabaseResult<()>;

    fn delete_if<F>(&self, predicate: F) -> NetabaseResult<()>
    where
        F: Fn(&D) -> bool;

    fn delete_range(&self, range: std::ops::Range<D::DefKeys>) -> NetabaseResult<()>;

    fn write<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::WriteTransaction) -> NetabaseResult<R>;

    fn read_fn<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::ReadTransaction) -> NetabaseResult<R>;

    // Cross-definition relational operations
    fn read_related<OD>(&self, key: &OD::DefKeys) -> NetabaseResult<Option<OD>>
    where
        OD: NetabaseDefinition,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug;

    fn can_access_definition<OD>(&self) -> bool
    where
        OD: NetabaseDefinition,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug;
}
