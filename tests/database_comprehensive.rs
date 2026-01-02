/// Comprehensive database tests with full state inspection before and after operations.
///
/// These tests verify:
/// - Transaction isolation
/// - CRUD operations correctness
/// - Query operations
/// - Error handling
/// - Rollback behavior
mod common;

use bincode::{Decode, Encode};
use netabase_macros::{NetabaseModel, netabase_definition};
use netabase_store::databases::redb::RedbStore;
use netabase_store::errors::{NetabaseError, NetabaseResult};
use netabase_store::query::{QueryConfig, QueryResult};
use netabase_store::traits::database::transaction::{NetabaseRoTransaction, NetabaseRwTransaction};

#[netabase_definition]
mod inventory {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, NetabaseModel, Encode, Decode)]
    pub struct Item {
        #[primary]
        pub id: u64,
        pub name: String,
        pub quantity: u32,
        pub price: u64,
    }
}

#[test]
fn test_empty_database_initial_state() {
    let (store, db_path) = common::create_test_db::<inventory::Inventory>("db_empty_state")
        .expect("Failed to create test db");

    // Verify empty database
    let txn = store.begin_read().expect("Failed to begin read");

    // Reading non-existent key returns None
    let result: Option<inventory::Item> = txn.read(&1u64).expect("Failed to read");
    assert!(
        result.is_none(),
        "New database should not contain any records"
    );

    let result2: Option<inventory::Item> = txn.read(&999u64).expect("Failed to read");
    assert!(
        result2.is_none(),
        "New database should not contain any records"
    );

    common::cleanup_test_db(db_path);
}

#[test]
fn test_create_single_record() {
    let (store, db_path) = common::create_test_db::<inventory::Inventory>("db_create_single")
        .expect("Failed to create test db");

    let item_id = 1u64;
    let item = inventory::Item {
        id: item_id,
        name: String::from("Widget"),
        quantity: 100,
        price: 1999,
    };

    // State before: record doesn't exist
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let result: Option<inventory::Item> = txn.read(&item_id).expect("Failed to read");
        assert!(result.is_none());
    }

    // Create record
    {
        let txn = store.begin_write().expect("Failed to begin write");
        txn.create(&item).expect("Failed to create");
        txn.commit().expect("Failed to commit");
    }

    // State after: record exists with exact data
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let result: Option<inventory::Item> = txn.read(&item_id).expect("Failed to read");
        assert!(result.is_some(), "Record should exist after create");

        let retrieved = result.unwrap();
        assert_eq!(retrieved.id, item.id);
        assert_eq!(retrieved.name, item.name);
        assert_eq!(retrieved.quantity, item.quantity);
        assert_eq!(retrieved.price, item.price);
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_create_duplicate_fails() {
    let (store, db_path) = common::create_test_db::<inventory::Inventory>("db_create_duplicate")
        .expect("Failed to create test db");

    let item = inventory::Item {
        id: 1,
        name: String::from("First"),
        quantity: 10,
        price: 100,
    };

    // First create succeeds
    {
        let txn = store.begin_write().expect("Failed to begin write");
        txn.create(&item).expect("First create should succeed");
        txn.commit().expect("Failed to commit");
    }

    // Second create with same ID should fail
    {
        let txn = store.begin_write().expect("Failed to begin write");
        let item2 = inventory::Item {
            id: 1, // Same ID
            name: String::from("Second"),
            quantity: 20,
            price: 200,
        };

        let result = txn.create(&item2);
        assert!(result.is_err(), "Creating duplicate key should fail");

        // Don't commit - rollback
    }

    // Original record unchanged
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let retrieved: inventory::Item = txn.read(&1u64).expect("Failed to read").unwrap();
        assert_eq!(
            retrieved.name, "First",
            "Original record should be unchanged"
        );
        assert_eq!(retrieved.quantity, 10);
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_update_existing_record() {
    let (store, db_path) = common::create_test_db::<inventory::Inventory>("db_update_existing")
        .expect("Failed to create test db");

    // Create initial record
    {
        let txn = store.begin_write().expect("Failed to begin write");
        let item = inventory::Item {
            id: 1,
            name: String::from("Gadget"),
            quantity: 50,
            price: 2999,
        };
        txn.create(&item).expect("Failed to create");
        txn.commit().expect("Failed to commit");
    }

    // State before update
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let item: inventory::Item = txn.read(&1u64).expect("Failed to read").unwrap();
        assert_eq!(item.quantity, 50);
        assert_eq!(item.price, 2999);
        assert_eq!(item.name, "Gadget");
    }

    // Update record
    {
        let txn = store.begin_write().expect("Failed to begin write");
        let mut item: inventory::Item = txn.read(&1u64).expect("Failed to read").unwrap();

        item.quantity = 25; // Sold half
        item.price = 1999; // Price reduction

        txn.update(&item).expect("Failed to update");
        txn.commit().expect("Failed to commit");
    }

    // State after update: changes applied
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let item: inventory::Item = txn.read(&1u64).expect("Failed to read").unwrap();
        assert_eq!(item.quantity, 25, "Quantity should be updated");
        assert_eq!(item.price, 1999, "Price should be updated");
        assert_eq!(item.name, "Gadget", "Name should be unchanged");
        assert_eq!(item.id, 1, "ID should be unchanged");
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_update_nonexistent_record_fails() {
    let (store, db_path) = common::create_test_db::<inventory::Inventory>("db_update_nonexistent")
        .expect("Failed to create test db");

    // Attempt to update non-existent record
    {
        let txn = store.begin_write().expect("Failed to begin write");
        let item = inventory::Item {
            id: 999,
            name: String::from("Phantom"),
            quantity: 1,
            price: 100,
        };

        let result = txn.update(&item);
        assert!(result.is_err(), "Updating non-existent record should fail");
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_delete_existing_record() {
    let (store, db_path) = common::create_test_db::<inventory::Inventory>("db_delete_existing")
        .expect("Failed to create test db");

    let item_id = 1u64;

    // Create record
    {
        let txn = store.begin_write().expect("Failed to begin write");
        let item = inventory::Item {
            id: item_id,
            name: String::from("Temporary"),
            quantity: 1,
            price: 100,
        };
        txn.create(&item).expect("Failed to create");
        txn.commit().expect("Failed to commit");
    }

    // State before delete: record exists
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let result: Option<inventory::Item> = txn.read(&item_id).expect("Failed to read");
        assert!(result.is_some(), "Record should exist before delete");
    }

    // Delete record
    {
        let txn = store.begin_write().expect("Failed to begin write");
        txn.delete(&item_id).expect("Failed to delete");
        txn.commit().expect("Failed to commit");
    }

    // State after delete: record doesn't exist
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let result: Option<inventory::Item> = txn.read(&item_id).expect("Failed to read");
        assert!(result.is_none(), "Record should not exist after delete");
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_delete_nonexistent_record_succeeds() {
    let (store, db_path) = common::create_test_db::<inventory::Inventory>("db_delete_nonexistent")
        .expect("Failed to create test db");

    // Delete non-existent record (should succeed as no-op)
    {
        let txn = store.begin_write().expect("Failed to begin write");
        let result = txn.delete(&999u64);
        assert!(
            result.is_ok(),
            "Deleting non-existent record should succeed (no-op)"
        );
        txn.commit().expect("Failed to commit");
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_transaction_rollback_on_drop() {
    let (store, db_path) = common::create_test_db::<inventory::Inventory>("db_rollback_drop")
        .expect("Failed to create test db");

    // Create initial record
    {
        let txn = store.begin_write().expect("Failed to begin write");
        let item = inventory::Item {
            id: 1,
            name: String::from("Original"),
            quantity: 100,
            price: 1000,
        };
        txn.create(&item).expect("Failed to create");
        txn.commit().expect("Failed to commit");
    }

    // Modify in transaction but don't commit (implicit rollback on drop)
    {
        let txn = store.begin_write().expect("Failed to begin write");
        let mut item: inventory::Item = txn.read(&1u64).expect("Failed to read").unwrap();
        item.quantity = 999;
        item.name = String::from("Modified");
        txn.update(&item).expect("Failed to update");
        // Transaction drops here without commit - should rollback
    }

    // Verify original state preserved
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let item: inventory::Item = txn.read(&1u64).expect("Failed to read").unwrap();
        assert_eq!(item.quantity, 100, "Changes should be rolled back");
        assert_eq!(item.name, "Original", "Changes should be rolled back");
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_multiple_records_crud() {
    let (store, db_path) = common::create_test_db::<inventory::Inventory>("db_multiple_records")
        .expect("Failed to create test db");

    let num_records = 10;

    // Create multiple records
    {
        let txn = store.begin_write().expect("Failed to begin write");
        for i in 1..=num_records {
            let item = inventory::Item {
                id: i,
                name: format!("Item{}", i),
                quantity: i as u32 * 10,
                price: i * 100,
            };
            txn.create(&item).expect("Failed to create");
        }
        txn.commit().expect("Failed to commit");
    }

    // Verify all records exist with correct data
    {
        let txn = store.begin_read().expect("Failed to begin read");
        for i in 1..=num_records {
            let item: Option<inventory::Item> = txn.read(&i).expect("Failed to read");
            assert!(item.is_some(), "Record {} should exist", i);

            let item = item.unwrap();
            assert_eq!(item.id, i);
            assert_eq!(item.name, format!("Item{}", i));
            assert_eq!(item.quantity, i as u32 * 10);
            assert_eq!(item.price, i * 100);
        }
    }

    // Update selective records
    {
        let txn = store.begin_write().expect("Failed to begin write");
        for i in [2u64, 4, 6, 8] {
            let mut item: inventory::Item = txn.read(&i).expect("Failed to read").unwrap();
            item.quantity += 100;
            txn.update(&item).expect("Failed to update");
        }
        txn.commit().expect("Failed to commit");
    }

    // Verify selective updates
    {
        let txn = store.begin_read().expect("Failed to begin read");

        // Updated records
        for i in [2u64, 4, 6, 8] {
            let item: inventory::Item = txn.read(&i).expect("Failed to read").unwrap();
            assert_eq!(
                item.quantity,
                i as u32 * 10 + 100,
                "Record {} should be updated",
                i
            );
        }

        // Non-updated records
        for i in [1u64, 3, 5, 7, 9, 10] {
            let item: inventory::Item = txn.read(&i).expect("Failed to read").unwrap();
            assert_eq!(
                item.quantity,
                i as u32 * 10,
                "Record {} should be unchanged",
                i
            );
        }
    }

    // Delete selective records
    {
        let txn = store.begin_write().expect("Failed to begin write");
        for i in [1u64, 5, 9] {
            txn.delete(&i).expect("Failed to delete");
        }
        txn.commit().expect("Failed to commit");
    }

    // Verify selective deletions
    {
        let txn = store.begin_read().expect("Failed to begin read");

        // Deleted records
        for i in [1u64, 5, 9] {
            let result: Option<inventory::Item> = txn.read(&i).expect("Failed to read");
            assert!(result.is_none(), "Record {} should be deleted", i);
        }

        // Remaining records
        for i in [2u64, 3, 4, 6, 7, 8, 10] {
            let result: Option<inventory::Item> = txn.read(&i).expect("Failed to read");
            assert!(result.is_some(), "Record {} should still exist", i);
        }
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_transaction_isolation() {
    let (store, db_path) = common::create_test_db::<inventory::Inventory>("db_isolation")
        .expect("Failed to create test db");

    // Create initial record
    {
        let txn = store.begin_write().expect("Failed to begin write");
        let item = inventory::Item {
            id: 1,
            name: String::from("Shared"),
            quantity: 100,
            price: 1000,
        };
        txn.create(&item).expect("Failed to create");
        txn.commit().expect("Failed to commit");
    }

    // Start write transaction but don't commit yet
    let write_txn = store.begin_write().expect("Failed to begin write");
    let mut item: inventory::Item = write_txn.read(&1u64).expect("Failed to read").unwrap();
    item.quantity = 50;
    write_txn.update(&item).expect("Failed to update");

    // Read transaction should see original state (not the uncommitted change)
    {
        let read_txn = store.begin_read().expect("Failed to begin read");
        let item: inventory::Item = read_txn.read(&1u64).expect("Failed to read").unwrap();
        assert_eq!(
            item.quantity, 100,
            "Read transaction should see original value"
        );
    }

    // Commit write transaction
    write_txn.commit().expect("Failed to commit");

    // Now read transaction should see new state
    {
        let read_txn = store.begin_read().expect("Failed to begin read");
        let item: inventory::Item = read_txn.read(&1u64).expect("Failed to read").unwrap();
        assert_eq!(
            item.quantity, 50,
            "Read transaction should see committed value"
        );
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_query_config_helpers() {
    // Test all query config helper methods
    let all = QueryConfig::all();
    assert_eq!(all.range, ..);

    let first = QueryConfig::first();
    assert_eq!(first.pagination.limit, Some(1));

    let dump = QueryConfig::dump_all();
    assert!(dump.fetch_options.include_blobs);
    assert_eq!(dump.fetch_options.hydration_depth, 0);

    let inspect = QueryConfig::inspect_range(0u64..100u64);
    assert_eq!(inspect.range, 0u64..100u64);
    assert!(inspect.fetch_options.include_blobs);

    // Test builder pattern
    let custom = QueryConfig::default()
        .with_limit(10)
        .with_offset(5)
        .no_blobs()
        .no_hydration()
        .reversed();

    assert_eq!(custom.pagination.limit, Some(10));
    assert_eq!(custom.pagination.offset, Some(5));
    assert!(!custom.fetch_options.include_blobs);
    assert_eq!(custom.fetch_options.hydration_depth, 0);
    assert!(custom.reversed);
}

#[test]
fn test_query_result_methods() {
    // Single variant
    let single = QueryResult::Single(Some(42));
    assert_eq!(single.len(), 1);
    assert!(!single.is_empty());
    assert_eq!(single.as_single(), Some(&42));
    assert_eq!(single.clone().unwrap_single(), 42);

    // Single None
    let empty: QueryResult<i32> = QueryResult::Single(None);
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
    assert_eq!(empty.as_single(), None);

    // Multiple variant
    let multiple = QueryResult::Multiple(vec![1, 2, 3, 4, 5]);
    assert_eq!(multiple.len(), 5);
    assert!(!multiple.is_empty());
    assert_eq!(multiple.as_multiple(), Some(&vec![1, 2, 3, 4, 5]));
    assert_eq!(multiple.clone().into_vec(), vec![1, 2, 3, 4, 5]);

    // Count variant
    let count: QueryResult<i32> = QueryResult::Count(100);
    assert_eq!(count.len(), 100);
    assert!(!count.is_empty());
    assert_eq!(count.count(), Some(100));
}
