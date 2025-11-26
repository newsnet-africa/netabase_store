#![cfg(not(target_arch = "wasm32"))]
#![cfg(feature = "native")]
use blog::*;
use databases::sled_store::*;
use netabase_store::model::NetabaseModelTrait;
use netabase_store::{NetabaseModel, netabase_definition_module, *};

// Test schema
#[netabase_definition_module(BlogDefinition, BlogKeys)]
mod blog {
    use super::*;

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
    #[netabase(BlogDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub username: String,
        #[secondary_key]
        pub email: String,
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
    #[netabase(BlogDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
        #[secondary_key]
        pub published: bool,
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
    #[netabase(BlogDefinition)]
    pub struct Comment {
        #[primary_key]
        pub id: u64,
        pub post_id: u64,
        pub author: String,
        pub content: String,
    }
}

#[test]
fn test_sled_store_creation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let store = SledStore::<BlogDefinition>::new(temp_dir.path());
    assert!(store.is_ok());
}

#[test]
fn test_temp_store_creation() {
    let store = SledStore::<BlogDefinition>::temp();
    assert!(store.is_ok());
}

#[test]
fn test_put_and_get_user() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Insert user
    assert!(user_tree.put(alice.clone()).is_ok());

    // Retrieve user
    let retrieved = user_tree.get(UserPrimaryKey(1)).unwrap();
    assert_eq!(Some(alice), retrieved);
}

#[test]
fn test_remove_user() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Insert and remove
    user_tree.put(alice.clone()).unwrap();
    let removed = user_tree.remove(UserPrimaryKey(1)).unwrap();
    assert_eq!(Some(alice), removed);

    // Verify it's gone
    let retrieved = user_tree.get(UserPrimaryKey(1)).unwrap();
    assert_eq!(None, retrieved);
}

#[test]
fn test_multiple_models() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();

    // Insert users
    let user_tree = store.open_tree::<User>();
    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    let bob = User {
        id: 2,
        username: "bob".to_string(),
        email: "bob@example.com".to_string(),
    };
    user_tree.put(alice.clone()).unwrap();
    user_tree.put(bob.clone()).unwrap();

    // Insert posts
    let post_tree = store.open_tree::<Post>();
    let post1 = Post {
        id: 1,
        title: "Hello World".to_string(),
        content: "This is my first post".to_string(),
        author_id: 1,
        published: true,
    };
    let post2 = Post {
        id: 2,
        title: "Draft".to_string(),
        content: "Work in progress".to_string(),
        author_id: 2,
        published: false,
    };
    post_tree.put(post1.clone()).unwrap();
    post_tree.put(post2.clone()).unwrap();

    // Verify users
    assert_eq!(Some(alice), user_tree.get(UserPrimaryKey(1)).unwrap());
    assert_eq!(Some(bob), user_tree.get(UserPrimaryKey(2)).unwrap());

    // Verify posts
    assert_eq!(Some(post1), post_tree.get(PostPrimaryKey(1)).unwrap());
    assert_eq!(Some(post2), post_tree.get(PostPrimaryKey(2)).unwrap());
}

#[test]
fn test_iteration() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    let users = vec![
        User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
        },
        User {
            id: 3,
            username: "carol".to_string(),
            email: "carol@example.com".to_string(),
        },
    ];

    for user in &users {
        user_tree.put(user.clone()).unwrap();
    }

    let mut retrieved = Vec::new();
    for result in user_tree.iter() {
        let (_, user) = result.unwrap();
        retrieved.push(user);
    }

    assert_eq!(users.len(), retrieved.len());
    for user in &users {
        assert!(retrieved.contains(user));
    }
}

#[test]
fn test_tree_len_and_is_empty() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    assert!(user_tree.is_empty());
    assert_eq!(0, user_tree.len());

    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    user_tree.put(alice).unwrap();

    assert!(!user_tree.is_empty());
    assert_eq!(1, user_tree.len());
}

#[test]
fn test_clear() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    user_tree.put(alice).unwrap();

    assert_eq!(1, user_tree.len());

    user_tree.clear().unwrap();
    assert!(user_tree.is_empty());
}

#[test]
fn test_secondary_key_lookup() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    let bob = User {
        id: 2,
        username: "bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    user_tree.put(alice.clone()).unwrap();
    user_tree.put(bob.clone()).unwrap();

    // Look up by secondary key (email)
    let results = user_tree
        .get_by_secondary_key(UserSecondaryKeys::Email(UserEmailSecondaryKey(
            "alice@example.com".to_string(),
        )))
        .unwrap();

    assert_eq!(1, results.len());
    assert_eq!(alice, results[0]);
}

#[test]
fn test_secondary_key_multiple_results() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let post_tree = store.open_tree::<Post>();

    let post1 = Post {
        id: 1,
        title: "Post 1".to_string(),
        content: "Content 1".to_string(),
        author_id: 1,
        published: true,
    };
    let post2 = Post {
        id: 2,
        title: "Post 2".to_string(),
        content: "Content 2".to_string(),
        author_id: 1,
        published: true,
    };
    let post3 = Post {
        id: 3,
        title: "Post 3".to_string(),
        content: "Content 3".to_string(),
        author_id: 2,
        published: true,
    };

    post_tree.put(post1.clone()).unwrap();
    post_tree.put(post2.clone()).unwrap();
    post_tree.put(post3.clone()).unwrap();

    // Look up posts by author_id = 1
    let results = post_tree
        .get_by_secondary_key(PostSecondaryKeys::AuthorId(PostAuthorIdSecondaryKey(1)))
        .unwrap();

    // Should find 2 posts
    assert_eq!(2, results.len());
    assert!(results.contains(&post1));
    assert!(results.contains(&post2));
}

#[test]
fn test_update_model() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    user_tree.put(alice.clone()).unwrap();

    let updated_alice = User {
        id: 1,
        username: "alice_updated".to_string(),
        email: "alice_new@example.com".to_string(),
    };

    user_tree.put(updated_alice.clone()).unwrap();

    let retrieved = user_tree.get(UserPrimaryKey(1)).unwrap();
    assert_eq!(Some(updated_alice), retrieved);
}

#[test]
fn test_flush() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();

    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    user_tree.put(alice).unwrap();

    // Flush should succeed
    let result = store.flush();
    assert!(result.is_ok());
}

#[test]
fn test_trait_implementations() {
    let user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Test NetabaseModel trait
    let primary_key = user.primary_key();
    assert_eq!(UserPrimaryKey(1), primary_key);

    let secondary_keys = user.secondary_keys();
    assert_eq!(1, secondary_keys.len());

    assert_eq!("User", User::discriminant_name());
}

// ============================================================================
// Transaction Tests
// ============================================================================

#[test]
fn test_transaction_basic() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();

    // Insert users in a single transaction
    store
        .transaction::<User, _, _>(|txn_tree| {
            let alice = User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
            };
            let bob = User {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
            };

            txn_tree.put(alice)?;
            txn_tree.put(bob)?;

            Ok(())
        })
        .unwrap();

    // Verify both users exist
    let user_tree = store.open_tree::<User>();
    let alice = user_tree.get(UserPrimaryKey(1)).unwrap();
    let bob = user_tree.get(UserPrimaryKey(2)).unwrap();

    assert!(alice.is_some());
    assert!(bob.is_some());
    assert_eq!("alice", alice.unwrap().username);
    assert_eq!("bob", bob.unwrap().username);
}

#[test]
fn test_transaction_read() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();

    // Setup: Insert a user outside transaction
    let user_tree = store.open_tree::<User>();
    user_tree
        .put(User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        })
        .unwrap();

    // Read within transaction
    let result = store
        .transaction::<User, _, _>(|txn_tree| {
            let user = txn_tree.get(UserPrimaryKey(1))?;
            Ok(user.map(|u| u.username))
        })
        .unwrap();

    assert_eq!(Some("alice".to_string()), result);
}

#[test]
fn test_transaction_remove() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();

    // Setup: Insert a user
    let user_tree = store.open_tree::<User>();
    user_tree
        .put(User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        })
        .unwrap();

    // Remove within transaction
    store
        .transaction::<User, _, _>(|txn_tree| {
            txn_tree.remove(UserPrimaryKey(1))?;
            Ok(())
        })
        .unwrap();

    // Verify user is gone
    let result = user_tree.get(UserPrimaryKey(1)).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_transaction_bulk_insert() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();

    // Insert 100 users in a single transaction
    store
        .transaction::<User, _, _>(|txn_tree| {
            for i in 0..100 {
                let user = User {
                    id: i,
                    username: format!("user{}", i),
                    email: format!("user{}@example.com", i),
                };
                txn_tree.put(user)?;
            }
            Ok(())
        })
        .unwrap();

    // Verify all users exist
    let user_tree = store.open_tree::<User>();
    assert_eq!(100, user_tree.len());

    // Spot check a few users
    let user0 = user_tree.get(UserPrimaryKey(0)).unwrap().unwrap();
    assert_eq!("user0", user0.username);

    let user50 = user_tree.get(UserPrimaryKey(50)).unwrap().unwrap();
    assert_eq!("user50", user50.username);

    let user99 = user_tree.get(UserPrimaryKey(99)).unwrap().unwrap();
    assert_eq!("user99", user99.username);
}

#[test]
fn test_transaction_update() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();

    // Setup: Insert a user
    let user_tree = store.open_tree::<User>();
    user_tree
        .put(User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        })
        .unwrap();

    // Update within transaction
    store
        .transaction::<User, _, _>(|txn_tree| {
            let mut user = txn_tree.get(UserPrimaryKey(1))?.unwrap();
            user.username = "alice_updated".to_string();
            user.email = "alice_new@example.com".to_string();
            txn_tree.put(user)?;
            Ok(())
        })
        .unwrap();

    // Verify update
    let updated = user_tree.get(UserPrimaryKey(1)).unwrap().unwrap();
    assert_eq!("alice_updated", updated.username);
    assert_eq!("alice_new@example.com", updated.email);
}

#[test]
fn test_transaction_atomic_transfer() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();

    // Setup: Create two posts with different author_ids
    let post_tree = store.open_tree::<Post>();
    post_tree
        .put(Post {
            id: 1,
            title: "Post 1".to_string(),
            content: "Content 1".to_string(),
            author_id: 1,
            published: true,
        })
        .unwrap();

    post_tree
        .put(Post {
            id: 2,
            title: "Post 2".to_string(),
            content: "Content 2".to_string(),
            author_id: 2,
            published: false,
        })
        .unwrap();

    // Transaction: Atomically swap author_ids
    store
        .transaction::<Post, _, _>(|txn_tree| {
            let mut post1 = txn_tree.get(PostPrimaryKey(1))?.unwrap();
            let mut post2 = txn_tree.get(PostPrimaryKey(2))?.unwrap();

            // Swap author_ids
            let temp_author = post1.author_id;
            post1.author_id = post2.author_id;
            post2.author_id = temp_author;

            txn_tree.put(post1)?;
            txn_tree.put(post2)?;

            Ok(())
        })
        .unwrap();

    // Verify swap
    let post1 = post_tree.get(PostPrimaryKey(1)).unwrap().unwrap();
    let post2 = post_tree.get(PostPrimaryKey(2)).unwrap().unwrap();

    assert_eq!(2, post1.author_id);
    assert_eq!(1, post2.author_id);
}

#[test]
fn test_transaction_return_value() {
    let store = SledStore::<BlogDefinition>::temp().unwrap();

    // Setup
    let user_tree = store.open_tree::<User>();
    for i in 0..10 {
        user_tree
            .put(User {
                id: i,
                username: format!("user{}", i),
                email: format!("user{}@example.com", i),
            })
            .unwrap();
    }

    // Transaction that returns data
    let usernames: Vec<String> = store
        .transaction::<User, _, _>(|txn_tree| {
            let mut names = Vec::new();
            for i in 0..10 {
                if let Some(user) = txn_tree.get(UserPrimaryKey(i))? {
                    names.push(user.username);
                }
            }
            Ok(names)
        })
        .unwrap();

    assert_eq!(10, usernames.len());
    assert_eq!("user0", usernames[0]);
    assert_eq!("user9", usernames[9]);
}
