use crate::traits::definition::{DiscriminantName, NetabaseDefinition};
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

/// Represents a definition store that can be either loaded or unloaded
///
/// This enum follows the same pattern as `RelationalLink<M, D>` but for entire
/// definition databases instead of individual models. It enables lazy loading
/// of definition stores - they start as Unloaded and are only loaded when accessed.
///
/// # Type Parameters
/// * `D` - The definition enum type
/// * `B` - The backend store type (e.g., RedbStore<D> or SledStore<D>)
///
/// # Example
/// ```ignore
/// // Initially unloaded
/// let link = DefinitionStoreLink::Unloaded(UserDefinitionDiscriminant::User);
///
/// // Load on first access
/// let store = RedbStore::new(path)?;
/// let link = DefinitionStoreLink::Loaded {
///     discriminant: UserDefinitionDiscriminant::User,
///     store,
/// };
///
/// // Check status
/// assert!(link.is_loaded());
/// ```
#[derive(Debug)]
pub enum DefinitionStoreLink<D, B>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Store is not loaded - only the discriminant is known
    ///
    /// In this state, no database connection is open and no resources are allocated.
    Unloaded(<D as IntoDiscriminant>::Discriminant),

    /// Store is loaded and ready for transactions
    ///
    /// In this state, the database is open and available for read/write operations.
    Loaded {
        /// The discriminant identifying which definition this store holds
        discriminant: <D as IntoDiscriminant>::Discriminant,
        /// The actual backend store (RedbStore<D> or SledStore<D>)
        store: B,
    },
}

impl<D, B> DefinitionStoreLink<D, B>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Get the discriminant, regardless of load state
    ///
    /// This works for both Unloaded and Loaded variants.
    ///
    /// # Returns
    /// Reference to the definition discriminant
    pub fn discriminant(&self) -> &<D as IntoDiscriminant>::Discriminant {
        match self {
            DefinitionStoreLink::Unloaded(d) => d,
            DefinitionStoreLink::Loaded { discriminant, .. } => discriminant,
        }
    }

    /// Check if the store is currently loaded
    ///
    /// # Returns
    /// `true` if the store is loaded, `false` if unloaded
    pub fn is_loaded(&self) -> bool {
        matches!(self, DefinitionStoreLink::Loaded { .. })
    }

    /// Check if the store is currently unloaded
    ///
    /// # Returns
    /// `true` if the store is unloaded, `false` if loaded
    pub fn is_unloaded(&self) -> bool {
        matches!(self, DefinitionStoreLink::Unloaded(_))
    }

    /// Get a reference to the store if it's loaded
    ///
    /// # Returns
    /// * `Some(&B)` - If the store is loaded
    /// * `None` - If the store is unloaded
    pub fn store(&self) -> Option<&B> {
        match self {
            DefinitionStoreLink::Unloaded(_) => None,
            DefinitionStoreLink::Loaded { store, .. } => Some(store),
        }
    }

    /// Get a mutable reference to the store if it's loaded
    ///
    /// # Returns
    /// * `Some(&mut B)` - If the store is loaded
    /// * `None` - If the store is unloaded
    pub fn store_mut(&mut self) -> Option<&mut B> {
        match self {
            DefinitionStoreLink::Unloaded(_) => None,
            DefinitionStoreLink::Loaded { store, .. } => Some(store),
        }
    }

    /// Unwrap the store, panicking if it's unloaded
    ///
    /// # Panics
    /// Panics if the store is Unloaded
    ///
    /// # Returns
    /// Reference to the backend store
    pub fn unwrap_store(&self) -> &B {
        match self {
            DefinitionStoreLink::Unloaded(d) => {
                panic!(
                    "Called unwrap_store on Unloaded definition store: {}",
                    d.name()
                )
            }
            DefinitionStoreLink::Loaded { store, .. } => store,
        }
    }

    /// Unwrap the store mutably, panicking if it's unloaded
    ///
    /// # Panics
    /// Panics if the store is Unloaded
    ///
    /// # Returns
    /// Mutable reference to the backend store
    pub fn unwrap_store_mut(&mut self) -> &mut B {
        match self {
            DefinitionStoreLink::Unloaded(d) => {
                panic!(
                    "Called unwrap_store_mut on Unloaded definition store: {}",
                    d.name()
                )
            }
            DefinitionStoreLink::Loaded { store, .. } => store,
        }
    }

    /// Get the store or return an error if unloaded
    ///
    /// This is a safer alternative to `unwrap_store()` that returns a Result
    /// instead of panicking.
    ///
    /// # Returns
    /// * `Ok(&B)` - If the store is loaded
    /// * `Err(NetabaseError::StoreNotLoaded)` - If the store is unloaded
    pub fn get_store(&self) -> crate::error::NetabaseResult<&B> {
        match self {
            DefinitionStoreLink::Unloaded(d) => Err(crate::error::NetabaseError::StoreNotLoaded(
                d.name().to_string(),
            )),
            DefinitionStoreLink::Loaded { store, .. } => Ok(store),
        }
    }

    /// Get the store mutably or return an error if unloaded
    ///
    /// This is a safer alternative to `unwrap_store_mut()` that returns a Result
    /// instead of panicking.
    ///
    /// # Returns
    /// * `Ok(&mut B)` - If the store is loaded
    /// * `Err(NetabaseError::StoreNotLoaded)` - If the store is unloaded
    pub fn get_store_mut(&mut self) -> crate::error::NetabaseResult<&mut B> {
        match self {
            DefinitionStoreLink::Unloaded(d) => Err(crate::error::NetabaseError::StoreNotLoaded(
                d.name().to_string(),
            )),
            DefinitionStoreLink::Loaded { store, .. } => Ok(store),
        }
    }

    /// Convert from Unloaded to Loaded state
    ///
    /// # Arguments
    /// * `store` - The backend store to associate with this link
    ///
    /// # Returns
    /// The discriminant that was previously stored (for verification)
    ///
    /// # Panics
    /// Panics if the link is already loaded
    pub fn load(&mut self, store: B) -> <D as IntoDiscriminant>::Discriminant {
        let discriminant = self.discriminant().clone();
        match self {
            DefinitionStoreLink::Unloaded(_) => {
                *self = DefinitionStoreLink::Loaded {
                    discriminant: discriminant.clone(),
                    store,
                };
            }
            DefinitionStoreLink::Loaded { .. } => {
                panic!(
                    "Attempted to load already-loaded definition store: {}",
                    discriminant.name()
                )
            }
        }
        discriminant
    }

    /// Convert from Loaded to Unloaded state, dropping the store
    ///
    /// # Returns
    /// The backend store that was dropped (if it was loaded)
    pub fn unload(&mut self) -> Option<B> {
        let discriminant = self.discriminant().clone();
        match std::mem::replace(self, DefinitionStoreLink::Unloaded(discriminant.clone())) {
            DefinitionStoreLink::Unloaded(_) => None,
            DefinitionStoreLink::Loaded { store, .. } => Some(store),
        }
    }
}

impl<D, B> Clone for DefinitionStoreLink<D, B>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    B: Clone,
{
    fn clone(&self) -> Self {
        match self {
            DefinitionStoreLink::Unloaded(d) => DefinitionStoreLink::Unloaded(d.clone()),
            DefinitionStoreLink::Loaded { discriminant, store } => {
                DefinitionStoreLink::Loaded {
                    discriminant: discriminant.clone(),
                    store: store.clone(),
                }
            }
        }
    }
}
