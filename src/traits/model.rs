use strum::IntoDiscriminant;

use crate::definition::NetabaseDefinitionTrait;
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
/// ```ignore
/// use netabase_store::NetabaseModel;
///
/// #[derive(NetabaseModel, Clone, Debug, PartialEq,
///          bincode::Encode, bincode::Decode,
///          serde::Serialize, serde::Deserialize)]
/// #[netabase(MyDefinition)]
/// pub struct User {
///     #[primary_key]
///     pub id: u64,           // Unique identifier
///     pub name: String,       // Regular field
///     #[secondary_key]
///     pub email: String,      // Can query by email
///     #[secondary_key]
///     pub age: u32,           // Can query by age
/// }
///
/// // Usage
/// let user = User { id: 1, name: "Alice".into(), email: "alice@example.com".into(), age: 30 };
///
/// // Access primary key
/// let pk = user.primary_key();  // Returns UserPrimaryKey(1)
///
/// // Access secondary keys
/// let sk = user.secondary_keys(); // Returns vec![EmailKey("alice@example.com"), AgeKey(30)]
/// ```
///
/// # See Also
///
/// - [`NetabaseModel` derive macro](crate::NetabaseModel) - Derives this trait
/// - [`netabase_definition_module`](crate::netabase_definition_module) - Groups models into a schema
/// - [`NetabaseTreeSync`](crate::traits::tree::NetabaseTreeSync) - CRUD operations on models
pub trait NetabaseModelTrait<D: NetabaseDefinitionTrait>:
    bincode::Encode + Sized + Clone + MaybeSend + MaybeSync + 'static
where
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    const DISCRIMINANT: <D as IntoDiscriminant>::Discriminant;

    /// The primary key type for this model
    type PrimaryKey: NetabaseModelTraitKey<D>;

    /// The secondary keys enum for this model
    type SecondaryKeys: NetabaseModelTraitKey<D>;

    /// The keys enum that wraps both primary and secondary keys
    type Keys: NetabaseModelTraitKey<D>;

    /// Extract the primary key from the model instance
    fn primary_key(&self) -> Self::PrimaryKey;

    /// Extract all secondary keys from the model instance
    fn secondary_keys(&self) -> Vec<Self::SecondaryKeys>;

    /// Get the discriminant name for this model (used for tree names)
    fn discriminant_name() -> &'static str;
}

/// Marker trait for key types (both primary and secondary).
///
/// This trait is automatically implemented by the macro-generated key types.
pub trait NetabaseModelTraitKey<D: NetabaseDefinitionTrait>:
    bincode::Encode + std::fmt::Debug + Clone + MaybeSend + MaybeSync + 'static
where
    <<D as NetabaseDefinitionTrait>::Keys as IntoDiscriminant>::Discriminant:
        Clone + Copy + std::fmt::Debug + PartialEq + Eq + std::hash::Hash + MaybeSend + MaybeSync + 'static,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    const DISCRIMINANT: <<D as NetabaseDefinitionTrait>::Keys as IntoDiscriminant>::Discriminant;
}
