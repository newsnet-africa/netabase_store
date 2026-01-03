//! Definition trait hierarchy for grouping related models.
//!
//! A **Definition** is a collection of related models that share a common schema context.
//! For example, an `Employee` definition might contain `User`, `Shift`, `Timecard` models.
//!
//! # Core Concepts
//!
//! - **Definition**: A namespace grouping related models
//! - **TreeNames**: Enum wrapping all table names for a definition
//! - **DefKeys**: Enum wrapping all key types across all models
//! - **DebugName**: Human-readable identifier for debugging relational links
//! - **Schema**: The complete schema metadata for export/import
//!
//! # Schema Export/Import
//!
//! Definitions can export their schema to TOML for:
//! - Version control of schema evolution
//! - P2P schema negotiation
//! - Migration planning
//! - Documentation generation
//!
//! # Example
//!
//! ```rust,ignore
//! #[netabase_definition(Employee)]
//! mod employee {
//!     #[netabase_model]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: UserId,
//!         pub name: String,
//!     }
//!
//!     #[netabase_model]
//!     pub struct Shift {
//!         #[primary_key]
//!         pub id: ShiftId,
//!         #[relational]
//!         pub user: RelationalLink<User>,
//!     }
//! }
//!
//! // Generated code provides:
//! // - EmployeeDefinition enum wrapping User | Shift
//! // - EmployeeTreeNames for all table names
//! // - EmployeeDefKeys for all key types
//! // - Schema export via Employee::export_toml()
//! ```
//!
//! # Subscription System
//!
//! Definitions coordinate subscription keys across models, enabling:
//! - Topic-based message routing
//! - Many-to-many relationships via topics
//! - Event publishing to subscribed models
//!
//! See [`subscription`] module for details.

//! Definition trait hierarchy for grouping related models.
//!
//! A **definition** is a logical grouping of related models that share a common
//! schema and lifecycle. Definitions are the primary unit of schema organization
//! in netabase_store.
//!
//! # Architecture
//!
//! Each definition:
//! - Contains one or more models (tables)
//! - Has a unique discriminant for type-safe access
//! - Defines its subscription topics
//! - Exports a schema for P2P synchronization
//! - Can belong to one or more repositories
//!
//! # The `NetabaseDefinition` Trait
//!
//! This is the core trait for definitions. It's automatically implemented
//! by the `#[netabase_definition]` macro and provides:
//!
//! - **Type Safety**: Each definition has a unique discriminant type
//! - **Schema Export**: Generate TOML schemas for P2P comparison
//! - **Subscription Management**: Define topics for pub/sub patterns
//! - **Debug Names**: Human-readable identifiers for serialization
//!
//! # Example Structure
//!
//! ```rust,ignore
//! #[netabase_definition(UserDef)]
//! mod user_definition {
//!     #[netabase_model]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: UserId,
//!         pub name: String,
//!     }
//!
//!     #[netabase_model]
//!     pub struct Post {
//!         #[primary_key]
//!         pub id: PostId,
//!         #[link(UserDef, User)]
//!         pub author: UserId,
//!     }
//! }
//! ```
//!
//! This generates:
//! - `UserDef` enum with `User` and `Post` variants
//! - `UserDefDiscriminant` for type-safe pattern matching
//! - `UserDefTreeNames` for database table naming
//! - `UserDefKeys` for unified key access
//!
//! See [tests/comprehensive_functionality.rs] for complete usage examples.

pub mod redb_definition;
pub mod schema;
pub mod subscription;

use schema::DefinitionSchema;
use serde::Serialize;
use strum::IntoDiscriminant;
use subscription::{DefinitionSubscriptionRegistry, NetabaseDefinitionSubscriptionKeys};

use crate::traits::registery::models::{
    keys::NetabaseModelKeys,
    model::NetabaseModel,
    treenames::{DiscriminantTableName, ModelTreeNames},
};

/// Core trait for definition abstractions.
///
/// A definition is a collection of related models that share a schema.
/// This trait is automatically implemented by the `#[netabase_definition]`
/// macro and should not be manually implemented.
///
/// # Associated Types
///
/// - **TreeNames**: Enum of all table names in this definition
/// - **DefKeys**: Unified key type wrapping all model keys
/// - **DebugName**: Human-readable identifier for debugging/serialization
/// - **SubscriptionKeys**: Enum of subscription topics
/// - **SubscriptionKeysDiscriminant**: Discriminant for subscription enum
///
/// # Methods
///
/// - `debug_name()`: Returns the definition's human-readable name
/// - `schema()`: Returns the structured schema definition
/// - `export_toml()`: Exports the schema as TOML for P2P sync
///
/// # Schema Export
///
/// The `export_toml()` method generates a complete schema including:
/// - All model structures with field types
/// - Primary, secondary, and relational keys
/// - Blob field definitions
/// - Subscription topics
///
/// # Example Usage
///
/// ```rust,ignore
/// use netabase_store::traits::registery::definition::NetabaseDefinition;
///
/// // Generated by #[netabase_definition(MyDef)]
/// let schema = MyDef::schema();
/// let toml = MyDef::export_toml();
///
/// // Schema can be compared with remote nodes
/// if local_schema_hash != remote_schema_hash {
///     // Schemas differ - may need migration
/// }
/// ```
///
/// # Trait Bounds
///
/// Requires `IntoDiscriminant` for efficient pattern matching on definition
/// variants. The discriminant must be `'static + std::fmt::Debug`.
pub trait NetabaseDefinition: IntoDiscriminant + Sized
where
    Self::Discriminant: 'static + std::fmt::Debug,
{
    type TreeNames: NetabaseDefinitionTreeNames<Self> + 'static;
    type DefKeys: NetabaseDefinitionKeys<Self>;

    /// A user-friendly identifier for the definition, used in RelationalLink for better debugging/serialization.
    /// This replaces PhantomData to bind the definition type while providing useful info.
    type DebugName: Clone + std::fmt::Debug + PartialEq + Eq + std::hash::Hash + Serialize + 'static;

    /// Returns the debug identifier for this definition
    fn debug_name() -> Self::DebugName;

    /// Returns the schema definition
    fn schema() -> DefinitionSchema;

    /// Exports the schema to a TOML string
    fn export_toml() -> String {
        let schema = Self::schema();
        toml::to_string_pretty(&schema)
            .unwrap_or_else(|e| format!("# Error serializing to TOML: {}", e))
    }

    /// Definition-level subscription keys enum
    /// This enum holds all subscription topics for the definition
    /// and serves as the unified key type for subscription tables
    type SubscriptionKeys: NetabaseDefinitionSubscriptionKeys<
        Discriminant = Self::SubscriptionKeysDiscriminant,
    >;

    /// Discriminant type for subscription keys
    type SubscriptionKeysDiscriminant: 'static + std::fmt::Debug;

    /// Subscription registry mapping topics to models
    const SUBSCRIPTION_REGISTRY: DefinitionSubscriptionRegistry<'static, Self>;
}

/// Trait for an enum that encapsulates the tree names for all models in a definition.
///
/// This trait provides a unified interface for accessing database table names
/// across all models in a definition. The structure is nested:
/// `Definition -> Model -> TreeNames`.
///
/// # Purpose
///
/// - Type-safe access to table names without string literals
/// - Compile-time verification of table existence
/// - Support for discriminant-based table lookup
///
/// # Generated Structure
///
/// For a definition with models `User` and `Post`, the macro generates:
///
/// ```rust,ignore
/// pub enum MyDefTreeNames {
///     User(UserTreeNames),
///     Post(PostTreeNames),
/// }
///
/// pub enum UserTreeNames {
///     Primary,
///     SecondaryEmail,
///     BlobAvatar,
///     // ... other tables
/// }
/// ```
///
/// # Usage
///
/// ```rust,ignore
/// // Get all tree names for a specific model discriminant
/// let tree_names = MyDefTreeNames::get_tree_names(MyDefDiscriminant::User);
///
/// // Extract model-specific trees
/// for tree in tree_names {
///     if let Some(user) = tree.get_model_tree::<User>() {
///         // Access User model
///     }
/// }
/// ```
pub trait NetabaseDefinitionTreeNames<D: NetabaseDefinition>: Sized + Clone
where
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    Self: TryInto<DiscriminantTableName<D>>,
{
    // Methods to access specific tree names based on the definition's discriminant
    fn get_tree_names(discriminant: D::Discriminant) -> Vec<Self>;

    fn get_model_tree<M: NetabaseModel<D>>(&self) -> Option<M>
    where
        for<'a> Self: From<ModelTreeNames<'a, Self, M>>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: IntoDiscriminant,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: IntoDiscriminant,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: IntoDiscriminant,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: IntoDiscriminant,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static
    ;
}

/// Trait for an enum that encapsulates the keys for all models in a definition.
///
/// This trait provides unified access to all key types across all models,
/// enabling polymorphic operations on mixed model data. The structure is nested:
/// `Definition -> Model -> KeyType -> ConcreteKey`.
///
/// # Purpose
///
/// - **Unified Key Handling**: Work with keys from different models uniformly
/// - **Relational Links**: Enable cross-model references via discriminated keys
/// - **Type Safety**: Preserve type information through discriminants
///
/// # Generated Structure
///
/// For a definition with models `User` and `Post`, the macro generates:
///
/// ```rust,ignore
/// #[derive(Debug, Clone)]
/// pub enum MyDefKeys {
///     UserPrimary(UserId),
///     UserSecondary(UserSecondaryKeys),
///     UserRelational(UserRelationalKeys),
///     PostPrimary(PostId),
///     PostSecondary(PostSecondaryKeys),
///     // ... other key variants
/// }
/// ```
///
/// # Usage in Relational Links
///
/// ```rust,ignore
/// // RelationalLink stores a DefKeys variant
/// let link = RelationalLink::new_dehydrated(MyDefKeys::UserPrimary(user_id));
///
/// // Pattern match to extract specific key type
/// match link.key {
///     MyDefKeys::UserPrimary(id) => { /* ... */ }
///     _ => { /* ... */ }
/// }
/// ```
///
/// See [tests/comprehensive_functionality.rs] for examples.
pub trait NetabaseDefinitionKeys<D: NetabaseDefinition>: Sized + Clone + std::fmt::Debug
where
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    // Methods to access specific keys, potentially converting from/to generic representations
}
