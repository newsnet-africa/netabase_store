use bincode::Decode;
use netabase_deps::blake3;
#[cfg(feature = "redb")]
use redb::{Key, Value};
use strum::IntoDiscriminant;

use crate::definition::{NetabaseDefinitionTrait, NetabaseDefinitionWithSubscription};
use crate::error::NetabaseError;
use crate::{MaybeSend, MaybeSync};

/// Trait for user-defined models that can be stored in the netabase database.
///
/// This trait is **automatically derived** using the `#[derive(NetabaseModel)]` macro.
/// You should never implement this trait manually - always use the derive macro.
///
/// # Requirements
///
/// Models must have:
/// - **One** `#[primary_key]` field - used for unique identification and primary access
/// - **Zero or more** `#[secondary_key]` fields - used for efficient querying
/// - All standard derives: `Clone`, `bincode::Encode`, `bincode::Decode`, `serde::Serialize`, `serde::Deserialize`
///
/// # Generated Types
///
/// The derive macro automatically generates:
/// - `{ModelName}PrimaryKey` - Newtype wrapper for the primary key
/// - `{ModelName}SecondaryKeys` - Enum of all secondary key types
/// - `{ModelName}Keys` - Combined enum of primary and secondary keys
///
/// # Examples
///
/// ```
/// use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
/// use netabase_store::traits::model::NetabaseModelTrait;
///
/// #[netabase_definition_module(MyDefinition, MyKeys)]
/// mod my_models {
///     use super::*;
///     use netabase_store::{NetabaseModel, netabase};
///
///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
///              bincode::Encode, bincode::Decode,
///              serde::Serialize, serde::Deserialize)]
///     #[netabase(MyDefinition)]
///     pub struct User {
///         #[primary_key]
///         pub id: u64,           // Unique identifier
///         pub name: String,       // Regular field
///         #[secondary_key]
///         pub email: String,      // Can query by email
///         #[secondary_key]
///         pub age: u32,           // Can query by age
///     }
/// }
/// use my_models::*;
///
/// // Usage
/// let user = User { id: 1, name: "Alice".into(), email: "alice@example.com".into(), age: 30 };
///
/// // Access primary key
/// let pk = user.primary_key();  // Returns UserPrimaryKey(1)
///
/// // Access secondary keys as a HashMap
/// let sk = user.secondary_keys(); // Returns HashMap with discriminant -> key mappings
/// ```
///
/// # See Also
///
/// - [`NetabaseModel` derive macro](crate::NetabaseModel) - Derives this trait
/// - [`netabase_definition_module`](crate::netabase_definition_module) - Groups models into a schema
/// - [`NetabaseTreeSync`](crate::traits::tree::NetabaseTreeSync) - CRUD operations on models
#[cfg(not(feature = "redb"))]
pub trait NetabaseModelTrait<D: NetabaseDefinitionTrait>:
    bincode::Encode + bincode::Decode<()> + Sized + Clone + MaybeSend + MaybeSync + 'static
where
    <D as IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as IntoDiscriminant>::Discriminant:
        crate::traits::definition::NetabaseKeyDiscriminant,
{
    const DISCRIMINANT: <D as IntoDiscriminant>::Discriminant;

    /// The keys enum that wraps both primary and secondary keys
    /// The Keys type must support conversion from both PrimaryKey and SecondaryKey
    type Keys: NetabaseModelTraitKey<D, PrimaryKey = Self::PrimaryKey, SecondaryKey = Self::SecondaryKeys>
        + From<Self::PrimaryKey>
        + From<Self::SecondaryKeys>;

    /// Primary key type
    type PrimaryKey: bincode::Encode + Decode<()> + Clone + Ord;

    /// Secondary keys type
    type SecondaryKeys: bincode::Encode + Decode<()> + Clone + Ord + IntoDiscriminant;

    fn key(&self) -> Self::Keys;

    /// Extract the primary key from the model instance
    fn primary_key(&self) -> Self::PrimaryKey;

    /// Extract all secondary keys from the model instance as a HashMap
    ///
    /// The HashMap is keyed by the secondary key discriminant, allowing direct
    /// access to specific secondary keys without iteration.
    fn secondary_keys(
        &self,
    ) -> std::collections::HashMap<
        <Self::SecondaryKeys as IntoDiscriminant>::Discriminant,
        Self::SecondaryKeys,
    >
    where
        Self::SecondaryKeys: IntoDiscriminant;

    fn has_secondary(&self) -> bool;

    /// Get the discriminant name for this model (used for tree names)
    fn discriminant_name() -> &'static str;
}

#[cfg(feature = "redb")]
pub trait NetabaseModelTrait<D: NetabaseDefinitionTrait>:
    bincode::Encode + bincode::Decode<()> + Sized + Clone + MaybeSend + MaybeSync + 'static
where
    for<'a> Self: std::borrow::Borrow<<Self as Value>::SelfType<'a>>,
    for<'a> Self: Value<SelfType<'a> = Self::BorrowedType<'a>>,
    <D as IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as IntoDiscriminant>::Discriminant:
        crate::traits::definition::NetabaseKeyDiscriminant,
{
    const DISCRIMINANT: <D as IntoDiscriminant>::Discriminant;

    type BorrowedType<'a>;

    /// The keys enum that wraps both primary and secondary keys
    /// The Keys type must support conversion from both PrimaryKey and SecondaryKey
    type Keys: NetabaseModelTraitKey<D, PrimaryKey = Self::PrimaryKey, SecondaryKey = Self::SecondaryKeys>
        + From<Self::PrimaryKey>
        + From<Self::SecondaryKeys>;

    /// Primary key type (for backwards compatibility)
    type PrimaryKey: InnerKey + bincode::Encode + Decode<()> + Clone + Ord;

    /// Secondary keys type (for backwards compatibility)
    type SecondaryKeys: InnerKey + bincode::Encode + Decode<()> + Clone + Ord + IntoDiscriminant;

    fn key(&self) -> Self::Keys;
    /// Extract the primary key from the model instance
    fn primary_key(&self) -> Self::PrimaryKey;

    /// Extract all secondary keys from the model instance as a HashMap
    ///
    /// The HashMap is keyed by the secondary key discriminant, allowing direct
    /// access to specific secondary keys without iteration.
    fn secondary_keys(
        &self,
    ) -> std::collections::HashMap<
        <Self::SecondaryKeys as IntoDiscriminant>::Discriminant,
        Self::SecondaryKeys,
    >
    where
        Self::SecondaryKeys: IntoDiscriminant;

    fn has_secondary(&self) -> bool;

    /// Get the discriminant name for this model (used for tree names)
    fn discriminant_name() -> &'static str;
}

/// Marker trait for key types (both primary and secondary).
///
/// This trait is automatically implemented by the macro-generated key types.
#[cfg(not(feature = "redb"))]
pub trait NetabaseModelTraitKey<D: NetabaseDefinitionTrait>:
    bincode::Encode + Decode<()> + std::fmt::Debug + Clone + MaybeSend + MaybeSync + 'static + Ord
where
    <D as IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as IntoDiscriminant>::Discriminant:
        crate::traits::definition::NetabaseKeyDiscriminant,
{
    const DISCRIMINANT: <<D as NetabaseDefinitionTrait>::Keys as IntoDiscriminant>::Discriminant;
    type PrimaryKey: bincode::Encode + Decode<()> + Clone + Ord;
    type SecondaryKey: bincode::Encode + Decode<()> + Clone + Ord + IntoDiscriminant;
}

/// Marker trait for key types (both primary and secondary).
///
/// This trait is automatically implemented by the macro-generated key types.
#[cfg(feature = "redb")]
pub trait NetabaseModelTraitKey<D: NetabaseDefinitionTrait>:
    bincode::Encode
    + Decode<()>
    + std::fmt::Debug
    + Clone
    + MaybeSend
    + MaybeSync
    + 'static
    + Ord
    + redb::Key
    + redb::Value
where
    <D as IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as IntoDiscriminant>::Discriminant:
        crate::traits::definition::NetabaseKeyDiscriminant,
    Self: Key,
    for<'a> Self: std::borrow::Borrow<<Self as redb::Value>::SelfType<'a>>,
{
    const DISCRIMINANT: <<D as NetabaseDefinitionTrait>::Keys as IntoDiscriminant>::Discriminant;
    type PrimaryKey: InnerKey + bincode::Encode + Decode<()> + Clone + Ord;
    type SecondaryKey: InnerKey + bincode::Encode + Decode<()> + Clone + Ord + IntoDiscriminant;
}

#[cfg(feature = "redb")]
pub trait InnerKey: Key + Value
where
    for<'a> Self: std::borrow::Borrow<<Self as Value>::SelfType<'a>>,
{
}

#[cfg(not(feature = "redb"))]
pub trait InnerKey: bincode::Encode + Decode<()> + Clone + Ord {}

pub trait SubscribedModel<D: NetabaseDefinitionWithSubscription>: NetabaseModelTrait<D>
where
    std::vec::Vec<u8>: std::convert::TryFrom<Self, Error = NetabaseError>,
    D: TryInto<Self>,
    Self: From<D>,
    <Self as NetabaseModelTrait<D>>::Keys: From<D::Keys>,
    D::Keys: TryInto<<Self as NetabaseModelTrait<D>>::Keys>,
{
    fn hashed(self) -> Result<blake3::Hash, NetabaseError> {
        let v: Vec<u8> = self.try_into()?;
        Ok(blake3::hash(&v))
    }

    fn subscriptions(&self) -> Vec<D::Subscriptions>;
}
