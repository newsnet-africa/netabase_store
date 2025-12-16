use crate::traits::permissions::{AccessType, NetabasePermissionTicket};
use crate::traits::registery::definition::NetabaseDefinition;
use crate::traits::registery::models::model::NetabaseModel;
use crate::traits::registery::models::keys::NetabaseModelKeys;
use strum::IntoDiscriminant;
use std::collections::HashMap;
use std::marker::PhantomData;

/// Represents the global permission object passed to the transaction.
/// It acts as the "Ticket" held by the user/requestor.
#[derive(Debug, Clone)]
pub struct NetabasePermissions<D: NetabaseDefinition> 
where
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static
{
    /// Map of Model Name -> Allowed AccessType
    /// In a real implementation, this would be a more complex tree structure
    /// or the "generated enum" list described by the user.
    allowed_models: HashMap<String, Vec<AccessType>>,
    _marker: PhantomData<D>,
}

impl<D: NetabaseDefinition> Default for NetabasePermissions<D> 
where
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static
{
    fn default() -> Self {
        Self {
            allowed_models: HashMap::new(),
            _marker: PhantomData,
        }
    }
}

impl<D: NetabaseDefinition> NetabasePermissions<D> 
where
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow_all() -> Self {
        Self::default()
    }

    /// Grant permission to a specific model (by string name for now, as we don't have the specific Model types generic here easily)
    pub fn grant_permission(&mut self, model_name: &str, access: AccessType) {
        self.allowed_models.entry(model_name.to_string())
            .or_default()
            .push(access);
    }

    /// Check if a specific operation is allowed for a specific model name
    pub fn check(&self, model_name: &str, access: AccessType) -> bool {
        if let Some(perms) = self.allowed_models.get(model_name) {
            perms.contains(&access) 
                || (access == AccessType::Read && perms.contains(&AccessType::Update)) // Example hierarchy
                || (access == AccessType::Read && perms.contains(&AccessType::Create))
        } else {
            false
        }
    }
}

impl<D: NetabaseDefinition> NetabasePermissionTicket<D> for NetabasePermissions<D>
where
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static
{
    fn allows_access_to<M>(&self, access: AccessType) -> bool
    where
        M: NetabaseModel<D>,
        <D as IntoDiscriminant>::Discriminant: 'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'static>: 'static
    {
        let model_name = M::TREE_NAMES.main.table_name; // This is a &'static str
        self.check(model_name, access)
    }
}

pub type RedbPermissions<D> = NetabasePermissions<D>;
