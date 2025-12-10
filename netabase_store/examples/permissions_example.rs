// This example demonstrates the permission system for multi-definition managers
//
// The permission system provides runtime permission checking that ensures
// transactions can only access definitions they have permission to use.
//
// Note: This is a conceptual example showing the permission system API.
// A complete working example would require full definition and model implementations.

use netabase_store::traits::permission::{
    PermissionEnumTrait, PermissionGrant, PermissionLevel,
};
use strum::{EnumDiscriminants, EnumIter, IntoDiscriminant};

// Example: Restaurant management system with multiple definitions
//
// This system has three definitions:
// - User: User accounts (customers, staff, managers)
// - Product: Menu items and inventory
// - Order: Customer orders

/// Restaurant definitions enum
///
/// In a real implementation, each variant would contain the actual definition type.
/// For this example, we use unit variants to demonstrate the permission system.
#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(Hash, EnumIter, strum::AsRefStr))]
#[strum_discriminants(name(RestaurantDefinitionsDiscriminants))]
pub enum RestaurantDefinitions {
    User,
    Product,
    Order,
}

/// Permission roles for the restaurant system
///
/// Each role has different levels of access to the three definitions:
/// - Manager: Full read/write access to all definitions
/// - Waiter: Read all, write orders only
/// - Customer: Read products, read/write their own orders only
#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Hash))]
pub enum RestaurantPermissions {
    /// Manager has full access to everything
    Manager {
        grant: PermissionGrant<RestaurantDefinitions>,
    },

    /// Waiter can read everything but only write orders
    Waiter {
        read_grant: PermissionGrant<RestaurantDefinitions>,
        write_grant: PermissionGrant<RestaurantDefinitions>,
    },

    /// Customer can only read products and manage their own orders
    Customer {
        grant: PermissionGrant<RestaurantDefinitions>,
    },
}

impl PermissionEnumTrait for RestaurantPermissions {
    fn permission_level(&self) -> PermissionLevel {
        match self {
            RestaurantPermissions::Manager { .. } => PermissionLevel::ReadWrite,
            RestaurantPermissions::Waiter { .. } => PermissionLevel::ReadWrite,
            RestaurantPermissions::Customer { .. } => PermissionLevel::ReadWrite,
        }
    }

    fn grants_access_to<R>(&self, definition: &R::Discriminant) -> bool
    where
        R: IntoDiscriminant,
        R::Discriminant: strum::IntoEnumIterator + std::hash::Hash + Eq + std::fmt::Debug + Clone,
    {
        // This is a simplified check - real implementation would need proper type handling
        match self {
            RestaurantPermissions::Manager { grant } => {
                // Manager has access to all definitions in their grant
                grant.definitions.iter().any(|d| format!("{:?}", d) == format!("{:?}", definition))
            }
            RestaurantPermissions::Waiter { read_grant, write_grant } => {
                // Waiter has access if it's in either read or write grant
                read_grant.definitions.iter().any(|d| format!("{:?}", d) == format!("{:?}", definition))
                    || write_grant.definitions.iter().any(|d| format!("{:?}", d) == format!("{:?}", definition))
            }
            RestaurantPermissions::Customer { grant } => {
                // Customer has limited access
                grant.definitions.iter().any(|d| format!("{:?}", d) == format!("{:?}", definition))
            }
        }
    }
}

impl RestaurantPermissions {
    /// Create a manager permission with full access
    pub fn manager() -> Self {
        RestaurantPermissions::Manager {
            grant: PermissionGrant::read_write(vec![
                RestaurantDefinitionsDiscriminants::User,
                RestaurantDefinitionsDiscriminants::Product,
                RestaurantDefinitionsDiscriminants::Order,
            ]),
        }
    }

    /// Create a waiter permission
    ///
    /// Waiters can:
    /// - Read: Users, Products, Orders
    /// - Write: Orders only
    pub fn waiter() -> Self {
        RestaurantPermissions::Waiter {
            read_grant: PermissionGrant::read_only(vec![
                RestaurantDefinitionsDiscriminants::User,
                RestaurantDefinitionsDiscriminants::Product,
                RestaurantDefinitionsDiscriminants::Order,
            ]),
            write_grant: PermissionGrant::write_only(vec![
                RestaurantDefinitionsDiscriminants::Order,
            ]),
        }
    }

    /// Create a customer permission
    ///
    /// Customers can:
    /// - Read: Products
    /// - Write: Orders (their own)
    pub fn customer() -> Self {
        RestaurantPermissions::Customer {
            grant: PermissionGrant::read_write(vec![
                RestaurantDefinitionsDiscriminants::Product,
                RestaurantDefinitionsDiscriminants::Order,
            ]),
        }
    }
}

fn main() {
    println!("Permission System Example");
    println!("========================\n");

    // Create different permission levels
    let manager_perm = RestaurantPermissions::manager();
    let waiter_perm = RestaurantPermissions::waiter();
    let customer_perm = RestaurantPermissions::customer();

    println!("1. Manager Permission");
    println!("   - Level: {:?}", manager_perm.permission_level());
    println!("   - Can access User: {}",
        manager_perm.grants_access_to::<RestaurantDefinitions>(
            &RestaurantDefinitionsDiscriminants::User
        )
    );
    println!("   - Can access Product: {}",
        manager_perm.grants_access_to::<RestaurantDefinitions>(
            &RestaurantDefinitionsDiscriminants::Product
        )
    );
    println!("   - Can access Order: {}\n",
        manager_perm.grants_access_to::<RestaurantDefinitions>(
            &RestaurantDefinitionsDiscriminants::Order
        )
    );

    println!("2. Waiter Permission");
    println!("   - Level: {:?}", waiter_perm.permission_level());
    println!("   - Can access User: {}",
        waiter_perm.grants_access_to::<RestaurantDefinitions>(
            &RestaurantDefinitionsDiscriminants::User
        )
    );
    println!("   - Can access Product: {}",
        waiter_perm.grants_access_to::<RestaurantDefinitions>(
            &RestaurantDefinitionsDiscriminants::Product
        )
    );
    println!("   - Can access Order: {}\n",
        waiter_perm.grants_access_to::<RestaurantDefinitions>(
            &RestaurantDefinitionsDiscriminants::Order
        )
    );

    println!("3. Customer Permission");
    println!("   - Level: {:?}", customer_perm.permission_level());
    println!("   - Can access User: {}",
        customer_perm.grants_access_to::<RestaurantDefinitions>(
            &RestaurantDefinitionsDiscriminants::User
        )
    );
    println!("   - Can access Product: {}",
        customer_perm.grants_access_to::<RestaurantDefinitions>(
            &RestaurantDefinitionsDiscriminants::Product
        )
    );
    println!("   - Can access Order: {}\n",
        customer_perm.grants_access_to::<RestaurantDefinitions>(
            &RestaurantDefinitionsDiscriminants::Order
        )
    );

    println!("\nUsage Pattern:");
    println!("==============\n");
    println!("With a full manager implementation, you would use permissions like this:\n");
    println!("```rust");
    println!("// Create manager with permission system");
    println!("let mut manager = RedbDefinitionManager::new(\"./restaurant_data\")?;");
    println!();
    println!("// Manager permission - can access all definitions");
    println!("manager.write(RestaurantPermissions::manager(), |txn| {{");
    println!("    // Access User definition");
    println!("    txn.definition_txn_mut(&RestaurantDefinitionsDiscriminants::User, |user_txn| {{");
    println!("        // Modify users...");
    println!("        Ok(())");
    println!("    }})?;");
    println!("    ");
    println!("    // Access Product definition");
    println!("    txn.definition_txn_mut(&RestaurantDefinitionsDiscriminants::Product, |product_txn| {{");
    println!("        // Modify products...");
    println!("        Ok(())");
    println!("    }})?;");
    println!("    ");
    println!("    Ok(())");
    println!("}});");
    println!();
    println!("// Customer permission - limited access");
    println!("manager.write(RestaurantPermissions::customer(), |txn| {{");
    println!("    // Can read products");
    println!("    txn.definition_txn(&RestaurantDefinitionsDiscriminants::Product, |product_txn| {{");
    println!("        // Read products... ✓");
    println!("        Ok(())");
    println!("    }})?;");
    println!("    ");
    println!("    // CANNOT access users - would return PermissionDenied error");
    println!("    // txn.definition_txn(&RestaurantDefinitionsDiscriminants::User, |user_txn| {{");
    println!("    //     // This would fail! ✗");
    println!("    // }})?;");
    println!("    ");
    println!("    Ok(())");
    println!("}});");
    println!("```\n");

    println!("Key Benefits:");
    println!("=============");
    println!("1. Type-safe permission checking at transaction level");
    println!("2. Clear separation of concerns - permissions are explicit");
    println!("3. Runtime enforcement prevents unauthorized access");
    println!("4. Flexible permission grants per role");
    println!("5. Future: Compile-time checking with marker traits (Phase 4+)");
}
