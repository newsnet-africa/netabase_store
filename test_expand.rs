use serde::{Deserialize, Serialize};

#[netabase_macros::netabase_definition(MyDef)]
mod my_def {
    use super::*;

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
    pub struct Item {
        #[primary_key]
        pub id: String,
    }
}

fn main() {
    // Access the schema
    let schema = my_def::MyDef::schema();
    let toml = my_def::MyDef::export_toml();
}
