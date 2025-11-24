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
    // Use the unified NetabaseStore API with Sled backend
    let store = NetabaseStore::<ExampleDefs, _>::sled(
        tempfile::tempdir()
            .expect("Failed to create temp dir")
            .path(),
    )
    .expect("The store failed to open");

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
}
