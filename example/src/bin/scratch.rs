use netabase_macros::netabase_definition;

#[netabase_definition(SomeGuy)]
pub mod some_guy {
    use netabase_macros::NetabaseModel;
    use redb::Value;

    #[derive(
        NetabaseModel,
        PartialEq,
        PartialOrd,
        Hash,
        Eq,
        Ord,
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
    )]
    pub struct TheThing {
        #[primary_key]
        pub id: u128,
    }
}

use some_guy::*;

pub fn main() {
    use redb::Value;

    let thing = TheThing {
        id: TheThingID(123),
    };
}
