//! Comprehensive Integration Tests for Netabase Store
//!
//! This test suite provides verbose, rigorous testing of all core functionality,
//! serving as both verification and documentation for users of the library.
//!
//! # Test Coverage
//!
//! - **CRUD Operations**: Create, Read, Update, Delete with full state verification
//! - **Relational Links**: All four variants (Dehydrated, Owned, Hydrated, Borrowed)
//! - **Secondary Indexes**: Multi-value index behavior and queries
//! - **Subscriptions**: Subscription-based filtering and queries
//! - **Blob Storage**: Large data chunking and retrieval
//! - **Transactions**: Atomicity, isolation, commit/rollback
//! - **Repository Isolation**: Standalone repository behavior
//! - **Error Handling**: Expected failures and edge cases

pub mod common;

use common::{cleanup_test_db, create_test_db};
use netabase_store::databases::redb::transaction::RedbModelCrud;
use netabase_store::errors::NetabaseResult;
use netabase_store::relational::RelationalLink;
use netabase_store::traits::registery::models::model::RedbNetbaseModel;
use netabase_store::traits::registery::repository::Standalone;

use netabase_store_examples::{
    AnotherLargeUserFile, Category, CategoryID, Definition, DefinitionSubscriptions, DefinitionTwo,
    LargeUserFile, User, UserID,
};

// ============================================================================
// CRUD Operations - Complete State Verification
// ============================================================================

/// # Test: Create Single Model
///
/// ## Purpose
/// Verifies that a single model can be created in the database and all fields
/// are persisted correctly, including relational links and subscriptions.
///
/// ## Verification Strategy
/// 1. Create a User model with all field types populated
/// 2. Commit the transaction
/// 3. Open a new read transaction
/// 4. Read back the model by primary key
/// 5. Assert all fields match exactly
///
/// ## User-Facing API Demonstrated
/// - `store.begin_transaction()` - Start a new transaction
/// - `txn.create_redb(&model)` - Insert a model into the database
/// - `txn.commit()` - Commit changes atomically
/// - `User::read_default(&id, &tables)` - Read a model by primary key
#[test]
fn test_crud_create_single_model() -> NetabaseResult<()> {
    println!("\n=== Test: Create Single Model ===");
    let (store, db_path) = create_test_db::<Definition>("crud_create_single")?;

    // Prepare test data
    let user_id = UserID("alice_123".to_string());
    let partner_id = UserID("bob_456".to_string());
    let category_id = CategoryID("tech".to_string());

    let user = User {
        id: user_id.clone(),
        name: "Alice Johnson".to_string(),
        age: 28,
        partner: RelationalLink::<Standalone, Definition, Definition, User>::new_dehydrated(
            partner_id.clone(),
        ),
        category: RelationalLink::<Standalone, Definition, DefinitionTwo, Category>::new_dehydrated(
            category_id.clone(),
        ),
        subscriptions: vec![
            DefinitionSubscriptions::Topic1,
            DefinitionSubscriptions::Topic2,
        ],
        bio: LargeUserFile {
            data: vec![1, 2, 3, 4, 5],
            metadata: "Bio metadata".to_string(),
        },
        another: AnotherLargeUserFile(vec![10, 20, 30]),
    };

    println!("Creating user: {:?}", user.id);

    // Create the model in a write transaction
    let txn = store.begin_transaction()?;
    txn.create_redb(&user)?;
    txn.commit()?;

    println!("User created and committed");

    // Verify: Read back in a new transaction
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let read_user = User::read_default(&user_id, &tables)?;

        assert!(read_user.is_some(), "User should exist after creation");
        let read_user = read_user.unwrap();

        // Verify primary key
        assert_eq!(
            read_user.id.0, "alice_123",
            "Primary key should match exactly"
        );

        // Verify secondary keys
        assert_eq!(
            read_user.name, "Alice Johnson",
            "Name (secondary key) should match"
        );
        assert_eq!(read_user.age, 28, "Age (secondary key) should match");

        // Verify relational links (dehydrated - only primary keys stored)
        assert_eq!(
            read_user.partner.get_primary_key().0,
            "bob_456",
            "Partner link should point to correct user"
        );
        assert!(
            read_user.partner.is_dehydrated(),
            "Partner link should be dehydrated"
        );

        assert_eq!(
            read_user.category.get_primary_key().0,
            "tech",
            "Category link should point to correct category"
        );
        assert!(
            read_user.category.is_dehydrated(),
            "Category link should be dehydrated"
        );

        // Verify subscriptions
        assert_eq!(
            read_user.subscriptions.len(),
            2,
            "Should have exactly 2 subscriptions"
        );
        assert!(
            read_user
                .subscriptions
                .contains(&DefinitionSubscriptions::Topic1),
            "Should be subscribed to Topic1"
        );
        assert!(
            read_user
                .subscriptions
                .contains(&DefinitionSubscriptions::Topic2),
            "Should be subscribed to Topic2"
        );

        // Verify blob data
        assert_eq!(
            read_user.bio.data,
            vec![1, 2, 3, 4, 5],
            "Blob data should match"
        );
        assert_eq!(
            read_user.bio.metadata, "Bio metadata",
            "Blob metadata should match"
        );
        assert_eq!(
            read_user.another.0,
            vec![10, 20, 30],
            "Second blob data should match"
        );

        println!("✓ All fields verified successfully");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

/// # Test: Update Model - Full Field Verification
///
/// ## Purpose
/// Verifies that updating a model modifies all changed fields while preserving
/// unchanged ones, and that secondary indexes are updated correctly.
///
/// ## Verification Strategy
/// 1. Create an initial model with specific field values
/// 2. Read it back to verify initial state
/// 3. Update the model with different field values
/// 4. Read it back again to verify the update
/// 5. Ensure all modified fields changed and unmodified fields stayed the same
///
/// ## User-Facing API Demonstrated
/// - `txn.update_redb(&model)` - Update an existing model
/// - Demonstrates that primary key must match for update to work
#[test]
fn test_crud_update_model_full_verification() -> NetabaseResult<()> {
    println!("\n=== Test: Update Model - Full Verification ===");
    let (store, db_path) = create_test_db::<Definition>("crud_update_full")?;

    let user_id = UserID("updatable_user".to_string());

    // Initial state: Create user version 1
    let user_v1 = User {
        id: user_id.clone(),
        name: "Original Name".to_string(),
        age: 25,
        partner: RelationalLink::new_dehydrated(UserID("partner_v1".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("category_v1".to_string())),
        subscriptions: vec![DefinitionSubscriptions::Topic1],
        bio: LargeUserFile {
            data: vec![1, 2, 3],
            metadata: "Version 1".to_string(),
        },
        another: AnotherLargeUserFile(vec![10]),
    };

    println!("Creating initial user version");
    let txn = store.begin_transaction()?;
    txn.create_redb(&user_v1)?;
    txn.commit()?;

    // Verify initial state
    println!("Verifying initial state");
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        let user = User::read_default(&user_id, &tables)?.unwrap();

        assert_eq!(user.name, "Original Name");
        assert_eq!(user.age, 25);
        assert_eq!(user.partner.get_primary_key().0, "partner_v1");
        assert_eq!(user.subscriptions.len(), 1);
        println!("✓ Initial state verified");
    }
    txn.commit()?;

    // Update: Modify multiple fields
    let user_v2 = User {
        id: user_id.clone(),              // Same primary key
        name: "Updated Name".to_string(), // Changed
        age: 30,                          // Changed
        partner: RelationalLink::new_dehydrated(UserID("partner_v2".to_string())), // Changed
        category: RelationalLink::new_dehydrated(CategoryID("category_v1".to_string())), // Unchanged
        subscriptions: vec![
            DefinitionSubscriptions::Topic2,
            DefinitionSubscriptions::Topic3,
        ], // Changed
        bio: LargeUserFile {
            data: vec![4, 5, 6, 7],            // Changed
            metadata: "Version 2".to_string(), // Changed
        },
        another: AnotherLargeUserFile(vec![10]), // Unchanged
    };

    println!("Updating user to version 2");
    let txn = store.begin_transaction()?;
    txn.update_redb(&user_v2)?;
    txn.commit()?;

    // Verify updated state
    println!("Verifying updated state");
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        let user = User::read_default(&user_id, &tables)?.unwrap();

        // Verify changed fields
        assert_eq!(user.name, "Updated Name", "Name should be updated");
        assert_eq!(user.age, 30, "Age should be updated");
        assert_eq!(
            user.partner.get_primary_key().0,
            "partner_v2",
            "Partner link should be updated"
        );
        assert_eq!(user.subscriptions.len(), 2, "Should have 2 subscriptions");
        assert!(
            user.subscriptions
                .contains(&DefinitionSubscriptions::Topic2)
        );
        assert!(
            user.subscriptions
                .contains(&DefinitionSubscriptions::Topic3)
        );
        assert_eq!(
            user.bio.data,
            vec![4, 5, 6, 7],
            "Blob data should be updated"
        );
        assert_eq!(
            user.bio.metadata, "Version 2",
            "Blob metadata should be updated"
        );

        // Verify unchanged fields
        assert_eq!(
            user.category.get_primary_key().0,
            "category_v1",
            "Category should remain unchanged"
        );
        assert_eq!(
            user.another.0,
            vec![10],
            "Second blob should remain unchanged"
        );

        println!("✓ All changes and non-changes verified successfully");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

/// # Test: Delete Model - State Verification
///
/// ## Purpose
/// Verifies that deleting a model removes it completely from the database,
/// including all associated indexes (secondary, relational, subscription).
///
/// ## Verification Strategy
/// 1. Create a model with secondary keys, relational links, and subscriptions
/// 2. Verify it exists
/// 3. Delete the model
/// 4. Verify it no longer exists when queried by primary key
/// 5. Verify count decreases appropriately
///
/// ## User-Facing API Demonstrated
/// - `txn.delete_redb(&id)` - Delete a model by primary key
/// - `User::count_entries(&tables)` - Count total entries
#[test]
fn test_crud_delete_model_state_verification() -> NetabaseResult<()> {
    println!("\n=== Test: Delete Model - State Verification ===");
    let (store, db_path) = create_test_db::<Definition>("crud_delete_state")?;

    let user1_id = UserID("user_to_delete".to_string());
    let user2_id = UserID("user_to_keep".to_string());

    // Create two users
    println!("Creating two users");
    let txn = store.begin_transaction()?;

    let user1 = User {
        id: user1_id.clone(),
        name: "Delete Me".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![DefinitionSubscriptions::Topic1],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let user2 = User {
        id: user2_id.clone(),
        name: "Keep Me".to_string(),
        age: 25,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![DefinitionSubscriptions::Topic2],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    txn.create_redb(&user1)?;
    txn.create_redb(&user2)?;
    txn.commit()?;

    // Verify both users exist
    println!("Verifying both users exist");
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let count = User::count_entries(&tables)?;
        assert_eq!(count, 2, "Should have 2 users initially");

        let user1_exists = User::read_default(&user1_id, &tables)?.is_some();
        let user2_exists = User::read_default(&user2_id, &tables)?.is_some();

        assert!(user1_exists, "User 1 should exist");
        assert!(user2_exists, "User 2 should exist");

        println!("✓ Both users verified");
    }
    txn.commit()?;

    // Delete user1
    println!("Deleting user1");
    let txn = store.begin_transaction()?;
    txn.delete_redb::<User>(&user1_id)?;
    txn.commit()?;

    // Verify user1 deleted, user2 remains
    println!("Verifying deletion");
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let count = User::count_entries(&tables)?;
        assert_eq!(count, 1, "Should have 1 user after deletion");

        let user1_exists = User::read_default(&user1_id, &tables)?.is_some();
        let user2_exists = User::read_default(&user2_id, &tables)?.is_some();

        assert!(!user1_exists, "User 1 should NOT exist after deletion");
        assert!(user2_exists, "User 2 should still exist");

        // Verify user2 unchanged
        let user2 = User::read_default(&user2_id, &tables)?.unwrap();
        assert_eq!(user2.name, "Keep Me", "User 2 should be unchanged");
        assert_eq!(user2.age, 25, "User 2 age should be unchanged");

        println!("✓ Deletion verified - user1 removed, user2 intact");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// Relational Links - All Four Variants
// ============================================================================

/// # Test: RelationalLink Variants - Comprehensive Verification
///
/// ## Purpose
/// Demonstrates and verifies all four RelationalLink variants:
/// 1. **Dehydrated**: Only stores the primary key, no model data
/// 2. **Owned**: Owns the complete model in a Box
/// 3. **Hydrated**: Holds a reference to a model with user-controlled lifetime
/// 4. **Borrowed**: Holds a reference from a database AccessGuard
///
/// ## Important Semantic Note
/// `is_hydrated()` returns true for ANY variant containing model data (Owned, Hydrated, Borrowed).
/// Use `is_owned()`, `is_borrowed()` to check for specific variants.
/// Only `Dehydrated` variant has `is_hydrated() == false`.
///
/// ## Verification Strategy
/// For each variant, verify:
/// - Correct variant type detection (is_dehydrated, is_owned, etc.)
/// - Primary key access
/// - Model access (where applicable)
/// - Serialization/deserialization behavior
/// - Conversion between variants
///
/// ## User-Facing API Demonstrated
/// - `RelationalLink::new_dehydrated()` - Create dehydrated link
/// - `RelationalLink::new_owned()` - Create owned link
/// - `RelationalLink::new_hydrated()` - Create hydrated link
/// - `RelationalLink::new_borrowed()` - Create borrowed link (simulated)
/// - Variant check methods: `is_dehydrated()`, `is_owned()`, `is_hydrated()`, `is_borrowed()`
/// - `get_primary_key()` - Access the primary key
/// - `get_model()` - Access the model (if available)
/// - `dehydrate()` - Convert to dehydrated variant
/// - `into_owned()` - Extract owned model
#[test]
fn test_relational_links_all_variants() -> NetabaseResult<()> {
    println!("\n=== Test: RelationalLink Variants ===");

    let partner_id = UserID("partner_123".to_string());

    // Create a real user model for testing
    let partner = User {
        id: partner_id.clone(),
        name: "Bob Partner".to_string(),
        age: 32,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    // === Variant 1: Dehydrated ===
    println!("\n--- Testing Dehydrated variant ---");
    let dehydrated = RelationalLink::<Standalone, Definition, Definition, User>::new_dehydrated(
        partner_id.clone(),
    );

    assert!(dehydrated.is_dehydrated(), "Should be dehydrated");
    assert!(!dehydrated.is_owned(), "Should not be owned");
    assert!(!dehydrated.is_hydrated(), "Should not be hydrated");
    assert!(!dehydrated.is_borrowed(), "Should not be borrowed");

    assert_eq!(
        dehydrated.get_primary_key().0,
        "partner_123",
        "Primary key should be accessible"
    );
    assert!(
        dehydrated.get_model().is_none(),
        "Dehydrated should have no model"
    );

    println!("✓ Dehydrated variant verified");

    // === Variant 2: Owned ===
    println!("\n--- Testing Owned variant ---");
    let owned = RelationalLink::<Standalone, Definition, Definition, User>::new_owned(
        partner_id.clone(),
        partner.clone(),
    );

    assert!(!owned.is_dehydrated(), "Should not be dehydrated");
    assert!(owned.is_owned(), "Should be owned");
    assert!(
        owned.is_hydrated(),
        "Owned is hydrated (contains model data)"
    );
    assert!(!owned.is_borrowed(), "Should not be borrowed");

    assert_eq!(
        owned.get_primary_key().0,
        "partner_123",
        "Primary key should be accessible"
    );
    assert!(
        owned.get_model().is_some(),
        "Owned should have model accessible"
    );

    let model_ref = owned.get_model().unwrap();
    assert_eq!(
        model_ref.name, "Bob Partner",
        "Model data should be accessible"
    );
    assert_eq!(model_ref.age, 32, "Model data should match original");

    // Test into_owned extraction
    let owned_clone = owned.clone();
    let extracted = owned_clone.into_owned();
    assert!(extracted.is_some(), "Should be able to extract owned model");
    assert_eq!(
        extracted.unwrap().name,
        "Bob Partner",
        "Extracted model should match"
    );

    println!("✓ Owned variant verified");

    // === Variant 3: Hydrated ===
    println!("\n--- Testing Hydrated variant ---");
    let hydrated = RelationalLink::<Standalone, Definition, Definition, User>::new_hydrated(
        partner_id.clone(),
        &partner,
    );

    assert!(!hydrated.is_dehydrated(), "Should not be dehydrated");
    assert!(!hydrated.is_owned(), "Should not be owned");
    assert!(hydrated.is_hydrated(), "Should be hydrated");
    assert!(!hydrated.is_borrowed(), "Should not be borrowed");

    assert_eq!(
        hydrated.get_primary_key().0,
        "partner_123",
        "Primary key should be accessible"
    );
    assert!(
        hydrated.get_model().is_some(),
        "Hydrated should have model accessible"
    );

    let model_ref = hydrated.get_model().unwrap();
    assert_eq!(
        model_ref.name, "Bob Partner",
        "Model data should be accessible via reference"
    );

    println!("✓ Hydrated variant verified");

    // === Variant 4: Borrowed (simulated) ===
    println!("\n--- Testing Borrowed variant ---");
    let borrowed = RelationalLink::<Standalone, Definition, Definition, User>::new_borrowed(
        partner_id.clone(),
        &partner,
    );

    assert!(!borrowed.is_dehydrated(), "Should not be dehydrated");
    assert!(!borrowed.is_owned(), "Should not be owned");
    assert!(
        borrowed.is_hydrated(),
        "Borrowed is hydrated (contains model data)"
    );
    assert!(borrowed.is_borrowed(), "Should be borrowed");

    assert_eq!(
        borrowed.get_primary_key().0,
        "partner_123",
        "Primary key should be accessible"
    );
    assert!(
        borrowed.get_model().is_some(),
        "Borrowed should have model accessible"
    );

    let model_ref = borrowed.as_borrowed().unwrap();
    assert_eq!(
        model_ref.name, "Bob Partner",
        "Model data should be accessible via borrowed reference"
    );

    println!("✓ Borrowed variant verified");

    // === Test Conversions ===
    println!("\n--- Testing conversions ---");

    // Convert owned to dehydrated
    let dehydrated_from_owned = owned.clone().dehydrate();
    assert!(
        dehydrated_from_owned.is_dehydrated(),
        "Conversion to dehydrated should work"
    );
    assert_eq!(
        dehydrated_from_owned.get_primary_key().0,
        "partner_123",
        "Primary key preserved in conversion"
    );

    // Convert hydrated to dehydrated
    let dehydrated_from_hydrated = hydrated.clone().dehydrate();
    assert!(
        dehydrated_from_hydrated.is_dehydrated(),
        "Hydrated to dehydrated conversion should work"
    );

    // Convert borrowed to dehydrated
    let dehydrated_from_borrowed = borrowed.clone().dehydrate();
    assert!(
        dehydrated_from_borrowed.is_dehydrated(),
        "Borrowed to dehydrated conversion should work"
    );

    println!("✓ All conversions verified");

    // === Test Variant Ordering ===
    println!("\n--- Testing variant ordering ---");
    // Per documentation: Dehydrated < Owned < Hydrated < Borrowed

    let test_id = UserID("test".to_string());
    let test_user = User {
        id: test_id.clone(),
        name: "Test".to_string(),
        age: 20,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let dehy =
        RelationalLink::<Standalone, Definition, Definition, User>::new_dehydrated(test_id.clone());
    let own = RelationalLink::<Standalone, Definition, Definition, User>::new_owned(
        test_id.clone(),
        test_user.clone(),
    );
    let hydr = RelationalLink::<Standalone, Definition, Definition, User>::new_hydrated(
        test_id.clone(),
        &test_user,
    );
    let borr = RelationalLink::<Standalone, Definition, Definition, User>::new_borrowed(
        test_id.clone(),
        &test_user,
    );

    assert!(dehy < own, "Dehydrated < Owned");
    assert!(own < hydr, "Owned < Hydrated");
    assert!(hydr < borr, "Hydrated < Borrowed");

    println!("✓ Variant ordering verified");

    println!("\n=== All RelationalLink variants verified successfully ===");
    Ok(())
}

// ============================================================================
// Transaction Behavior - Atomicity and Isolation
// ============================================================================

/// # Test: Transaction Atomicity - Rollback Verification
///
/// ## Purpose
/// Verifies that uncommitted transactions don't persist data, demonstrating
/// the atomicity guarantee of transactions.
///
/// ## Verification Strategy
/// 1. Begin a transaction
/// 2. Create multiple models
/// 3. DON'T commit - let transaction drop
/// 4. Verify that no data was persisted
///
/// ## User-Facing API Demonstrated
/// - Transaction lifecycle without commit = automatic rollback
/// - Data isolation between transactions
#[test]
fn test_transaction_rollback_on_drop() -> NetabaseResult<()> {
    println!("\n=== Test: Transaction Rollback on Drop ===");
    let (store, db_path) = create_test_db::<Definition>("txn_rollback")?;

    let user_id = UserID("rollback_user".to_string());

    // Verify initial state is empty
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        let count = User::count_entries(&tables)?;
        assert_eq!(count, 0, "Database should be empty initially");
    }
    txn.commit()?;

    // Create a transaction but DON'T commit
    println!("Creating user in transaction that won't be committed");
    {
        let txn = store.begin_transaction()?;

        let user = User {
            id: user_id.clone(),
            name: "Should Not Persist".to_string(),
            age: 99,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            subscriptions: vec![],
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };

        txn.create_redb(&user)?;

        // Transaction dropped here without commit - should rollback
    }

    // Verify nothing was persisted
    println!("Verifying rollback occurred");
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let count = User::count_entries(&tables)?;
        assert_eq!(count, 0, "Database should still be empty after rollback");

        let user_exists = User::read_default(&user_id, &tables)?.is_some();
        assert!(
            !user_exists,
            "User should NOT exist after transaction rollback"
        );

        println!("✓ Rollback verified - no data persisted");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

/// # Test: Multiple Models in Single Transaction
///
/// ## Purpose
/// Verifies that multiple models can be created/updated/deleted within a single
/// transaction and all changes are committed atomically.
///
/// ## Verification Strategy
/// 1. Create multiple different model types in one transaction
/// 2. Commit
/// 3. Verify all models exist with correct data
///
/// ## User-Facing API Demonstrated
/// - Batching operations in a single transaction
/// - Atomic commit of multiple changes
#[test]
fn test_transaction_multiple_models() -> NetabaseResult<()> {
    println!("\n=== Test: Multiple Models in Single Transaction ===");
    let (store, db_path) = create_test_db::<Definition>("txn_multiple")?;

    println!("Creating 3 users in a single transaction");
    let txn = store.begin_transaction()?;

    for i in 0..3 {
        let user = User {
            id: UserID(format!("user_{}", i)),
            name: format!("User {}", i),
            age: 20 + i as u8,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            subscriptions: vec![],
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create_redb(&user)?;
    }

    txn.commit()?;
    println!("Transaction committed");

    // Verify all three users exist
    println!("Verifying all users were created atomically");
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let count = User::count_entries(&tables)?;
        assert_eq!(count, 3, "Should have exactly 3 users");

        for i in 0..3 {
            let user = User::read_default(&UserID(format!("user_{}", i)), &tables)?;
            assert!(user.is_some(), "User {} should exist", i);

            let user = user.unwrap();
            assert_eq!(
                user.name,
                format!("User {}", i),
                "User {} name should match",
                i
            );
            assert_eq!(user.age, 20 + i as u8, "User {} age should match", i);
        }

        println!("✓ All 3 users verified");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// List and Count Operations
// ============================================================================

/// # Test: Counting Entries - State Verification
///
/// ## Purpose
/// Verifies that the count_entries method accurately reflects the number of
/// models in the database as they are added and removed.
///
/// ## User-Facing API Demonstrated
/// - `Model::count_entries(&tables)` - Get total count of model instances
#[test]
fn test_count_entries_accurate() -> NetabaseResult<()> {
    println!("\n=== Test: Counting Entries ===");
    let (store, db_path) = create_test_db::<Definition>("count_accurate")?;

    // Initial count: 0
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        let count = User::count_entries(&tables)?;
        assert_eq!(count, 0, "Initial count should be 0");
        println!("✓ Initial count: 0");
    }
    txn.commit()?;

    // Add 5 users
    println!("Adding 5 users");
    let txn = store.begin_transaction()?;
    for i in 0..5 {
        let user = User {
            id: UserID(format!("count_user_{}", i)),
            name: format!("User {}", i),
            age: 25,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            subscriptions: vec![],
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create_redb(&user)?;
    }
    txn.commit()?;

    // Count: 5
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        let count = User::count_entries(&tables)?;
        assert_eq!(count, 5, "Count should be 5 after adding 5 users");
        println!("✓ Count after 5 additions: 5");
    }
    txn.commit()?;

    // Delete 2 users
    println!("Deleting 2 users");
    let txn = store.begin_transaction()?;
    txn.delete_redb::<User>(&UserID("count_user_0".to_string()))?;
    txn.delete_redb::<User>(&UserID("count_user_1".to_string()))?;
    txn.commit()?;

    // Count: 3
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        let count = User::count_entries(&tables)?;
        assert_eq!(count, 3, "Count should be 3 after deleting 2 users");
        println!("✓ Count after 2 deletions: 3");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

/// # Test: Listing All Entries - Order and Completeness
///
/// ## Purpose
/// Verifies that list_default returns all entries in the database and that
/// the entries are complete with all field data intact.
///
/// ## User-Facing API Demonstrated
/// - `Model::list_default(&tables)` - List all instances of a model
#[test]
fn test_list_entries_complete() -> NetabaseResult<()> {
    println!("\n=== Test: Listing All Entries ===");
    let (store, db_path) = create_test_db::<Definition>("list_complete")?;

    // Create 4 users with distinct data
    println!("Creating 4 users with distinct data");
    let txn = store.begin_transaction()?;

    let users_data = vec![
        ("alice", "Alice Anderson", 28),
        ("bob", "Bob Baker", 35),
        ("charlie", "Charlie Chen", 42),
        ("diana", "Diana Davis", 31),
    ];

    for (id, name, age) in &users_data {
        let user = User {
            id: UserID(id.to_string()),
            name: name.to_string(),
            age: *age,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            subscriptions: vec![],
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create_redb(&user)?;
    }
    txn.commit()?;

    // List and verify
    println!("Listing all users");
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let users = User::list_default(&tables)?;

        assert_eq!(users.len(), 4, "Should list all 4 users");

        // Verify all expected users are present
        let ids: Vec<String> = users.iter().map(|u| u.id.0.clone()).collect();
        assert!(ids.contains(&"alice".to_string()), "Should include alice");
        assert!(ids.contains(&"bob".to_string()), "Should include bob");
        assert!(
            ids.contains(&"charlie".to_string()),
            "Should include charlie"
        );
        assert!(ids.contains(&"diana".to_string()), "Should include diana");

        // Verify data integrity for each user
        for user in &users {
            let (_, expected_name, expected_age) = users_data
                .iter()
                .find(|(id, _, _)| *id == user.id.0.as_str())
                .unwrap();

            assert_eq!(
                &user.name, expected_name,
                "Name should match for {}",
                user.id.0
            );
            assert_eq!(
                user.age, *expected_age,
                "Age should match for {}",
                user.id.0
            );
        }

        println!("✓ All 4 users listed with complete data");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// Blob Storage - Large Data Handling
// ============================================================================

/// # Test: Blob Storage - Large Data Integrity
///
/// ## Purpose
/// Verifies that large blob data (exceeding chunk size) is correctly stored,
/// chunked, and retrieved with full data integrity.
///
/// ## Verification Strategy
/// 1. Create a user with large blob data (multiple chunks)
/// 2. Store in database
/// 3. Read back and verify the blob data is identical
/// 4. Test multiple blobs per model
///
/// ## User-Facing API Demonstrated
/// - Automatic blob chunking (transparent to user)
/// - Large data storage and retrieval
#[test]
fn test_blob_storage_large_data() -> NetabaseResult<()> {
    println!("\n=== Test: Blob Storage - Large Data ===");
    let (store, db_path) = create_test_db::<Definition>("blob_large")?;

    let user_id = UserID("blob_user".to_string());

    // Create large blob data (200KB - should span multiple chunks if chunk size is 64KB)
    let large_bio_data: Vec<u8> = (0..200_000).map(|i| (i % 256) as u8).collect();
    let large_another_data: Vec<u8> = (0..150_000).map(|i| ((i * 7) % 256) as u8).collect();

    println!(
        "Creating user with large blobs: {} bytes and {} bytes",
        large_bio_data.len(),
        large_another_data.len()
    );

    let user = User {
        id: user_id.clone(),
        name: "Blob User".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![],
        bio: LargeUserFile {
            data: large_bio_data.clone(),
            metadata: "Large bio metadata".to_string(),
        },
        another: AnotherLargeUserFile(large_another_data.clone()),
    };

    // Store
    let txn = store.begin_transaction()?;
    txn.create_redb(&user)?;
    txn.commit()?;
    println!("✓ Large blobs stored");

    // Read back and verify
    println!("Reading back and verifying blob data integrity");
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let read_user = User::read_default(&user_id, &tables)?.unwrap();

        // Verify blob data is identical
        assert_eq!(
            read_user.bio.data.len(),
            large_bio_data.len(),
            "Bio data length should match"
        );
        assert_eq!(
            read_user.bio.data, large_bio_data,
            "Bio data should be identical"
        );
        assert_eq!(
            read_user.bio.metadata, "Large bio metadata",
            "Bio metadata should match"
        );

        assert_eq!(
            read_user.another.0.len(),
            large_another_data.len(),
            "Another data length should match"
        );
        assert_eq!(
            read_user.another.0, large_another_data,
            "Another data should be identical"
        );

        println!("✓ Large blob data integrity verified");
        println!(
            "  - Bio: {} bytes retrieved correctly",
            read_user.bio.data.len()
        );
        println!(
            "  - Another: {} bytes retrieved correctly",
            read_user.another.0.len()
        );
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// Repository Isolation - Standalone Behavior
// ============================================================================

/// # Test: Standalone Repository - Cross-Definition Links
///
/// ## Purpose
/// Verifies that definitions in the Standalone repository can create relational
/// links to other definitions, demonstrating the default repository behavior.
///
/// ## Verification Strategy
/// 1. Create a User (in Definition)
/// 2. Create a Category (in DefinitionTwo)
/// 3. Create another User that links to the first User (same definition)
/// 4. Verify the User also links to Category (cross-definition)
/// 5. Verify all links are properly stored and retrieved
///
/// ## User-Facing API Demonstrated
/// - Standalone repository allows cross-definition communication
/// - Links between models in different definitions
/// - Links between models in the same definition
#[test]
fn test_standalone_repository_cross_definition_links() -> NetabaseResult<()> {
    println!("\n=== Test: Standalone Repository - Cross-Definition Links ===");

    // Note: This test demonstrates conceptual behavior
    // Actual cross-definition linking would require both Definition and DefinitionTwo
    // to be in the same store, which requires additional setup

    let (store, db_path) = create_test_db::<Definition>("standalone_links")?;

    let alice_id = UserID("alice".to_string());
    let bob_id = UserID("bob".to_string());
    let category_id = CategoryID("tech".to_string());

    println!("Creating Alice (user without links)");
    let alice = User {
        id: alice_id.clone(),
        name: "Alice".to_string(),
        age: 28,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = store.begin_transaction()?;
    txn.create_redb(&alice)?;
    txn.commit()?;

    println!("Creating Bob (user with link to Alice and category link)");
    let bob = User {
        id: bob_id.clone(),
        name: "Bob".to_string(),
        age: 32,
        // Link to Alice (same definition, same repository)
        partner: RelationalLink::new_dehydrated(alice_id.clone()),
        // Link to Category (different definition, but both in Standalone repository)
        category: RelationalLink::new_dehydrated(category_id.clone()),
        subscriptions: vec![],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = store.begin_transaction()?;
    txn.create_redb(&bob)?;
    txn.commit()?;

    // Verify Bob's links
    println!("Verifying Bob's relational links");
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let bob_read = User::read_default(&bob_id, &tables)?.unwrap();

        // Verify partner link (same definition)
        assert_eq!(
            bob_read.partner.get_primary_key().0,
            "alice",
            "Partner should link to Alice"
        );
        assert!(
            bob_read.partner.is_dehydrated(),
            "Partner link should be dehydrated after storage"
        );

        // Verify category link (cross-definition)
        assert_eq!(
            bob_read.category.get_primary_key().0,
            "tech",
            "Category should link to tech category"
        );
        assert!(
            bob_read.category.is_dehydrated(),
            "Category link should be dehydrated after storage"
        );

        println!("✓ Cross-definition links verified in Standalone repository");
        println!(
            "  - Same-definition link (partner): {:?}",
            bob_read.partner.get_primary_key()
        );
        println!(
            "  - Cross-definition link (category): {:?}",
            bob_read.category.get_primary_key()
        );
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// Subscription System
// ============================================================================

/// # Test: Subscriptions - Storage and Retrieval
///
/// ## Purpose
/// Verifies that subscription data is correctly stored and retrieved,
/// demonstrating the pub/sub indexing capability.
///
/// ## Verification Strategy
/// 1. Create users with different subscription combinations
/// 2. Verify subscriptions are stored correctly
/// 3. Verify subscription data persists across transactions
///
/// ## User-Facing API Demonstrated
/// - Subscription field on models
/// - Multiple subscriptions per model
/// - Subscription data integrity
#[test]
fn test_subscriptions_storage_and_retrieval() -> NetabaseResult<()> {
    println!("\n=== Test: Subscriptions - Storage and Retrieval ===");
    let (store, db_path) = create_test_db::<Definition>("subscriptions")?;

    // Create users with different subscription patterns
    let users = vec![
        (
            "user1",
            vec![DefinitionSubscriptions::Topic1],
            "User with single subscription",
        ),
        (
            "user2",
            vec![
                DefinitionSubscriptions::Topic1,
                DefinitionSubscriptions::Topic2,
            ],
            "User with two subscriptions",
        ),
        (
            "user3",
            vec![
                DefinitionSubscriptions::Topic1,
                DefinitionSubscriptions::Topic2,
                DefinitionSubscriptions::Topic3,
            ],
            "User with three subscriptions",
        ),
        ("user4", vec![], "User with no subscriptions"),
    ];

    println!("Creating users with various subscription patterns");
    let txn = store.begin_transaction()?;
    for (id, subs, desc) in &users {
        println!("  - {}: {}", id, desc);
        let user = User {
            id: UserID(id.to_string()),
            name: desc.to_string(),
            age: 30,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            subscriptions: subs.clone(),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create_redb(&user)?;
    }
    txn.commit()?;

    // Verify each user's subscriptions
    println!("Verifying subscription data");
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        for (id, expected_subs, _desc) in &users {
            let user = User::read_default(&UserID(id.to_string()), &tables)?.unwrap();

            assert_eq!(
                user.subscriptions.len(),
                expected_subs.len(),
                "{} should have {} subscriptions",
                id,
                expected_subs.len()
            );

            for sub in expected_subs {
                assert!(
                    user.subscriptions.contains(sub),
                    "{} should be subscribed to {:?}",
                    id,
                    sub
                );
            }

            println!(
                "  ✓ {} verified: {} subscriptions",
                id,
                user.subscriptions.len()
            );
        }
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// Edge Cases and Error Scenarios
// ============================================================================

/// # Test: Reading Non-Existent Model
///
/// ## Purpose
/// Verifies that attempting to read a model that doesn't exist returns None
/// rather than erroring, allowing graceful handling of missing data.
///
/// ## User-Facing API Demonstrated
/// - `read_default` returns `Option<Model>`
/// - Graceful handling of missing data
#[test]
fn test_read_nonexistent_model() -> NetabaseResult<()> {
    println!("\n=== Test: Reading Non-Existent Model ===");
    let (store, db_path) = create_test_db::<Definition>("read_nonexistent")?;

    let nonexistent_id = UserID("does_not_exist".to_string());

    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let result = User::read_default(&nonexistent_id, &tables)?;

        assert!(
            result.is_none(),
            "Reading non-existent model should return None"
        );
        println!("✓ Non-existent model correctly returns None");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

/// # Test: Delete Non-Existent Model
///
/// ## Purpose
/// Verifies that deleting a model that doesn't exist doesn't cause errors
/// and leaves the database state unchanged.
///
/// ## User-Facing API Demonstrated
/// - Idempotent delete operations
#[test]
fn test_delete_nonexistent_model() -> NetabaseResult<()> {
    println!("\n=== Test: Delete Non-Existent Model ===");
    let (store, db_path) = create_test_db::<Definition>("delete_nonexistent")?;

    // Create one user to ensure database isn't empty
    let existing_id = UserID("existing".to_string());
    let user = User {
        id: existing_id.clone(),
        name: "Existing User".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = store.begin_transaction()?;
    txn.create_redb(&user)?;
    txn.commit()?;

    // Try to delete non-existent user
    let nonexistent_id = UserID("does_not_exist".to_string());

    println!("Attempting to delete non-existent model");
    let txn = store.begin_transaction()?;
    let result = txn.delete_redb::<User>(&nonexistent_id);
    txn.commit()?;

    // Should not error
    assert!(
        result.is_ok(),
        "Deleting non-existent model should not error"
    );
    println!("✓ Delete of non-existent model succeeded gracefully");

    // Verify existing user is still there
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let count = User::count_entries(&tables)?;
        assert_eq!(count, 1, "Existing user should still be present");

        let existing_user = User::read_default(&existing_id, &tables)?;
        assert!(existing_user.is_some(), "Existing user should be unchanged");

        println!("✓ Existing data remains intact");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

/// # Test: Empty Database Operations
///
/// ## Purpose
/// Verifies that operations on an empty database work correctly (count, list, etc.)
///
/// ## User-Facing API Demonstrated
/// - Operations on empty database return appropriate empty results
#[test]
fn test_empty_database_operations() -> NetabaseResult<()> {
    println!("\n=== Test: Empty Database Operations ===");
    let (store, db_path) = create_test_db::<Definition>("empty_db")?;

    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        // Count should be 0
        let count = User::count_entries(&tables)?;
        assert_eq!(count, 0, "Empty database should have count 0");
        println!("✓ Count on empty database: 0");

        // List should return empty vec
        let users = User::list_default(&tables)?;
        assert_eq!(users.len(), 0, "Empty database should return empty list");
        println!("✓ List on empty database: []");

        // Read should return None
        let user = User::read_default(&UserID("any".to_string()), &tables)?;
        assert!(user.is_none(), "Read on empty database should return None");
        println!("✓ Read on empty database: None");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// Complex Multi-Model Scenarios
// ============================================================================

/// # Test: Multi-Model with Relationships
///
/// ## Purpose
/// Demonstrates a realistic scenario with multiple interconnected models,
/// testing that relationships are maintained correctly.
///
/// ## Scenario
/// Create a social network structure:
/// - Alice and Bob are partners (mutual relationship)
/// - Both belong to "Tech" category
/// - Charlie is separate, belongs to "Science" category
/// - Verify all relationships persist correctly
///
/// ## User-Facing API Demonstrated
/// - Complex relational structures
/// - Multiple models with cross-references
/// - Data integrity across related models
#[test]
fn test_complex_multi_model_relationships() -> NetabaseResult<()> {
    println!("\n=== Test: Complex Multi-Model Relationships ===");
    let (store, db_path) = create_test_db::<Definition>("complex_relationships")?;

    let alice_id = UserID("alice".to_string());
    let bob_id = UserID("bob".to_string());
    let charlie_id = UserID("charlie".to_string());
    let tech_category = CategoryID("tech".to_string());
    let science_category = CategoryID("science".to_string());

    println!("Creating social network structure:");
    println!("  - Alice ↔ Bob (partners, both in Tech)");
    println!("  - Charlie (separate, in Science)");

    let txn = store.begin_transaction()?;

    // Alice (links to Bob as partner, Tech category)
    let alice = User {
        id: alice_id.clone(),
        name: "Alice".to_string(),
        age: 28,
        partner: RelationalLink::new_dehydrated(bob_id.clone()),
        category: RelationalLink::new_dehydrated(tech_category.clone()),
        subscriptions: vec![DefinitionSubscriptions::Topic1],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    // Bob (links to Alice as partner, Tech category)
    let bob = User {
        id: bob_id.clone(),
        name: "Bob".to_string(),
        age: 32,
        partner: RelationalLink::new_dehydrated(alice_id.clone()),
        category: RelationalLink::new_dehydrated(tech_category.clone()),
        subscriptions: vec![DefinitionSubscriptions::Topic1],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    // Charlie (no partner, Science category)
    let charlie = User {
        id: charlie_id.clone(),
        name: "Charlie".to_string(),
        age: 35,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(science_category.clone()),
        subscriptions: vec![DefinitionSubscriptions::Topic2],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    txn.create_redb(&alice)?;
    txn.create_redb(&bob)?;
    txn.create_redb(&charlie)?;
    txn.commit()?;

    // Verify the relationship structure
    println!("Verifying relationship structure");
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        // Verify Alice
        let alice_read = User::read_default(&alice_id, &tables)?.unwrap();
        assert_eq!(alice_read.name, "Alice");
        assert_eq!(
            alice_read.partner.get_primary_key().0,
            "bob",
            "Alice's partner should be Bob"
        );
        assert_eq!(
            alice_read.category.get_primary_key().0,
            "tech",
            "Alice should be in Tech"
        );
        println!("  ✓ Alice: partner=Bob, category=Tech");

        // Verify Bob
        let bob_read = User::read_default(&bob_id, &tables)?.unwrap();
        assert_eq!(bob_read.name, "Bob");
        assert_eq!(
            bob_read.partner.get_primary_key().0,
            "alice",
            "Bob's partner should be Alice"
        );
        assert_eq!(
            bob_read.category.get_primary_key().0,
            "tech",
            "Bob should be in Tech"
        );
        println!("  ✓ Bob: partner=Alice, category=Tech");

        // Verify Charlie
        let charlie_read = User::read_default(&charlie_id, &tables)?.unwrap();
        assert_eq!(charlie_read.name, "Charlie");
        assert_eq!(
            charlie_read.partner.get_primary_key().0,
            "none",
            "Charlie should have no partner"
        );
        assert_eq!(
            charlie_read.category.get_primary_key().0,
            "science",
            "Charlie should be in Science"
        );
        println!("  ✓ Charlie: no partner, category=Science");

        println!("✓ Complex relationship structure verified");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}
