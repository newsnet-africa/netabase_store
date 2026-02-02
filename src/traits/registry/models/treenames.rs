//! Table name management for models.
//!
//! This module provides types for managing table names in the database backend.
//! Each model and its associated indexes get unique table names in the format:
//! `{Definition}:{Model}:{KeyType}:{FieldName}` (in PascalCase).
//!
//! # Table Naming Convention
//!
//! Table names follow a hierarchical structure:
//! - **Definition**: The definition enum name (e.g., `MyApp`)
//! - **Model**: The model struct name (e.g., `User`)
//! - **KeyType**: The type of index (e.g., `Secondary`, `Blob`, `Relational`)
//! - **FieldName**: The specific field being indexed (e.g., `Email`)
//!
//! Example: `MyApp:User:Secondary:Email`
//!
//! # Types
//!
//! - [`DiscriminantTableName<D>`]: A pairing of discriminant and table name
//! - [`ModelTreeNames<D, M>`]: Complete collection of table names for a model
//!
//! # Example
//!
//! ```rust,ignore
//! // Generated code for a User model with email secondary key:
//! let tree_names = ModelTreeNames {
//!     main: DiscriminantTableName::new(UserDiscriminant, "MyApp:User"),
//!     secondary: &[DiscriminantTableName::new(EmailDiscriminant, "MyApp:User:Secondary:Email")],
//!     blob: &[],
//!     relational: &[],
//!     subscription: None,
//!     providers: &[],
//! };
//! ```

use crate::traits::registry::{
    definition::NetabaseDefinition,
    models::{keys::NetabaseModelKeys, model::NetabaseModel},
};
use strum::IntoDiscriminant;

/// A tuple that stores a discriminant alongside its formatted table name.
///
/// Table names follow the format: `{Definition}:{Model}:{KeyType}:{TableName}` in PascalCase.
///
/// # Type Parameters
///
/// - `D`: The discriminant type (usually an enum discriminant)
///
/// # Example
///
/// ```rust,ignore
/// let table_name = DiscriminantTableName::new(
///     UserEmailDiscriminant,
///     "MyApp:User:Secondary:Email"
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscriminantTableName<D> {
    /// The discriminant value for this table
    pub discriminant: D,
    /// The formatted table name as a static string
    pub table_name: &'static str, // Use &'static str for const contexts
}

impl<D> DiscriminantTableName<D> {
    /// Create a new discriminant-table name pair.
    ///
    /// # Arguments
    ///
    /// - `discriminant`: The discriminant value
    /// - `table_name`: The static table name string
    pub const fn new(discriminant: D, table_name: &'static str) -> Self {
        Self {
            discriminant,
            table_name,
        }
    }
}

/// Complete collection of table names for a model.
///
/// This struct contains references to all the table names that a model uses:
/// - Main table for primary key storage
/// - Secondary index tables
/// - Blob chunk tables
/// - Relational link tables
/// - Subscription topic tables
/// - P2P provider tables
///
/// # Type Parameters
///
/// - `'a`: Lifetime of the table name slices
/// - `D`: The definition this model belongs to
/// - `M`: The model type
///
/// # Example
///
/// ```rust,ignore
/// // Access table names for a model
/// let tree_names = User::table_definitions();
/// println!("Main table: {}", tree_names.main.table_name);
/// for secondary in tree_names.secondary {
///     println!("Secondary index: {}", secondary.table_name);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ModelTreeNames<'a, D: NetabaseDefinition, M>
where
    D::Discriminant: 'static + std::fmt::Debug,
    M: NetabaseModel<D>,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: IntoDiscriminant,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: IntoDiscriminant,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: IntoDiscriminant,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p: IntoDiscriminant,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    /// Main table for primary key storage
    pub main: DiscriminantTableName<D::Discriminant>,
    /// Secondary index tables
    pub secondary: &'a [DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant>],
    /// Blob chunk storage tables
    pub blob: &'a [DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant>],
    /// Relational link tables
    pub relational: &'a [DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant>],
    /// Subscription topic tables (optional)
    pub subscription: Option<&'a [DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant>]>,
    /// P2P provider tables
    pub providers: &'a [DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant>]
}

// Manual PartialEq implementation for ModelTreeNames comparing by table names
impl<'a, D: NetabaseDefinition, M> PartialEq for ModelTreeNames<'a, D, M>
where
    D::Discriminant: 'static + std::fmt::Debug,
    M: NetabaseModel<D>,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: IntoDiscriminant,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: IntoDiscriminant,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: IntoDiscriminant,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    /// Compare two ModelTreeNames by their main table name.
    ///
    /// This is sufficient because table names are unique per model.
    fn eq(&self, other: &Self) -> bool {
        // Compare by main table name
        self.main.table_name == other.main.table_name
    }
}
