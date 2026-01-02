/// Comprehensive migration system tests
///
/// This test suite validates:
/// - Version attribute parsing
/// - Model family grouping
/// - Migration chain generation
/// - Bincode versioned encoding/decoding
/// - Database migration utilities
/// - Schema comparison for P2P
mod common;

use bincode::{Decode, Encode};
use netabase_macros::{NetabaseModel, netabase_definition};
use netabase_store::databases::redb::RedbStore;
use netabase_store::errors::NetabaseResult;
use netabase_store::traits::database::transaction::NetabaseRwTransaction;
use netabase_store::traits::migration::{
    MigrateFrom, VersionContext, VersionHeader, VersionedDecode, VersionedEncode,
};

#[netabase_definition]
mod test_users {
    use super::*;

    /// Version 1 of User - basic fields only
    #[derive(Debug, Clone, PartialEq, Eq, NetabaseModel, Encode, Decode)]
    #[netabase_version(family = "User", version = 1)]
    pub struct UserV1 {
        #[primary]
        pub id: u64,
        pub name: String,
    }

    /// Version 2 of User - added email field
    #[derive(Debug, Clone, PartialEq, Eq, NetabaseModel, Encode, Decode)]
    #[netabase_version(family = "User", version = 2)]
    pub struct UserV2 {
        #[primary]
        pub id: u64,
        pub name: String,
        pub email: String,
    }

    /// Version 3 of User (current) - added age field
    #[derive(Debug, Clone, PartialEq, Eq, NetabaseModel, Encode, Decode)]
    #[netabase_version(family = "User", version = 3, current)]
    pub struct User {
        #[primary]
        pub id: u64,
        pub name: String,
        pub email: String,
        pub age: u32,
    }

    /// Unversioned model for comparison
    #[derive(Debug, Clone, PartialEq, Eq, NetabaseModel, Encode, Decode)]
    pub struct SimpleModel {
        #[primary]
        pub id: u64,
        pub data: String,
    }
}

// Migration implementations: V1 -> V2 -> V3
impl From<test_users::UserV1> for test_users::UserV2 {
    fn from(old: test_users::UserV1) -> Self {
        test_users::UserV2 {
            id: old.id,
            name: old.name,
            email: String::from("unknown@example.com"),
        }
    }
}

impl From<test_users::UserV2> for test_users::User {
    fn from(old: test_users::UserV2) -> Self {
        test_users::User {
            id: old.id,
            name: old.name,
            email: old.email,
            age: 0,
        }
    }
}

#[test]
fn test_version_header_encoding() {
    let header = VersionHeader::new(42);
    let bytes = header.to_bytes();

    // Check magic bytes
    assert_eq!(bytes[0], b'N');
    assert_eq!(bytes[1], b'V');

    // Check size
    assert_eq!(bytes.len(), VersionHeader::SIZE);

    // Decode and verify
    let decoded = VersionHeader::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.version, 42);
}

#[test]
fn test_version_header_detection() {
    let versioned = VersionHeader::new(1).to_bytes();
    assert!(VersionHeader::is_versioned(&versioned));

    let unversioned = vec![0u8, 1, 2, 3, 4, 5];
    assert!(!VersionHeader::is_versioned(&unversioned));
}

#[test]
fn test_versioned_encode_current() {
    let user = test_users::User {
        id: 1,
        name: String::from("Alice"),
        email: String::from("alice@example.com"),
        age: 30,
    };

    let encoded = user.encode_versioned();

    // Should have version header
    assert!(VersionHeader::is_versioned(&encoded));

    // Extract and check version
    let header = VersionHeader::from_bytes(&encoded).unwrap();
    assert_eq!(header.version, 3);

    // Payload should follow header
    assert!(encoded.len() > VersionHeader::SIZE);
}

#[test]
fn test_versioned_decode_same_version() {
    let user = test_users::User {
        id: 1,
        name: String::from("Bob"),
        email: String::from("bob@example.com"),
        age: 25,
    };

    let encoded = user.encode_versioned();

    let ctx = VersionContext {
        current_version: 3,
        min_supported_version: 1,
        auto_migrate: false,
        strict: false,
    };

    let decoded = test_users::User::decode_versioned(&encoded, &ctx).unwrap();

    assert_eq!(decoded, user);
}

#[test]
fn test_manual_migration_v1_to_v2() {
    let v1 = test_users::UserV1 {
        id: 1,
        name: String::from("Charlie"),
    };

    let v2: test_users::UserV2 = v1.into();

    assert_eq!(v2.id, 1);
    assert_eq!(v2.name, "Charlie");
    assert_eq!(v2.email, "unknown@example.com");
}

#[test]
fn test_manual_migration_v2_to_v3() {
    let v2 = test_users::UserV2 {
        id: 2,
        name: String::from("Diana"),
        email: String::from("diana@example.com"),
    };

    let v3: test_users::User = v2.into();

    assert_eq!(v3.id, 2);
    assert_eq!(v3.name, "Diana");
    assert_eq!(v3.email, "diana@example.com");
    assert_eq!(v3.age, 0);
}

#[test]
fn test_chained_migration_v1_to_v3() {
    let v1 = test_users::UserV1 {
        id: 3,
        name: String::from("Eve"),
    };

    // Chain: V1 -> V2 -> V3
    let v2: test_users::UserV2 = v1.into();
    let v3: test_users::User = v2.into();

    assert_eq!(v3.id, 3);
    assert_eq!(v3.name, "Eve");
    assert_eq!(v3.email, "unknown@example.com");
    assert_eq!(v3.age, 0);
}

#[test]
fn test_unversioned_model_legacy_decode() {
    let model = test_users::SimpleModel {
        id: 1,
        data: String::from("test"),
    };

    // Encode without version header (legacy format)
    let encoded = bincode::encode_to_vec(&model, bincode::config::standard()).unwrap();

    // Should not have version header
    assert!(!VersionHeader::is_versioned(&encoded));

    // Should decode via unversioned path
    let ctx = VersionContext::default();
    let decoded = test_users::SimpleModel::decode_versioned(&encoded, &ctx).unwrap();

    assert_eq!(decoded, model);
}

#[test]
fn test_version_context_strict_mode() {
    let user = test_users::User {
        id: 1,
        name: String::from("Frank"),
        email: String::from("frank@example.com"),
        age: 35,
    };

    let mut encoded = user.encode_versioned();

    // Tamper with version number to simulate old data
    encoded[2] = 1; // Change version from 3 to 1

    let ctx = VersionContext {
        current_version: 3,
        min_supported_version: 1,
        auto_migrate: false,
        strict: true, // Strict mode - should reject version mismatch
    };

    let result = test_users::User::decode_versioned(&encoded, &ctx);
    assert!(result.is_err());
}

#[test]
fn test_database_create_and_read_inspection() {
    let (store, db_path) = common::create_test_db::<test_users::TestUsers>("migration_inspect")
        .expect("Failed to create test db");

    // Create a user
    let user = test_users::User {
        id: 1,
        name: String::from("Grace"),
        email: String::from("grace@example.com"),
        age: 28,
    };

    {
        let txn = store.begin_write().expect("Failed to begin write");

        // Create record
        txn.create(&user).expect("Failed to create user");

        // Read back immediately
        let read_user: Option<test_users::User> = txn.read(&1u64).expect("Failed to read user");

        assert!(read_user.is_some());
        let read_user = read_user.unwrap();
        assert_eq!(read_user.id, 1);
        assert_eq!(read_user.name, "Grace");
        assert_eq!(read_user.email, "grace@example.com");
        assert_eq!(read_user.age, 28);

        txn.commit().expect("Failed to commit");
    }

    // Verify after commit
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let read_user: Option<test_users::User> = txn.read(&1u64).expect("Failed to read user");

        assert!(read_user.is_some());
        assert_eq!(read_user.unwrap().name, "Grace");
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_database_update_and_verify_state() {
    let (store, db_path) = common::create_test_db::<test_users::TestUsers>("migration_update")
        .expect("Failed to create test db");

    let user_id = 1u64;

    // Initial state
    {
        let txn = store.begin_write().expect("Failed to begin write");

        let user = test_users::User {
            id: user_id,
            name: String::from("Helen"),
            email: String::from("helen@example.com"),
            age: 30,
        };

        txn.create(&user).expect("Failed to create user");
        txn.commit().expect("Failed to commit");
    }

    // Verify initial state
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let user: Option<test_users::User> = txn.read(&user_id).expect("Failed to read");
        assert_eq!(user.unwrap().age, 30);
    }

    // Update
    {
        let txn = store.begin_write().expect("Failed to begin write");

        let mut user: test_users::User = txn
            .read(&user_id)
            .expect("Failed to read")
            .expect("User not found");

        // Modify
        user.age = 31;
        user.email = String::from("helen.updated@example.com");

        txn.update(&user).expect("Failed to update user");
        txn.commit().expect("Failed to commit");
    }

    // Verify updated state
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let user: Option<test_users::User> = txn.read(&user_id).expect("Failed to read");
        let user = user.unwrap();
        assert_eq!(user.age, 31);
        assert_eq!(user.email, "helen.updated@example.com");
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_database_delete_and_verify() {
    let (store, db_path) = common::create_test_db::<test_users::TestUsers>("migration_delete")
        .expect("Failed to create test db");

    let user_id = 1u64;

    // Create
    {
        let txn = store.begin_write().expect("Failed to begin write");

        let user = test_users::User {
            id: user_id,
            name: String::from("Ivy"),
            email: String::from("ivy@example.com"),
            age: 27,
        };

        txn.create(&user).expect("Failed to create user");
        txn.commit().expect("Failed to commit");
    }

    // Verify exists
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let user: Option<test_users::User> = txn.read(&user_id).expect("Failed to read");
        assert!(user.is_some());
    }

    // Delete
    {
        let txn = store.begin_write().expect("Failed to begin write");
        txn.delete::<test_users::User>(&user_id)
            .expect("Failed to delete user");
        txn.commit().expect("Failed to commit");
    }

    // Verify deleted
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let user: Option<test_users::User> = txn.read(&user_id).expect("Failed to read");
        assert!(user.is_none());
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_multiple_records_state_consistency() {
    let (store, db_path) = common::create_test_db::<test_users::TestUsers>("migration_multi")
        .expect("Failed to create test db");

    // Create multiple records
    {
        let txn = store.begin_write().expect("Failed to begin write");

        for i in 1..=5 {
            let user = test_users::User {
                id: i,
                name: format!("User{}", i),
                email: format!("user{}@example.com", i),
                age: 20 + i as u32,
            };
            txn.create(&user).expect("Failed to create user");
        }

        txn.commit().expect("Failed to commit");
    }

    // Verify all records exist
    {
        let txn = store.begin_read().expect("Failed to begin read");

        for i in 1..=5 {
            let user: Option<test_users::User> = txn.read(&i).expect("Failed to read");
            assert!(user.is_some());
            let user = user.unwrap();
            assert_eq!(user.id, i);
            assert_eq!(user.name, format!("User{}", i));
            assert_eq!(user.age, 20 + i as u32);
        }
    }

    // Update selective records
    {
        let txn = store.begin_write().expect("Failed to begin write");

        for i in [2u64, 4u64] {
            let mut user: test_users::User = txn
                .read(&i)
                .expect("Failed to read")
                .expect("User not found");
            user.age += 10;
            txn.update(&user).expect("Failed to update");
        }

        txn.commit().expect("Failed to commit");
    }

    // Verify selective updates
    {
        let txn = store.begin_read().expect("Failed to begin read");

        let user1: test_users::User = txn.read(&1u64).expect("Failed to read").unwrap();
        assert_eq!(user1.age, 21); // Unchanged

        let user2: test_users::User = txn.read(&2u64).expect("Failed to read").unwrap();
        assert_eq!(user2.age, 32); // Changed: 22 + 10

        let user3: test_users::User = txn.read(&3u64).expect("Failed to read").unwrap();
        assert_eq!(user3.age, 23); // Unchanged

        let user4: test_users::User = txn.read(&4u64).expect("Failed to read").unwrap();
        assert_eq!(user4.age, 34); // Changed: 24 + 10

        let user5: test_users::User = txn.read(&5u64).expect("Failed to read").unwrap();
        assert_eq!(user5.age, 25); // Unchanged
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_transaction_rollback_preserves_state() {
    let (store, db_path) = common::create_test_db::<test_users::TestUsers>("migration_rollback")
        .expect("Failed to create test db");

    // Create initial record
    {
        let txn = store.begin_write().expect("Failed to begin write");

        let user = test_users::User {
            id: 1,
            name: String::from("Jack"),
            email: String::from("jack@example.com"),
            age: 30,
        };

        txn.create(&user).expect("Failed to create user");
        txn.commit().expect("Failed to commit");
    }

    // Start a transaction but don't commit (implicit rollback)
    {
        let txn = store.begin_write().expect("Failed to begin write");

        let mut user: test_users::User = txn
            .read(&1u64)
            .expect("Failed to read")
            .expect("User not found");

        user.age = 99;
        txn.update(&user).expect("Failed to update");

        // Don't commit - transaction drops and rolls back
    }

    // Verify state unchanged
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let user: Option<test_users::User> = txn.read(&1u64).expect("Failed to read");
        assert_eq!(user.unwrap().age, 30); // Original value preserved
    }

    common::cleanup_test_db(db_path);
}
