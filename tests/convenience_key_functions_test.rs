/// Integration tests for convenience extension traits for secondary keys
///
/// These tests verify that the generated extension traits provide
/// ergonomic APIs for creating secondary keys.
use netabase_store::netabase_definition_module;

// Define a test schema with various secondary key types
#[netabase_definition_module(TestDef, TestKeys)]
mod test_models {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDef)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
        #[secondary_key]
        pub age: u32,
        #[secondary_key]
        pub active: bool,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDef)]
    pub struct Product {
        #[primary_key]
        pub id: String,
        pub name: String,
        #[secondary_key]
        pub category: String,
        #[secondary_key]
        pub price: u64,
    }
}

use test_models::*;

#[test]
#[cfg(feature = "sled")]
fn test_string_secondary_key_from_str() {
    use netabase_store::databases::sled_store::SledStore;

    let temp_dir = tempfile::tempdir().unwrap();
    let store = SledStore::<TestDef>::new(temp_dir.path().join("test.db")).unwrap();
    let tree = store.open_tree::<User>();

    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
        active: true,
    };

    tree.put(user.clone()).unwrap();

    // Old verbose API
    let _by_email_old = tree
        .get_by_secondary_key(UserSecondaryKeys::Email(UserEmailSecondaryKey(
            "alice@example.com".to_string(),
        )))
        .unwrap();

    // New ergonomic API using convenience trait
    use test_models::AsUserEmail;
    let by_email_new = tree
        .get_by_secondary_key("alice@example.com".as_user_email_key())
        .unwrap();

    assert_eq!(by_email_new.len(), 1);
    assert_eq!(by_email_new[0], user);
}

#[test]
#[cfg(feature = "sled")]
fn test_string_secondary_key_from_string() {
    use netabase_store::databases::sled_store::SledStore;

    let temp_dir = tempfile::tempdir().unwrap();
    let store = SledStore::<TestDef>::new(temp_dir.path().join("test.db")).unwrap();
    let tree = store.open_tree::<User>();

    let user = User {
        id: 1,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
        age: 25,
        active: true,
    };

    tree.put(user.clone()).unwrap();

    // Test with owned String
    use test_models::AsUserEmail;
    let email = "bob@example.com".to_string();
    let by_email = tree
        .get_by_secondary_key(email.as_user_email_key())
        .unwrap();

    assert_eq!(by_email.len(), 1);
    assert_eq!(by_email[0], user);
}

#[test]
#[cfg(feature = "sled")]
fn test_numeric_secondary_key() {
    use netabase_store::databases::sled_store::SledStore;

    let temp_dir = tempfile::tempdir().unwrap();
    let store = SledStore::<TestDef>::new(temp_dir.path().join("test.db")).unwrap();
    let tree = store.open_tree::<User>();

    let user = User {
        id: 1,
        name: "Charlie".to_string(),
        email: "charlie@example.com".to_string(),
        age: 35,
        active: false,
    };

    tree.put(user.clone()).unwrap();

    // Use convenience trait for numeric type
    use test_models::AsUserAge;
    let by_age = tree.get_by_secondary_key(35u32.as_user_age_key()).unwrap();

    assert_eq!(by_age.len(), 1);
    assert_eq!(by_age[0], user);
}

#[test]
#[cfg(feature = "sled")]
fn test_bool_secondary_key() {
    use netabase_store::databases::sled_store::SledStore;

    let temp_dir = tempfile::tempdir().unwrap();
    let store = SledStore::<TestDef>::new(temp_dir.path().join("test.db")).unwrap();
    let tree = store.open_tree::<User>();

    let active_user = User {
        id: 1,
        name: "Active".to_string(),
        email: "active@example.com".to_string(),
        age: 30,
        active: true,
    };

    let inactive_user = User {
        id: 2,
        name: "Inactive".to_string(),
        email: "inactive@example.com".to_string(),
        age: 25,
        active: false,
    };

    tree.put(active_user.clone()).unwrap();
    tree.put(inactive_user.clone()).unwrap();

    // Use convenience trait for bool type
    use test_models::AsUserActive;
    let active_users = tree
        .get_by_secondary_key(true.as_user_active_key())
        .unwrap();

    assert_eq!(active_users.len(), 1);
    assert_eq!(active_users[0], active_user);
}

#[test]
#[cfg(feature = "sled")]
fn test_multiple_models_different_traits() {
    use netabase_store::databases::sled_store::SledStore;

    let temp_dir = tempfile::tempdir().unwrap();
    let store = SledStore::<TestDef>::new(temp_dir.path().join("test.db")).unwrap();

    // Test User model
    let user_tree = store.open_tree::<User>();
    let user = User {
        id: 1,
        name: "Dave".to_string(),
        email: "dave@example.com".to_string(),
        age: 40,
        active: true,
    };
    user_tree.put(user.clone()).unwrap();

    // Test Product model
    let product_tree = store.open_tree::<Product>();
    let product = Product {
        id: "prod-1".to_string(),
        name: "Widget".to_string(),
        category: "Electronics".to_string(),
        price: 1999,
    };
    product_tree.put(product.clone()).unwrap();

    // Use extension traits for both models
    use test_models::{AsProductCategory, AsUserEmail};

    let users_by_email = user_tree
        .get_by_secondary_key("dave@example.com".as_user_email_key())
        .unwrap();

    let products_by_category = product_tree
        .get_by_secondary_key("Electronics".as_product_category_key())
        .unwrap();

    assert_eq!(users_by_email[0], user);
    assert_eq!(products_by_category[0], product);
}

#[test]
#[cfg(feature = "sled")]
fn test_convenience_vs_verbose_equivalence() {
    use netabase_store::databases::sled_store::SledStore;

    let temp_dir = tempfile::tempdir().unwrap();
    let store = SledStore::<TestDef>::new(temp_dir.path().join("test.db")).unwrap();
    let tree = store.open_tree::<User>();

    let user1 = User {
        id: 1,
        name: "Test1".to_string(),
        email: "test1@example.com".to_string(),
        age: 30,
        active: true,
    };

    let user2 = User {
        id: 2,
        name: "Test2".to_string(),
        email: "test2@example.com".to_string(),
        age: 30,
        active: false,
    };

    tree.put(user1.clone()).unwrap();
    tree.put(user2.clone()).unwrap();

    // Verbose API
    let by_age_verbose = tree
        .get_by_secondary_key(UserSecondaryKeys::Age(UserAgeSecondaryKey(30)))
        .unwrap();

    // Convenience API
    use test_models::AsUserAge;
    let by_age_convenience = tree.get_by_secondary_key(30u32.as_user_age_key()).unwrap();

    // Both should return the same results
    assert_eq!(by_age_verbose.len(), 2);
    assert_eq!(by_age_convenience.len(), 2);
    assert_eq!(by_age_verbose, by_age_convenience);
}

#[test]
#[cfg(feature = "sled")]
fn test_reference_types() {
    use netabase_store::databases::sled_store::SledStore;

    let temp_dir = tempfile::tempdir().unwrap();
    let store = SledStore::<TestDef>::new(temp_dir.path().join("test.db")).unwrap();
    let tree = store.open_tree::<User>();

    let user = User {
        id: 1,
        name: "Reference".to_string(),
        email: "ref@example.com".to_string(),
        age: 42,
        active: true,
    };

    tree.put(user.clone()).unwrap();

    // Test with &str
    use test_models::AsUserEmail;
    let email_str: &str = "ref@example.com";
    let by_email = tree
        .get_by_secondary_key(email_str.as_user_email_key())
        .unwrap();
    assert_eq!(by_email[0], user);

    // Test with &String
    let email_string = "ref@example.com".to_string();
    let by_email2 = tree
        .get_by_secondary_key((&email_string).as_user_email_key())
        .unwrap();
    assert_eq!(by_email2[0], user);

    // Test with &u32
    use test_models::AsUserAge;
    let age: u32 = 42;
    let by_age = tree.get_by_secondary_key((&age).as_user_age_key()).unwrap();
    assert_eq!(by_age[0], user);
}
