//! Basic Store Example
//!
//! This example demonstrates basic CRUD operations with NetabaseStore.
//!
//! ## Backend Support:
//! This example works with both synchronous backends by changing one line:
//! - **Sled** (shown below): `NetabaseStore::sled("./my_db")?`
//! - **Redb**: `NetabaseStore::redb("./my_db.redb")?`
//! - **Temporary**: `NetabaseStore::temp()?` (uses default backend for testing)
//!
//! ## API Consistency:
//! Both sync backends (Sled, Redb) have identical APIs:
//! - `tree.put()`, `tree.get()`, `tree.remove()`
//! - `tree.get_by_secondary_key()` for secondary key queries
//! - Same data models, same return types
//!
//! Run with:
//! ```bash
//! cargo run --example basic_store --features native
//! ```

use netabase_store::traits::model::NetabaseModelTrait;
use netabase_store::{NetabaseStore, netabase_definition_module};

#[netabase_definition_module(ExampleDefs, ExampleDefKeys)]
pub mod definitions {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(
        NetabaseModel,
        bincode::Encode,
        bincode::Decode,
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(ExampleDefs)]
    pub struct User {
        #[primary_key]
        pub name: String,
        pub age: u8,
        #[secondary_key]
        pub email: String,
    }
}

use definitions::*;

fn main() {
    // Use the unified NetabaseStore API - easily switch backends!

    // Option 1: Sled backend (demonstrated here)
    let store = NetabaseStore::<ExampleDefs, _>::sled(
        tempfile::tempdir()
            .expect("Failed to create temp dir")
            .path(),
    )
    .expect("The store failed to open");

    // Option 2: Redb backend - same API, just change this line
    // let store = NetabaseStore::<ExampleDefs, _>::redb(
    //     tempfile::tempdir()
    //         .unwrap()
    //         .path()
    //         .join("db.redb"),
    // )
    // .expect("The store failed to open");

    // Option 3: Temporary store - same API, auto-cleanup
    // let store = NetabaseStore::<ExampleDefs, _>::temp()
    //     .expect("The store failed to open");

    // ✅ Everything below works identically on ALL backends!

    let user_tree = store.open_tree::<User>();

    let user = User {
        name: "It's You!".to_string(),
        age: 24,
        email: "some@email.com".to_string(),
    };
    let user2 = User {
        name: "It's Me!".to_string(),
        age: 20,
        email: "some@email.com".to_string(),
    };

    let put_result = user_tree.put(user.clone());

    let get_result = user_tree.get(user.primary_key());

    // Query by secondary key using the model-prefixed type
    let get_secondary_result = user_tree.get_by_secondary_key(UserSecondaryKeys::Email(
        UserEmailSecondaryKey("some@email.com".to_string()),
    ));

    println!("Get Result: {get_result:?}");
    println!("Get Secondary Result: {get_secondary_result:?}");

    assert!(put_result.is_ok());
    assert!(get_result.is_ok());

    let put_result = user_tree.put(user2.clone());

    let get_result = user_tree.get(user2.primary_key());

    // Query by secondary key using the model-prefixed type
    let get_secondary_result = user_tree.get_by_secondary_key(UserSecondaryKeys::Email(
        UserEmailSecondaryKey("some@email.com".to_string()),
    ));

    println!("Get Result: {get_result:?}");
    println!("Get Secondary Result: {get_secondary_result:?}");

    assert!(put_result.is_ok());
    assert!(get_result.is_ok());

    println!("\n✓ Basic store operations completed successfully!");
    println!("\n💡 Backend Compatibility:");
    println!("   • This example uses Sled, but works identically with:");
    println!("     - Redb: Change NetabaseStore::sled() to ::redb()");
    println!("     - Temporary: Use ::temp() for auto-cleanup testing");
    println!("   • All operations (put, get, query, iterate) are identical!");
    println!("   • See examples/batch_operations.rs and examples/transactions.rs");
}
