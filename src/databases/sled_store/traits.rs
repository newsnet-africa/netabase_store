//! Sled-specific trait extensions for Netabase models
//!
//! This module provides trait extensions that add sled-specific functionality
//! to Netabase models. These traits mirror the redb-specific traits but require
//! bincode serialization instead of redb's Key and Value traits.

use crate::{
    error::NetabaseResult,
    traits::{
        definition::{DiscriminantName, NetabaseDefinition},
        model::{NetabaseModelTrait, key::NetabaseModelKeyTrait},
    },
};
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

/// Sled-specific extension trait for models
///
/// This trait extends `NetabaseModelTrait` with sled-specific requirements and operations.
/// Unlike `RedbNetabaseModelTrait` which requires `redb::Key` and `redb::Value` traits,
/// this trait requires `bincode::Encode` and `bincode::Decode` for serialization.
///
/// ## Type Requirements
///
/// - `Self`: Must implement `bincode::Encode` and `bincode::Decode`
/// - `Self::PrimaryKey`: Must implement `bincode::Encode` and `bincode::Decode`
///
/// ## Tree Naming
///
/// This trait provides methods for generating sled tree names that match the
/// naming scheme used by redb for consistency across backends.
///
/// ## Example
///
/// ```ignore
/// use netabase_store::databases::sled_store::SledNetabaseModelTrait;
///
/// impl<D: NetabaseDefinition> SledNetabaseModelTrait<D> for User
/// where
///     <D as IntoDiscriminant>::Discriminant:
///         IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
/// {
///     fn secondary_key_table_name(discriminant: Discriminant) -> String {
///         format!("User::secondary::{}", discriminant.name())
///     }
///
///     // ... implement other methods
/// }
/// ```
pub trait SledNetabaseModelTrait<D: NetabaseDefinition>: NetabaseModelTrait<D>
where
    Self: bincode::Encode + bincode::Decode<()> + 'static,
    Self::PrimaryKey: bincode::Encode + bincode::Decode<()> + 'static,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Generate the tree name for a secondary key index
    ///
    /// Secondary key trees map secondary keys to primary keys, enabling
    /// lookups by fields other than the primary key.
    ///
    /// ## Format
    ///
    /// `"{model_name}::secondary::{key_discriminant}"`
    ///
    /// ## Example
    ///
    /// For a `User` model with an `Email` secondary key:
    /// ```text
    /// "User::secondary::Email"
    /// ```
    fn secondary_key_table_name(
        key_discriminant: <<Self::Keys as NetabaseModelKeyTrait<D, Self>>::SecondaryEnum as IntoDiscriminant>::Discriminant,
    ) -> String;

    /// Generate the tree name for a relational key index
    ///
    /// Relational key trees map relational keys to primary keys, enabling
    /// relationship queries (e.g., "find all reviews by this user").
    ///
    /// ## Format
    ///
    /// `"{model_name}::relational::{key_discriminant}"`
    ///
    /// ## Example
    ///
    /// For a `Review` model with a `ReviewerId` relational key:
    /// ```text
    /// "Review::relational::ReviewerId"
    /// ```
    fn relational_key_table_name(
        key_discriminant: <<Self::Keys as NetabaseModelKeyTrait<D, Self>>::RelationalEnum as IntoDiscriminant>::Discriminant,
    ) -> String;

    /// Generate the tree name for a subscription index
    ///
    /// Subscription trees map primary keys to their hashes for a specific
    /// subscription type, enabling efficient sync detection through XOR
    /// accumulation.
    ///
    /// ## Format
    ///
    /// `"{model_name}::subscription::{subscription_discriminant}"`
    ///
    /// ## Example
    ///
    /// For a `User` model with a `Premium` subscription:
    /// ```text
    /// "User::subscription::Premium"
    /// ```
    fn subscription_key_table_name(
        key_discriminant: <<Self as NetabaseModelTrait<D>>::SubscriptionEnum as IntoDiscriminant>::Discriminant,
    ) -> String;

    /// Generate the tree name for the hash tree
    ///
    /// Hash trees map primary keys to their content hashes, enabling
    /// integrity verification and change detection.
    ///
    /// ## Format
    ///
    /// `"{model_name}::hash"`
    ///
    /// ## Example
    ///
    /// For a `User` model:
    /// ```text
    /// "User::hash"
    /// ```
    fn hash_tree_table_name() -> String;
}

/// Extension trait for ModelAssociatedTypes to provide Sled execution methods
///
/// This trait provides low-level methods for inserting and deleting models
/// and their associated indices in sled trees. These methods are typically
/// called by the transaction layer rather than directly by user code.
///
/// ## Type Parameters
///
/// - `D`: The Definition enum type
///
/// ## Example
///
/// ```ignore
/// // Typically used internally by write transactions
/// associated_type.insert_model_into_sled(
///     &db,
///     "User",
///     &primary_key_wrapped,
/// )?;
/// ```
pub trait SledModelAssociatedTypesExt<D>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Insert a model into the main tree
    ///
    /// This method serializes and inserts a model into its main tree,
    /// indexed by its primary key.
    ///
    /// ## Arguments
    ///
    /// - `db`: The sled database
    /// - `tree_name`: The name of the model's main tree
    /// - `key`: The wrapped primary key
    ///
    /// ## Errors
    ///
    /// Returns an error if serialization fails or the database operation fails.
    fn insert_model_into_sled(
        &self,
        db: &sled::Db,
        tree_name: &str,
        key: &D::ModelAssociatedTypes,
    ) -> NetabaseResult<()>;

    /// Insert a secondary key mapping
    ///
    /// This method creates a mapping from a secondary key to a primary key,
    /// enabling lookups by the secondary key.
    ///
    /// ## Arguments
    ///
    /// - `db`: The sled database
    /// - `tree_name`: The name of the secondary key index tree
    /// - `primary_key_ref`: The wrapped primary key to point to
    ///
    /// ## Errors
    ///
    /// Returns an error if serialization fails or the database operation fails.
    fn insert_secondary_key_into_sled(
        &self,
        db: &sled::Db,
        tree_name: &str,
        primary_key_ref: &D::ModelAssociatedTypes,
    ) -> NetabaseResult<()>;

    /// Insert a relational key mapping
    ///
    /// This method creates a mapping from a relational key to a primary key,
    /// enabling relationship queries.
    ///
    /// ## Arguments
    ///
    /// - `db`: The sled database
    /// - `tree_name`: The name of the relational key index tree
    /// - `primary_key_ref`: The wrapped primary key to point to
    ///
    /// ## Errors
    ///
    /// Returns an error if serialization fails or the database operation fails.
    fn insert_relational_key_into_sled(
        &self,
        db: &sled::Db,
        tree_name: &str,
        primary_key_ref: &D::ModelAssociatedTypes,
    ) -> NetabaseResult<()>;

    /// Insert a hash tree mapping
    ///
    /// This method stores the content hash for a model, indexed by its
    /// primary key.
    ///
    /// ## Arguments
    ///
    /// - `hash`: The content hash of the model
    /// - `db`: The sled database
    /// - `tree_name`: The name of the hash tree
    /// - `primary_key_ref`: The wrapped primary key
    ///
    /// ## Errors
    ///
    /// Returns an error if serialization fails or the database operation fails.
    fn insert_hash_into_sled(
        hash: &[u8; 32],
        db: &sled::Db,
        tree_name: &str,
        primary_key_ref: &D::ModelAssociatedTypes,
    ) -> NetabaseResult<()>;

    /// Insert a subscription tree mapping
    ///
    /// This method stores a hash in a subscription tree, indexed by the
    /// primary key. Subscription trees enable efficient sync detection
    /// through XOR accumulation.
    ///
    /// ## Arguments
    ///
    /// - `hash`: The content hash of the model
    /// - `db`: The sled database
    /// - `tree_name`: The name of the subscription tree
    /// - `primary_key_ref`: The wrapped primary key
    ///
    /// ## Errors
    ///
    /// Returns an error if serialization fails or the database operation fails.
    fn insert_subscription_into_sled(
        hash: &[u8; 32],
        db: &sled::Db,
        tree_name: &str,
        primary_key_ref: &D::ModelAssociatedTypes,
    ) -> NetabaseResult<()>;

    /// Delete a model from the main tree
    ///
    /// This method removes a model from its main tree by primary key.
    /// Note: This does not automatically delete associated indices.
    ///
    /// ## Arguments
    ///
    /// - `db`: The sled database
    /// - `tree_name`: The name of the model's main tree
    ///
    /// ## Errors
    ///
    /// Returns an error if serialization fails or the database operation fails.
    fn delete_model_from_sled(
        &self,
        db: &sled::Db,
        tree_name: &str,
    ) -> NetabaseResult<()>;

    /// Delete a subscription tree mapping
    ///
    /// This method removes a hash from a subscription tree.
    ///
    /// ## Arguments
    ///
    /// - `db`: The sled database
    /// - `tree_name`: The name of the subscription tree
    ///
    /// ## Errors
    ///
    /// Returns an error if serialization fails or the database operation fails.
    fn delete_subscription_from_sled(
        &self,
        db: &sled::Db,
        tree_name: &str,
    ) -> NetabaseResult<()>;
}
