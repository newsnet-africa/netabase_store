use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

use crate::traits::store::tree_manager::TreeManager;

pub mod key;

/// Trait for converting discriminants to string names safely
/// Uses strum's AsRefStr for prefix matching support
pub trait DiscriminantName: AsRef<str> {
    fn name(&self) -> &str {
        self.as_ref()
    }
}

/// Wrapper struct for tree names to provide strong typing and consistency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TreeName<T>(pub T);

impl<T> TreeName<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    pub fn inner(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: DiscriminantName> DiscriminantName for TreeName<T> {
    fn name(&self) -> &str {
        self.0.name()
    }
}

impl<T: DiscriminantName> AsRef<str> for TreeName<T> {
    fn as_ref(&self) -> &str {
        self.0.name()
    }
}

pub trait NetabaseDefinition: IntoDiscriminant + TreeManager<Self> + Sized
where
    <Self as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <Self::Permissions as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    type Keys: IntoDiscriminant;

    /// The permission enum type for this definition
    ///
    /// This associated type enables compile-time permission checking for
    /// multi-definition store managers.
    ///
    /// For backward compatibility with existing single-definition stores,
    /// use `NoPermissions` if you don't need permission checking.
    ///
    /// Note: The permission type's discriminant must implement IntoEnumIterator,
    /// Hash, Eq, Debug, and Clone.
    ///
    /// # Example
    /// ```ignore
    /// impl NetabaseDefinition for RestaurantDefinitions {
    ///     // ... other associated types ...
    ///     type Permissions = RestaurantPermissions;  // or NoPermissions for no checks
    /// }
    /// ```
    type Permissions: crate::traits::permission::PermissionEnumTrait;

    fn name(&self) -> String;

    fn get_tree_name(discriminant: &<Self as IntoDiscriminant>::Discriminant) -> Option<String> {
        Some(discriminant.name().to_string())
    }
}