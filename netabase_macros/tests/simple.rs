use netabase_macros::netabase_definition_module;

pub trait NetabaseModel {}

#[netabase_definition_module(TestDef, TestKeys)]
pub mod test_thing {
    use netabase_macros::{NetabaseModel, netabase};
    #[derive(NetabaseModel)]
    #[netabase(TestDef)]
    struct NewThing {
        #[primary_key]
        hi: String,
        #[secondary_key]
        you_key: String,
        #[secondary_key]
        there: String,
    }
}
