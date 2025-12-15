use crate::{
    errors::NetabaseResult,
    traits::registery::{
        definition::NetabaseDefinition,
        models::{
            keys::NetabaseModelKeys,
            model::NetabaseModel,
        },
    },
    relational::{RelationalLink, CrossDefinitionPermissions, GlobalDefinitionEnum},
};
use strum::IntoDiscriminant;

pub trait NBTransaction<'db, D: NetabaseDefinition + GlobalDefinitionEnum>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    type ReadTransaction;
    type WriteTransaction;

    fn create<M>(&self, model: M) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;

    fn read<M>(&self, key: M::Keys) -> NetabaseResult<M>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;

    fn update<M>(&self, model: M) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;

    fn delete<M>(&self, key: M::Keys) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;

    fn create_many<M>(&self, models: Vec<M>) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;

    fn read_if<M, F>(&self, predicate: F) -> NetabaseResult<Vec<M>>
    where
        M: NetabaseModel<D>,
        F: Fn(&M) -> bool,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;

    fn read_range<M, K>(&self, range: std::ops::Range<K>) -> NetabaseResult<Vec<M>>
    where
        M: NetabaseModel<D>,
        K: Into<M::Keys>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;

    fn update_range<M, K, F>(&self, range: std::ops::Range<K>, updater: F) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        K: Into<M::Keys>,
        F: Fn(&mut M),
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;

    fn update_if<M, P, U>(&self, predicate: P, updater: U) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        P: Fn(&M) -> bool,
        U: Fn(&mut M),
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;

    fn delete_many<M>(&self, keys: Vec<M::Keys>) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;

    fn delete_if<M, F>(&self, predicate: F) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        F: Fn(&M) -> bool,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;

    fn delete_range<M, K>(&self, range: std::ops::Range<K>) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        K: Into<M::Keys>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;

    fn write<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::WriteTransaction) -> NetabaseResult<R>;

    fn read_fn<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::ReadTransaction) -> NetabaseResult<R>;

    // Cross-definition relational operations
    fn read_related<OD, M>(&self, key: M::Keys) -> NetabaseResult<Option<M>>
    where
        OD: NetabaseDefinition,
        M: NetabaseModel<OD>,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<M::Keys as NetabaseModelKeys<OD, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<OD, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;

    fn hydrate_relation<M>(&self, link: RelationalLink<M>) -> NetabaseResult<RelationalLink<M>>
    where
        M: GlobalDefinitionEnum,
        <M as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    ;

    fn can_access_definition<OD>(&self) -> bool
    where
        OD: NetabaseDefinition,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug;

    fn get_cross_permissions<OD>(&self) -> Option<CrossDefinitionPermissions<D>>
    where
        OD: NetabaseDefinition + GlobalDefinitionEnum,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug;

    fn create_with_relations<M>(&self, model: M, relations: Vec<RelationalLink<M>>) -> NetabaseResult<()>
    where
        M: NetabaseModel<D> + GlobalDefinitionEnum,
        <M as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;

    fn update_relations<M, RM>(&self, model_key: M::Keys, relation_updates: Vec<RelationalLink<RM>>) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        RM: GlobalDefinitionEnum,
        <RM as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static;
}
