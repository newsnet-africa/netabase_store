//! Validates that all README code examples compile and work correctly.
//!
//! This test ensures the README is accurate by testing actual code patterns.

use netabase_store::prelude::*;
use netabase_store::traits::migration::MigrateFrom;
use serde::{Deserialize, Serialize};

// ============================================================================
// Test 1: Quick Start Example
// ============================================================================

#[netabase_macros::netabase_definition(QuickStartApp)]
mod quick_start_models {
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
    pub struct User {
        #[primary_key]
        pub id: String,

        pub name: String,

        #[secondary_key]
        pub email: String,
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
    pub struct Post {
        #[primary_key]
        pub id: String,

        pub title: String,
        pub content: String,

        #[link(QuickStartApp, User)]
        pub author: String,
    }
}

#[test]
fn test_readme_quick_start() -> Result<(), Box<dyn std::error::Error>> {
    use netabase_store::relational::RelationalLink;
    use quick_start_models::*;

    // Create an in-memory database
    let (store, _temp) = RedbStore::<QuickStartApp>::new_temporary()?;

    // Write data
    let txn = store.begin_write()?;
    txn.create(&User {
        id: UserID("alice".into()),
        name: "Alice Smith".into(),
        email: "alice@example.com".into(),
    })?;
    txn.create(&Post {
        id: PostID("post1".into()),
        title: "Hello World".into(),
        content: "My first post".into(),
        author: RelationalLink::new_dehydrated(UserID("alice".into())),
    })?;
    txn.commit()?;

    // Read data
    let txn = store.begin_read()?;
    let user: Option<User> = txn.read(&UserID("alice".into()))?;
    assert!(user.is_some());

    let post: Option<Post> = txn.read(&PostID("post1".into()))?;
    assert!(post.is_some());

    Ok(())
}

// ============================================================================
// Test 2: Models Example with Blob
// ============================================================================

#[netabase_macros::netabase_definition(ProductApp)]
mod product_models {
    use super::*;
    use netabase_store::blob::NetabaseBlobItem;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default, netabase_macros::NetabaseBlobItem)]
    pub struct ProductImage {
        pub data: Vec<u8>,
        pub mime_type: String,
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
    pub struct User {
        #[primary_key]
        pub id: String,
        pub name: String,
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
    pub struct Product {
        #[primary_key]
        pub sku: String,

        #[secondary_key]
        pub category: String,

        pub name: String,
        pub price: u64,

        #[link(ProductApp, User)]
        pub seller: String,

        #[blob]
        pub image: ProductImage,
    }
}

#[test]
fn test_readme_product_model() -> Result<(), Box<dyn std::error::Error>> {
    use netabase_store::relational::RelationalLink;
    use product_models::*;

    let (store, _temp) = RedbStore::<ProductApp>::new_temporary()?;

    let txn = store.begin_write()?;
    txn.create(&Product {
        sku: ProductID("SKU123".into()),
        category: "Electronics".into(),
        name: "Laptop".into(),
        price: 999,
        seller: RelationalLink::new_dehydrated(UserID("seller1".into())),
        image: product_models::ProductImage {
            data: vec![1, 2, 3, 4],
            mime_type: "image/png".into(),
        },
    })?;
    txn.commit()?;

    Ok(())
}

// ============================================================================
// Test 3: Migration Example
// ============================================================================

#[netabase_macros::netabase_definition(MigrationApp, subscriptions(Topic1))]
mod migration_models {
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
    #[netabase_version(family = "User", version = 1)]
    #[subscribe(Topic1)]
    pub struct UserV1 {
        #[primary_key]
        pub id: String,
        pub name: String,
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
    #[netabase_version(family = "User", version = 2, current)]
    #[subscribe(Topic1)]
    pub struct User {
        #[primary_key]
        pub id: String,
        pub first_name: String,
        pub last_name: String,
    }

    impl MigrateFrom<UserV1> for User {
        fn migrate_from(old: UserV1) -> Self {
            let parts: Vec<&str> = old.name.split_whitespace().collect();
            User {
                id: old.id,
                first_name: parts.get(0).unwrap_or(&"").to_string(),
                last_name: parts.get(1).unwrap_or(&"").to_string(),
            }
        }
    }
}

#[test]
fn test_readme_migration() {
    use migration_models::*;

    let old_user = UserV1 {
        id: UserID("user1".into()),
        name: "Alice Smith".into(),
    };

    let new_user = User::migrate_from(old_user);
    assert_eq!(new_user.first_name, "Alice");
    assert_eq!(new_user.last_name, "Smith");
}

// ============================================================================
// Test 4: Blob Storage Example (from Advanced Features section)
// ============================================================================

#[netabase_macros::netabase_definition(AvatarApp)]
mod avatar_models {
    use super::*;
    use netabase_store::blob::NetabaseBlobItem;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default, netabase_macros::NetabaseBlobItem)]
    pub struct ProfilePicture {
        pub data: Vec<u8>,
        pub mime_type: String,
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
    pub struct User {
        #[primary_key]
        pub id: String,

        #[blob]
        pub avatar: ProfilePicture,
    }
}

#[test]
fn test_readme_blob_storage() -> Result<(), Box<dyn std::error::Error>> {
    use avatar_models::*;

    let (store, _temp) = RedbStore::<AvatarApp>::new_temporary()?;

    let txn = store.begin_write()?;
    txn.create(&User {
        id: UserID("user1".into()),
        avatar: avatar_models::ProfilePicture {
            data: vec![0u8; 100_000], // Large blob
            mime_type: "image/jpeg".into(),
        },
    })?;
    txn.commit()?;

    Ok(())
}
