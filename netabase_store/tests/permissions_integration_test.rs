// Integration tests for the permission system
//
// These tests verify that runtime permission checking works correctly
// and that permissions are properly enforced at the transaction level.

use netabase_store::traits::permission::{
    PermissionEnumTrait, PermissionGrant, PermissionLevel,
};
use strum::{EnumDiscriminants, EnumIter, IntoDiscriminant};

/// Test definition enum
#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(Hash, EnumIter, strum::AsRefStr))]
#[strum_discriminants(name(TestDefinitionsDiscriminants))]
pub enum TestDefinitions {
    Users,
    Products,
    Orders,
}

/// Test permission enum
#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Hash))]
pub enum TestPermissions {
    Admin {
        grant: PermissionGrant<TestDefinitions>,
    },
    User {
        grant: PermissionGrant<TestDefinitions>,
    },
    Guest {
        grant: PermissionGrant<TestDefinitions>,
    },
}

impl PermissionEnumTrait for TestPermissions {
    fn permission_level(&self) -> PermissionLevel {
        match self {
            TestPermissions::Admin { .. } => PermissionLevel::ReadWrite,
            TestPermissions::User { .. } => PermissionLevel::ReadWrite,
            TestPermissions::Guest { .. } => PermissionLevel::Read,
        }
    }

    fn grants_access_to<R>(&self, definition: &R::Discriminant) -> bool
    where
        R: IntoDiscriminant,
        R::Discriminant: strum::IntoEnumIterator + std::hash::Hash + Eq + std::fmt::Debug + Clone,
    {
        // Check if the discriminant matches any in the grant
        match self {
            TestPermissions::Admin { grant } => {
                grant.definitions.iter().any(|d| format!("{:?}", d) == format!("{:?}", definition))
            }
            TestPermissions::User { grant } => {
                grant.definitions.iter().any(|d| format!("{:?}", d) == format!("{:?}", definition))
            }
            TestPermissions::Guest { grant } => {
                grant.definitions.iter().any(|d| format!("{:?}", d) == format!("{:?}", definition))
            }
        }
    }
}

#[test]
fn test_permission_levels() {
    let admin = TestPermissions::Admin {
        grant: PermissionGrant::read_write(vec![
            TestDefinitionsDiscriminants::Users,
            TestDefinitionsDiscriminants::Products,
            TestDefinitionsDiscriminants::Orders,
        ]),
    };

    let user = TestPermissions::User {
        grant: PermissionGrant::read_write(vec![
            TestDefinitionsDiscriminants::Products,
            TestDefinitionsDiscriminants::Orders,
        ]),
    };

    let guest = TestPermissions::Guest {
        grant: PermissionGrant::read_only(vec![
            TestDefinitionsDiscriminants::Products,
        ]),
    };

    assert_eq!(admin.permission_level(), PermissionLevel::ReadWrite);
    assert_eq!(user.permission_level(), PermissionLevel::ReadWrite);
    assert_eq!(guest.permission_level(), PermissionLevel::Read);
}

#[test]
fn test_admin_has_full_access() {
    let admin = TestPermissions::Admin {
        grant: PermissionGrant::read_write(vec![
            TestDefinitionsDiscriminants::Users,
            TestDefinitionsDiscriminants::Products,
            TestDefinitionsDiscriminants::Orders,
        ]),
    };

    assert!(admin.grants_access_to::<TestDefinitions>(&TestDefinitionsDiscriminants::Users));
    assert!(admin.grants_access_to::<TestDefinitions>(&TestDefinitionsDiscriminants::Products));
    assert!(admin.grants_access_to::<TestDefinitions>(&TestDefinitionsDiscriminants::Orders));

    // Check read/write permissions
    assert!(admin.can_read_definition::<TestDefinitions>(&TestDefinitionsDiscriminants::Users));
    assert!(admin.can_write_definition::<TestDefinitions>(&TestDefinitionsDiscriminants::Users));
    assert!(admin.can_read_definition::<TestDefinitions>(&TestDefinitionsDiscriminants::Products));
    assert!(admin.can_write_definition::<TestDefinitions>(&TestDefinitionsDiscriminants::Products));
}

#[test]
fn test_user_has_limited_access() {
    let user = TestPermissions::User {
        grant: PermissionGrant::read_write(vec![
            TestDefinitionsDiscriminants::Products,
            TestDefinitionsDiscriminants::Orders,
        ]),
    };

    // User should NOT have access to Users definition
    assert!(!user.grants_access_to::<TestDefinitions>(&TestDefinitionsDiscriminants::Users));
    assert!(!user.can_read_definition::<TestDefinitions>(&TestDefinitionsDiscriminants::Users));
    assert!(!user.can_write_definition::<TestDefinitions>(&TestDefinitionsDiscriminants::Users));

    // User should have access to Products and Orders
    assert!(user.grants_access_to::<TestDefinitions>(&TestDefinitionsDiscriminants::Products));
    assert!(user.can_read_definition::<TestDefinitions>(&TestDefinitionsDiscriminants::Products));
    assert!(user.can_write_definition::<TestDefinitions>(&TestDefinitionsDiscriminants::Products));

    assert!(user.grants_access_to::<TestDefinitions>(&TestDefinitionsDiscriminants::Orders));
    assert!(user.can_read_definition::<TestDefinitions>(&TestDefinitionsDiscriminants::Orders));
    assert!(user.can_write_definition::<TestDefinitions>(&TestDefinitionsDiscriminants::Orders));
}

#[test]
fn test_guest_has_read_only_access() {
    let guest = TestPermissions::Guest {
        grant: PermissionGrant::read_only(vec![
            TestDefinitionsDiscriminants::Products,
        ]),
    };

    // Guest can read Products
    assert!(guest.grants_access_to::<TestDefinitions>(&TestDefinitionsDiscriminants::Products));
    assert!(guest.can_read_definition::<TestDefinitions>(&TestDefinitionsDiscriminants::Products));

    // Guest CANNOT write to Products (read-only permission)
    assert!(!guest.can_write_definition::<TestDefinitions>(&TestDefinitionsDiscriminants::Products));

    // Guest cannot access Users or Orders at all
    assert!(!guest.grants_access_to::<TestDefinitions>(&TestDefinitionsDiscriminants::Users));
    assert!(!guest.grants_access_to::<TestDefinitions>(&TestDefinitionsDiscriminants::Orders));
}

#[test]
fn test_permission_grant_contains_check() {
    let grant: PermissionGrant<TestDefinitions> = PermissionGrant::read_write(vec![
        TestDefinitionsDiscriminants::Users,
        TestDefinitionsDiscriminants::Products,
    ]);

    assert!(grant.can_read(&TestDefinitionsDiscriminants::Users));
    assert!(grant.can_write(&TestDefinitionsDiscriminants::Users));
    assert!(grant.can_read_write(&TestDefinitionsDiscriminants::Users));

    assert!(grant.can_read(&TestDefinitionsDiscriminants::Products));
    assert!(grant.can_write(&TestDefinitionsDiscriminants::Products));

    // Orders is NOT in the grant
    assert!(!grant.can_read(&TestDefinitionsDiscriminants::Orders));
    assert!(!grant.can_write(&TestDefinitionsDiscriminants::Orders));
}

#[test]
fn test_read_only_grant() {
    let grant: PermissionGrant<TestDefinitions> = PermissionGrant::read_only(vec![
        TestDefinitionsDiscriminants::Products,
    ]);

    assert_eq!(grant.level(), PermissionLevel::Read);
    assert!(grant.can_read(&TestDefinitionsDiscriminants::Products));
    assert!(!grant.can_write(&TestDefinitionsDiscriminants::Products));
    assert!(!grant.can_read_write(&TestDefinitionsDiscriminants::Products));
}

#[test]
fn test_write_only_grant() {
    let grant: PermissionGrant<TestDefinitions> = PermissionGrant::write_only(vec![
        TestDefinitionsDiscriminants::Orders,
    ]);

    assert_eq!(grant.level(), PermissionLevel::Write);
    assert!(!grant.can_read(&TestDefinitionsDiscriminants::Orders));
    assert!(grant.can_write(&TestDefinitionsDiscriminants::Orders));
    assert!(!grant.can_read_write(&TestDefinitionsDiscriminants::Orders));
}

#[test]
fn test_no_permission_grant() {
    let grant: PermissionGrant<TestDefinitions> = PermissionGrant::none();

    assert_eq!(grant.level(), PermissionLevel::None);
    assert_eq!(grant.definitions().len(), 0);

    // No permissions granted for any definition
    assert!(!grant.can_read(&TestDefinitionsDiscriminants::Users));
    assert!(!grant.can_write(&TestDefinitionsDiscriminants::Users));
    assert!(!grant.can_read(&TestDefinitionsDiscriminants::Products));
    assert!(!grant.can_write(&TestDefinitionsDiscriminants::Products));
}

#[test]
fn test_permission_level_checks() {
    assert!(PermissionLevel::Read.can_read());
    assert!(!PermissionLevel::Read.can_write());
    assert!(!PermissionLevel::Read.can_read_write());

    assert!(!PermissionLevel::Write.can_read());
    assert!(PermissionLevel::Write.can_write());
    assert!(!PermissionLevel::Write.can_read_write());

    assert!(PermissionLevel::ReadWrite.can_read());
    assert!(PermissionLevel::ReadWrite.can_write());
    assert!(PermissionLevel::ReadWrite.can_read_write());

    assert!(!PermissionLevel::None.can_read());
    assert!(!PermissionLevel::None.can_write());
    assert!(!PermissionLevel::None.can_read_write());
}

#[test]
fn test_permission_level_ordering() {
    assert!(PermissionLevel::None < PermissionLevel::Read);
    assert!(PermissionLevel::Read < PermissionLevel::Write);
    assert!(PermissionLevel::Write < PermissionLevel::ReadWrite);

    assert!(PermissionLevel::ReadWrite > PermissionLevel::Write);
    assert!(PermissionLevel::Write > PermissionLevel::Read);
    assert!(PermissionLevel::Read > PermissionLevel::None);
}
