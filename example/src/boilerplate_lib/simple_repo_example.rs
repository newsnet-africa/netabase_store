//! Simple definition demonstrating basic model relationships.
//!
//! This module provides a minimal example of defining models with primary keys,
//! secondary keys, and relational links. It's useful as a starting point for
//! understanding the netabase_store macro system.
//!
//! # Models
//!
//! - `InventoryItem`: Basic inventory record with name indexing
//! - `BranchInventory`: Branch-level inventory with links to items
//!
//! # Example
//!
//! ```rust
//! use example::simple_definition::*;
//! use netabase_store::relational::RelationalLink;
//!
//! // Create an inventory item
//! let item = InventoryItem {
//!     id: InventoryItemID(12345),
//!     name: "Widget".to_string(),
//!     warehouse: "Main".to_string(),
//! };
//!
//! // Create a branch inventory that links to the item
//! let branch = BranchInventory {
//!     branch_id: BranchInventoryID(1),
//!     items: vec![InventoryItemID(12345)],
//!     one_item: RelationalLink::new_dehydrated(InventoryItemID(12345)),
//! };
//!
//! // The types are created successfully
//! assert!(item.id.0 == 12345);
//! ```

#[netabase_macros::netabase_definition(SimpleDefinition)]
pub mod simple_definition {
    use netabase_store::{relational::RelationalLink, traits::registery::models::keys::primary};
    use serde::{Deserialize, Serialize};

    /// An item in inventory with name-based indexing.
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
    pub struct InventoryItem {
        /// Unique inventory item identifier
        #[primary_key]
        pub id: u128,

        /// Item name - indexed for fast lookup
        #[secondary_key]
        pub name: String,

        /// Warehouse location
        pub warehouse: String,
    }

    /// Inventory tracking for a specific branch location.
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
    pub struct BranchInventory {
        /// Unique branch identifier
        #[primary_key]
        pub branch_id: u128,

        /// List of inventory item IDs at this branch
        pub items: Vec<InventoryItemID>,

        /// Example link to a specific inventory item
        #[link(SimpleDefinition, InventoryItem)]
        pub one_item: u128,
    }
}
