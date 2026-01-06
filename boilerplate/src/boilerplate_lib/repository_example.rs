//! Repository-based boilerplate library demonstrating repository isolation.
//!
//! This module shows how definitions can be grouped into repositories,
//! with compile-time enforcement of data graph completeness.
//!
//! # Architecture
//!
//! Definitions register themselves to repositories using `repos(...)`.
//! Multiple repositories can be defined on a single shared module,
//! each collecting definitions that subscribe to it.
//!
//! ```text
//! EmployeeRepo
//! ├── Employee (User, Shift)
//! └── Inventory (Product)
//!
//! ManagerRepo  
//! ├── Employee (User, Shift) - same definition, different context
//! └── Reports (Report)
//! ```
//!
//! # Examples
//!
//! Creating an employee user:
//!
//! ```
//! use netabase_store_examples::repository_example::{User, UserID, Shift, ShiftID, Product, Report};
//! use netabase_store::relational::RelationalLink;
//!
//! let employee = User {
//!     id: UserID("emp_001".to_string()),
//!     name: "John Doe".to_string(),
//!     email: "john@example.com".to_string(),
//!     department: "Sales".to_string(),
//! };
//!
//! let shift = Shift {
//!     id: ShiftID("shift_001".to_string()),
//!     date: "2023-01-01".to_string(),
//!     employee_id: RelationalLink::new_dehydrated(UserID("emp_001".to_string())),
//!     hours: 8,
//! };
//!
//! assert_eq!(employee.department, "Sales");
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// Shared Module with Multiple Repositories
// ============================================================================

/// Employee repository - accessible by all employees
/// Contains: Employee (User, Shift), Inventory (Product)
#[netabase_macros::netabase_repository(EmployeeRepo)]
/// Manager repository - accessible only by managers  
/// Contains: Employee (User, Shift), Reports (Report)
#[netabase_macros::netabase_repository(ManagerRepo)]
pub mod shared_definitions {
    use super::*;

    // ========================================================================
    // Employee Definition - registered to both repositories
    // ========================================================================

    #[netabase_macros::netabase_definition(Employee, repos(EmployeeRepo, ManagerRepo))]
    pub mod employee {
        use super::*;

        /// A user/employee in the system
        #[derive(
            netabase_macros::NetabaseModel,
            Debug,
            Clone,
            Serialize,
            Deserialize,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
        )]
        pub struct User {
            #[primary_key]
            pub id: String,

            #[secondary_key]
            pub name: String,

            #[secondary_key]
            pub email: String,

            pub department: String,
        }

        /// A work shift
        #[derive(
            netabase_macros::NetabaseModel,
            Debug,
            Clone,
            Serialize,
            Deserialize,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
        )]
        pub struct Shift {
            #[primary_key]
            pub id: String,

            #[secondary_key]
            pub date: String,

            /// Links to the employee working this shift (same definition)
            #[link(Employee, User)]
            pub employee_id: String,

            pub hours: u8,
        }
    }

    // ========================================================================
    // Inventory Definition - registered only to EmployeeRepo
    // ========================================================================

    #[netabase_macros::netabase_definition(Inventory, repos(EmployeeRepo))]
    pub mod inventory {
        use super::employee::{Employee, User, UserID};
        use super::*;

        /// A product in inventory
        #[derive(
            netabase_macros::NetabaseModel,
            Debug,
            Clone,
            Serialize,
            Deserialize,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
        )]
        pub struct Product {
            #[primary_key]
            pub sku: String,

            #[secondary_key]
            pub name: String,

            #[secondary_key]
            pub category: String,

            pub price_cents: u64,

            pub stock_count: u32,

            /// Links to the employee who last updated this product
            #[link(Employee, User)]
            pub last_updated_by: String,
        }
    }

    // ========================================================================
    // Reports Definition - registered only to ManagerRepo
    // ========================================================================

    #[netabase_macros::netabase_definition(Reports, repos(ManagerRepo))]
    pub mod reports {
        use super::employee::{Employee, User, UserID};
        use super::*;

        /// A report generated by a manager
        #[derive(
            netabase_macros::NetabaseModel,
            Debug,
            Clone,
            Serialize,
            Deserialize,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
        )]
        pub struct Report {
            #[primary_key]
            pub id: String,

            #[secondary_key]
            pub title: String,

            #[secondary_key]
            pub report_date: String,

            /// Links to the manager who created this report
            #[link(Employee, User)]
            pub created_by: String,

            pub content: String,

            pub is_published: bool,
        }
    }
}

// ============================================================================
// Re-exports
// ============================================================================

pub use shared_definitions::employee::*;
pub use shared_definitions::inventory::*;
pub use shared_definitions::reports::*;

// Re-export repository types
pub use shared_definitions::EmployeeRepo;
pub use shared_definitions::ManagerRepo;
