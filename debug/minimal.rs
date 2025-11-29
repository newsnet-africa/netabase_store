//! Minimal debug test to understand macro issue

use netabase_store::*;

#[netabase_definition_module(DebugDef, DebugKeys)]
mod debug_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode)]
    #[netabase(DebugDef)]
    pub struct SimpleUser {
        #[primary_key]
        pub id: u64,
        pub name: String,
    }
}

#[cfg(test)]
mod tests {
    use super::debug_schema::*;

    #[test]
    fn test_simple() {
        let user = SimpleUser {
            id: 1,
            name: "Test".to_string(),
        };
        println!("User: {:?}", user);
    }
}
