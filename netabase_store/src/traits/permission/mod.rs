use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

pub mod checker;
pub mod grants;

// Re-export key types for convenience
pub use checker::{GrantsReadAccess, GrantsWriteAccess};

/// Permission levels - similar to file permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PermissionLevel {
    /// No access permitted
    None,
    /// Read-only access
    Read,
    /// Write-only access (typically combined with Read for practical use)
    Write,
    /// Both read and write access
    ReadWrite,
    /// Full access including permission management
    Admin,
}

impl PermissionLevel {
    /// Check if this permission level allows reading
    pub fn can_read(&self) -> bool {
        matches!(self, PermissionLevel::Read | PermissionLevel::ReadWrite | PermissionLevel::Admin)
    }

    /// Check if this permission level allows writing
    pub fn can_write(&self) -> bool {
        matches!(self, PermissionLevel::Write | PermissionLevel::ReadWrite | PermissionLevel::Admin)
    }

    /// Check if this permission level allows both reading and writing
    pub fn can_read_write(&self) -> bool {
        matches!(self, PermissionLevel::ReadWrite | PermissionLevel::Admin)
    }
}

/// Core trait for permission enums
///
/// Permission enums must be enums with discriminants that can be used
/// for compile-time permission checking.
///
/// # Example
/// ```ignore
/// #[derive(Debug, Clone, EnumDiscriminants)]
/// #[strum_discriminants(derive(EnumIter, Hash))]
/// pub enum RestaurantPermissions {
///     Manager { grant: PermissionGrant<RestaurantDefinitions> },
///     Waiter { read: PermissionGrant<RestaurantDefinitions>,
///              write: PermissionGrant<RestaurantDefinitions> },
///     Customer { grant: PermissionGrant<RestaurantDefinitions> },
/// }
///
/// impl PermissionEnumTrait for RestaurantPermissions {
///     fn permission_level(&self) -> PermissionLevel {
///         match self {
///             RestaurantPermissions::Manager { .. } => PermissionLevel::ReadWrite,
///             RestaurantPermissions::Waiter { .. } => PermissionLevel::ReadWrite,
///             RestaurantPermissions::Customer { .. } => PermissionLevel::Read,
///         }
///     }
///
///     fn grants_access_to<R>(&self, definition: &R::Discriminant) -> bool
///     where
///         R: crate::traits::manager::DefinitionManagerTrait,
///     {
///         // Implementation based on variant
///         true
///     }
/// }
/// ```
pub trait PermissionEnumTrait: IntoDiscriminant + Sized
where
    <Self as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    /// Get the overall permission level for this permission variant
    fn permission_level(&self) -> PermissionLevel;

    /// Check if this permission grants access to a specific definition
    ///
    /// # Arguments
    /// * `definition` - The discriminant of the definition to check access for
    ///
    /// # Returns
    /// `true` if this permission grants any level of access to the definition
    fn grants_access_to<R>(&self, definition: &R::Discriminant) -> bool
    where
        R: IntoDiscriminant,
        R::Discriminant: IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone;

    /// Check if this permission grants read access to a specific definition
    fn can_read_definition<R>(&self, definition: &R::Discriminant) -> bool
    where
        R: IntoDiscriminant,
        R::Discriminant: IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
    {
        self.permission_level().can_read() && self.grants_access_to::<R>(definition)
    }

    /// Check if this permission grants write access to a specific definition
    fn can_write_definition<R>(&self, definition: &R::Discriminant) -> bool
    where
        R: IntoDiscriminant,
        R::Discriminant: IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
    {
        self.permission_level().can_write() && self.grants_access_to::<R>(definition)
    }
}

/// Wrapper type that holds definition discriminants with permission level
///
/// Used inside permission enum variants to specify which definitions
/// a permission role can access.
///
/// # Example
/// ```ignore
/// let manager_grant = PermissionGrant::read_write(vec![
///     RestaurantDefinitionsDiscriminants::User,
///     RestaurantDefinitionsDiscriminants::Product,
/// ]);
/// ```
#[derive(Debug, Clone)]
pub struct PermissionGrant<D>
where
    D: IntoDiscriminant,
    D::Discriminant: IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    pub level: PermissionLevel,
    pub definitions: Vec<D::Discriminant>,
}

impl<D> PermissionGrant<D>
where
    D: IntoDiscriminant,
    D::Discriminant: IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    /// Create a read-only permission grant
    pub fn read_only(definitions: Vec<D::Discriminant>) -> Self {
        Self {
            level: PermissionLevel::Read,
            definitions,
        }
    }

    /// Create a write-only permission grant
    pub fn write_only(definitions: Vec<D::Discriminant>) -> Self {
        Self {
            level: PermissionLevel::Write,
            definitions,
        }
    }

    /// Create a read-write permission grant
    pub fn read_write(definitions: Vec<D::Discriminant>) -> Self {
        Self {
            level: PermissionLevel::ReadWrite,
            definitions,
        }
    }

    /// Create a permission grant with no access
    pub fn none() -> Self {
        Self {
            level: PermissionLevel::None,
            definitions: Vec::new(),
        }
    }

    /// Check if this grant allows reading a specific definition
    pub fn can_read(&self, def: &D::Discriminant) -> bool {
        self.level.can_read() && self.definitions.contains(def)
    }

    /// Check if this grant allows writing a specific definition
    pub fn can_write(&self, def: &D::Discriminant) -> bool {
        self.level.can_write() && self.definitions.contains(def)
    }

    /// Check if this grant allows both reading and writing a specific definition
    pub fn can_read_write(&self, def: &D::Discriminant) -> bool {
        self.level.can_read_write() && self.definitions.contains(def)
    }

    /// Get all definitions this grant applies to
    pub fn definitions(&self) -> &[D::Discriminant] {
        &self.definitions
    }

    /// Get the permission level
    pub fn level(&self) -> PermissionLevel {
        self.level
    }
}

/// Default permission type for definitions that don't use permissions
///
/// This allows backward compatibility - existing single-definition stores
/// don't need to specify permissions.
#[derive(Debug, Clone, strum::EnumDiscriminants)]
#[strum_discriminants(derive(strum::EnumIter, Hash))]
pub enum NoPermissions {
    /// No permissions - always denies access
    None,
}

impl PermissionEnumTrait for NoPermissions {
    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::None
    }

    fn grants_access_to<R>(&self, _definition: &R::Discriminant) -> bool
    where
        R: IntoDiscriminant,
        R::Discriminant: IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
    {
        false
    }
}
