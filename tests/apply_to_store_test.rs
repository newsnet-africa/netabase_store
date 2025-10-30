//! Test for the macro-generated apply_to_store method
//!
//! This test verifies that the apply_to_store method is correctly generated
//! by the netabase_definition_module macro and can be used to apply Paxos
//! consensus entries to a RecordStore.

#![cfg(all(feature = "paxos", feature = "libp2p", feature = "sled"))]

use netabase_store::{netabase, netabase_definition_module, NetabaseModel};

// Define the schema with test models inside
#[netabase_definition_module(TestDefinition, TestDefinitionKeys)]
mod test_schema {
    use super::*;

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
    }
}

// Import the generated types
use test_schema::*;

#[test]
fn test_apply_to_store_method_exists() {
    // This test verifies that the apply_to_store method is generated

    use netabase_store::databases::sled_store::SledStore;

    // Create a SledStore
    let store = SledStore::<TestDefinition>::temp().unwrap();

    // Create test entries
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let post = Post {
        id: 100,
        title: "Hello World".to_string(),
        content: "This is a test post".to_string(),
        author_id: 1,
    };

    // Convert to Definition variants
    let user_entry = TestDefinition::User(user.clone());
    let post_entry = TestDefinition::Post(post.clone());

    // Apply entries to store using the generated method
    let mut record_store = store;

    // Test that apply_to_store method exists and is callable
    let result = user_entry.apply_to_store(&mut record_store);
    assert!(
        result.is_ok(),
        "Failed to apply user entry: {:?}",
        result.err()
    );

    let result = post_entry.apply_to_store(&mut record_store);
    assert!(
        result.is_ok(),
        "Failed to apply post entry: {:?}",
        result.err()
    );

    println!("✅ apply_to_store method works correctly!");
    println!("   - Applied User entry successfully");
    println!("   - Applied Post entry successfully");
}

#[test]
fn test_apply_to_store_actually_stores_data() {
    // This test verifies that apply_to_store actually persists data
    // that can be retrieved later

    use libp2p::kad::store::RecordStore;
    use libp2p::kad::RecordKey;
    use netabase_store::databases::sled_store::SledStore;

    let mut store = SledStore::<TestDefinition>::temp().unwrap();

    // Create and apply an entry
    let user = User {
        id: 42,
        name: "Bob".to_string(),
        email: "bob@test.com".to_string(),
    };

    let entry = TestDefinition::User(user.clone());
    entry.apply_to_store(&mut store).unwrap();

    // Verify the data was stored by retrieving it
    // The key format is: <discriminant>:<primary_key>
    use netabase_store::convert::ToIVec;
    let discriminant = "User";
    let key_bytes = user.id.to_ivec().unwrap();

    let mut record_key = Vec::new();
    record_key.extend_from_slice(discriminant.as_bytes());
    record_key.push(b':');
    record_key.extend_from_slice(&key_bytes);

    let retrieved = store.get(&RecordKey::new(&record_key));
    assert!(retrieved.is_some(), "Data was not persisted to store");

    // Verify the data matches
    let record = retrieved.unwrap();
    let decoded: User = bincode::decode_from_slice(&record.value, bincode::config::standard())
        .unwrap()
        .0;

    assert_eq!(decoded.id, user.id);
    assert_eq!(decoded.name, user.name);
    assert_eq!(decoded.email, user.email);

    println!("✅ apply_to_store persists data correctly!");
    println!("   - Data was stored");
    println!("   - Data was retrieved");
    println!("   - Data matches original");
}

#[test]
fn test_apply_to_store_idempotency() {
    // Verify that applying the same entry multiple times is safe

    use netabase_store::databases::sled_store::SledStore;

    let mut store = SledStore::<TestDefinition>::temp().unwrap();

    let user = User {
        id: 99,
        name: "Charlie".to_string(),
        email: "charlie@test.com".to_string(),
    };

    let entry = TestDefinition::User(user);

    // Apply the same entry multiple times
    for i in 0..5 {
        let result = entry.apply_to_store(&mut store);
        assert!(
            result.is_ok(),
            "Failed on iteration {}: {:?}",
            i,
            result.err()
        );
    }

    println!("✅ apply_to_store is idempotent!");
    println!("   - Applied same entry 5 times");
    println!("   - No errors occurred");
}
