use netabase_macros::netabase_definition_module;

#[netabase_definition_module(A, B)]
pub mod definitions {
    use netabase_macros::NetabaseModel;
    use netabase_macros::NetabaseModelKey;
    #[derive(NetabaseModel, Debug, bincode::Encode, bincode::Decode)]
    pub struct NewThing {
        #[primary_key]
        hi: String,
        #[secondary_key]
        you_key: String,
        #[secondary_key]
        there: String,
    }
    pub mod inner {
        use netabase_macros::NetabaseModel;
        use netabase_macros::NetabaseModelKey;
        #[derive(NetabaseModel, Debug, bincode::Encode, bincode::Decode)]
        pub struct AnotherNewThing {
            #[primary_key]
            second_hi: String,
            #[secondary_key]
            you_key: String,
            #[secondary_key]
            there: String,
            just_cause: i32,
        }
    }
}
