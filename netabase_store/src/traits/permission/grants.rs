use super::{PermissionGrant, PermissionLevel};
use crate::traits::definition::{DiscriminantName, NetabaseDefinition};
use std::marker::PhantomData;

/// Helper for implementing compile-time permission checks
///
/// This struct can be used in const contexts to verify permissions at compile time.
/// It uses PhantomData to carry type information without runtime overhead.
///
/// # Example
/// ```ignore
/// impl<D> GrantsReadAccess<User> for RestaurantPermissions
/// where
///     D: NetabaseDefinition,
///     RestaurantPermissions: HasReadPermission<User, D>,
/// {
///     fn assert_read_access() {
///         // Compile-time assertion - zero runtime cost
///     }
/// }
/// ```
pub struct PermissionChecker<P, D, M> {
    _permission: PhantomData<P>,
    _definition: PhantomData<D>,
    _model: PhantomData<M>,
}

impl<P, D, M> PermissionChecker<P, D, M> {
    /// Create a new permission checker
    ///
    /// This is a zero-cost abstraction - no runtime overhead.
    pub const fn new() -> Self {
        Self {
            _permission: PhantomData,
            _definition: PhantomData,
            _model: PhantomData,
        }
    }
}

/// Helper trait for checking if a permission grants read access at compile time
///
/// This trait should be implemented by permission types that want to support
/// compile-time permission checking. It works in conjunction with GrantsReadAccess.
///
/// # Type Parameters
/// * `M` - The model type being accessed
/// * `D` - The definition enum containing the model
pub trait HasReadPermission<M, D>
where
    D: NetabaseDefinition,
    <D as strum::IntoDiscriminant>::Discriminant:
        strum::IntoEnumIterator + std::hash::Hash + Eq + std::fmt::Debug + DiscriminantName + Clone,
{
    /// Check if this permission grants read access to model M
    ///
    /// This should return true if the permission includes read access
    /// to the definition containing model M.
    fn has_read(
        &self,
        definition_discriminant: &<D as strum::IntoDiscriminant>::Discriminant,
    ) -> bool;
}

/// Helper trait for checking if a permission grants write access at compile time
///
/// This trait should be implemented by permission types that want to support
/// compile-time permission checking. It works in conjunction with GrantsWriteAccess.
///
/// # Type Parameters
/// * `M` - The model type being accessed
/// * `D` - The definition enum containing the model
pub trait HasWritePermission<M, D>
where
    D: NetabaseDefinition,
    <D as strum::IntoDiscriminant>::Discriminant:
        strum::IntoEnumIterator + std::hash::Hash + Eq + std::fmt::Debug + DiscriminantName + Clone,
{
    /// Check if this permission grants write access to model M
    ///
    /// This should return true if the permission includes write access
    /// to the definition containing model M.
    fn has_write(
        &self,
        definition_discriminant: &<D as strum::IntoDiscriminant>::Discriminant,
    ) -> bool;
}

/// Helper function for runtime permission checking
///
/// This is used when `PERM_CHECK = false` in transaction methods.
///
/// # Arguments
/// * `grant` - The permission grant to check
/// * `discriminant` - The definition being accessed
/// * `write_access` - Whether write access is required
///
/// # Returns
/// * `Ok(())` - If permission is granted
/// * `Err(PermissionDenied)` - If permission is denied
pub fn check_permission_runtime<D>(
    grant: &PermissionGrant<D>,
    discriminant: &<D as strum::IntoDiscriminant>::Discriminant,
    write_access: bool,
) -> crate::error::NetabaseResult<()>
where
    D: NetabaseDefinition,
    <D as strum::IntoDiscriminant>::Discriminant:
        strum::IntoEnumIterator + std::hash::Hash + Eq + std::fmt::Debug + DiscriminantName + Clone,
{
    // Check if the discriminant is in the grant's definition list
    if !grant.definitions.contains(discriminant) {
        return Err(crate::error::NetabaseError::PermissionDenied(format!(
            "Permission does not include access to definition: {:?}",
            discriminant
        )));
    }

    // Check permission level
    match (grant.level, write_access) {
        (PermissionLevel::ReadWrite, _) => Ok(()),
        (PermissionLevel::Read, false) => Ok(()),
        (PermissionLevel::Read, true) => {
            Err(crate::error::NetabaseError::PermissionDenied(format!(
                "Permission is read-only, cannot write to definition: {:?}",
                discriminant
            )))
        }
        (PermissionLevel::Write, false) => {
            Err(crate::error::NetabaseError::PermissionDenied(format!(
                "Permission is write-only, cannot read from definition: {:?}",
                discriminant
            )))
        }
        (PermissionLevel::Write, true) => Ok(()),
        (PermissionLevel::None, _) => Err(crate::error::NetabaseError::PermissionDenied(format!(
            "No permission to access definition: {:?}",
            discriminant
        ))),
        (PermissionLevel::Admin, true) => todo!(),
        (PermissionLevel::Admin, false) => todo!(),
    }
}

/// Macro for implementing GrantsReadAccess for a permission type
///
/// This macro simplifies the implementation of compile-time permission checking.
///
/// # Example
/// ```ignore
/// impl_grants_read_access!(RestaurantPermissions, User, RestaurantDefinitions);
/// ```
#[macro_export]
macro_rules! impl_grants_read_access {
    ($permission:ty, $model:ty, $definition:ty) => {
        impl $crate::traits::permission::GrantsReadAccess<$model> for $permission {}
    };
}

/// Macro for implementing GrantsWriteAccess for a permission type
///
/// This macro simplifies the implementation of compile-time permission checking.
///
/// # Example
/// ```ignore
/// impl_grants_write_access!(RestaurantPermissions, User, RestaurantDefinitions);
/// ```
#[macro_export]
macro_rules! impl_grants_write_access {
    ($permission:ty, $model:ty, $definition:ty) => {
        impl $crate::traits::permission::GrantsWriteAccess<$model> for $permission {}
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_checker_is_zero_cost() {
        // This test verifies that PermissionChecker has no runtime overhead
        assert_eq!(
            std::mem::size_of::<PermissionChecker<(), (), ()>>(),
            0,
            "PermissionChecker should have zero size"
        );
    }
}
