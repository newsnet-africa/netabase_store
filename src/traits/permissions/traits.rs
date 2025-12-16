use crate::traits::registery::definition::NetabaseDefinition;
use crate::traits::registery::models::keys::NetabaseModelKeys;
use crate::traits::registery::models::model::{NetabaseModel, NetabaseModelMarker};
use strum::IntoDiscriminant;

/// Represents the intent of the access (Read/Write/etc)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Create,
    Update,
    Delete,
}

/// Trait for the generated enum that wraps a Relational Key in a Permission.
/// This corresponds to the user's description: "generated enum that holds a Relational key and wraps it in a permission enum"
pub trait NetabaseRelationalPermission<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
{
    type SourceModel: NetabaseModelMarker<D>;
    type TargetModel: NetabaseModelMarker<D>;

    /// Returns the type of access this permission grants (Read, Write, etc.)
    fn access_type(&self) -> AccessType;
}

/// The "Ticket" that holds a collection of permissions.
/// This is passed by the requestor.
pub trait NetabasePermissionTicket<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
{
    /// Check if this ticket contains a permission that allows access to the target model
    fn allows_access_to<M>(&self, access: AccessType) -> bool
    where
        M: NetabaseModel<D>,
        <D as IntoDiscriminant>::Discriminant: 'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'static>: 'static;
}

/// The "Guest List" implemented by Models/Definitions to validate incoming tickets.
pub trait NetabasePermissionRegistry<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
{
    /// The specific type of ticket this registry expects (usually the global or definition-level permission set)
    type Ticket: NetabasePermissionTicket<D>;

    /// The core check: "Does this set of incoming permissions include me?"
    /// And "do my accessors allow the source of these permissions?"
    fn check_access(ticket: &Self::Ticket, access: AccessType) -> bool;
}

/// Trait for the recursive/nested permission structure.
/// "tree-like structure that allows for parent child like relationships between permissions"
pub trait NetabasePermissionTree<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
{
    fn resolve(&self, target_model: &str) -> Option<AccessType>;
}
