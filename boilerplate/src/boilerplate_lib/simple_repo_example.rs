#[netabase_macros::netabase_definition(SimpleDefinition)]
pub mod simple_definition {
    use netabase_store::{relational::RelationalLink, traits::registery::models::keys::primary};
    use serde::{Deserialize, Serialize};

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
        #[primary_key]
        id: u128,
        #[secondary_key]
        name: String,
        warehouse: String,
    }

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
        #[primary_key]
        branch_id: u128,
        items: Vec<InventoryItemID>,
        #[link(SimpleDefinition, InventoryItem)]
        one_item: u128,
    }
}
