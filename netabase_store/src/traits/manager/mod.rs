use crate::error::NetabaseResult;
use crate::traits::definition::{DiscriminantName, NetabaseDefinition, NetabaseDefinitionTrait};
use crate::traits::permission::PermissionEnumTrait;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use strum::{IntoDiscriminant, IntoEnumIterator};

pub mod store_link;
pub mod transaction;

/// Trait for managing multiple definition stores
///
/// This trait coordinates access to multiple definition databases, each stored
/// in its own isolated file/directory under a common parent path.
///
/// # Type Parameters
/// * `Self` - The manager enum type (must be IntoDiscriminant)
///
/// # Example
/// ```ignore
/// pub enum RestaurantManager {
///     Instance { manager: RedbDefinitionManager<...> },
/// }
///
/// impl DefinitionManagerTrait for RestaurantManager {
///     type Permissions = RestaurantPermissions;
///     type Backend = RedbStore<RestaurantDefinitions>;
///     type Definition = RestaurantDefinitions;
///
///     fn root_path(&self) -> &Path {
///         // Return path to parent directory
///     }
///
///     fn manager_name() -> &'static str {
///         "RestaurantManager"
///     }
/// }
/// ```
pub trait DefinitionManagerTrait: IntoDiscriminant + Sized
where
    <Self as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <Self::Permissions as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
    <Self::Definition as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// The permission enum type for this manager
    type Permissions: PermissionEnumTrait;

    /// The backend store type (e.g., RedbStore<D> or SledStore<D>)
    type Backend;

    /// The definition enum type that this manager coordinates
    type Definition: NetabaseDefinition;

    /// Get the root directory path where all definition databases are stored
    ///
    /// Each definition will have its own subdirectory under this path:
    /// `<root_path>/<definition_name>/store.db`
    fn root_path(&self) -> &Path;

    /// Get the name of this manager
    ///
    /// Used for the root TOML file: `<root_path>/<manager_name>.root.netabase.toml`
    fn manager_name() -> &'static str;

    /// Get the path for a specific definition's database
    ///
    /// Default implementation: `<root_path>/<definition_name>/store.db`
    ///
    /// # Arguments
    /// * `discriminant` - The discriminant identifying which definition
    ///
    /// # Returns
    /// Path to the definition's database file
    fn definition_path(
        &self,
        discriminant: &<Self::Definition as IntoDiscriminant>::Discriminant,
    ) -> PathBuf
    where
        <Self::Definition as IntoDiscriminant>::Discriminant: DiscriminantName,
    {
        let def_name = discriminant.name();
        self.root_path().join(def_name).join("store.db")
    }

    /// Get the path for a specific definition's TOML metadata file
    ///
    /// Default implementation: `<root_path>/<definition_name>/<definition_name>.netabase.toml`
    ///
    /// # Arguments
    /// * `discriminant` - The discriminant identifying which definition
    ///
    /// # Returns
    /// Path to the definition's TOML metadata file
    fn definition_toml_path(
        &self,
        discriminant: &<Self::Definition as IntoDiscriminant>::Discriminant,
    ) -> PathBuf
    where
        <Self::Definition as IntoDiscriminant>::Discriminant: DiscriminantName,
    {
        let def_name = discriminant.name();
        self.root_path()
            .join(def_name)
            .join(format!("{}.netabase.toml", def_name))
    }

    /// Get the path for the root manager TOML metadata file
    ///
    /// Default implementation: `<root_path>/<manager_name>.root.netabase.toml`
    fn root_toml_path(&self) -> PathBuf {
        self.root_path()
            .join(format!("{}.root.netabase.toml", Self::manager_name()))
    }

    /// Generate the root TOML metadata file
    ///
    /// This should be implemented to write manager-level metadata,
    /// including which definitions are available and permission roles.
    fn generate_root_toml(&self) -> NetabaseResult<()>;

    /// Load a definition store on-demand
    ///
    /// This should:
    /// 1. Check if the definition is already loaded
    /// 2. If not, open the database at `definition_path(discriminant)`
    /// 3. Update internal state to mark as loaded
    ///
    /// # Arguments
    /// * `discriminant` - The discriminant identifying which definition to load
    fn load_definition(
        &mut self,
        discriminant: <Self::Definition as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<()>;

    /// Unload a definition store
    ///
    /// This should:
    /// 1. Close the database connection
    /// 2. Update internal state to mark as unloaded
    ///
    /// # Arguments
    /// * `discriminant` - The discriminant identifying which definition to unload
    fn unload_definition(
        &mut self,
        discriminant: <Self::Definition as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<()>;

    /// Check if a definition is currently loaded
    ///
    /// # Arguments
    /// * `discriminant` - The discriminant identifying which definition to check
    ///
    /// # Returns
    /// `true` if the definition store is currently loaded in memory
    fn is_loaded(
        &self,
        discriminant: &<Self::Definition as IntoDiscriminant>::Discriminant,
    ) -> bool;
}

/// Marker trait for definitions that can be managed by a DefinitionManager
///
/// This trait extends NetabaseDefinition with permission support.
///
/// # Type Parameters
/// * `Self` - The definition enum type
///
/// # Example
/// ```ignore
/// impl ManagedDefinition for RestaurantDefinitions {
///     type Permissions = RestaurantPermissions;
/// }
/// ```
pub trait ManagedDefinition: NetabaseDefinition
where
    <Self as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <<Self as NetabaseDefinitionTrait>::Permissions as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    // No additional associated types - uses Permissions from NetabaseDefinitionTrait
}

// Note: ManagedDefinition must be implemented manually for each definition type.
// In Phase 5 (macro implementation), this will be generated automatically.
