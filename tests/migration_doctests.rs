/// Comprehensive migration tests with state inspection before and after operations.
///
/// This test suite ensures all migration features work correctly by:
/// 1. Checking initial state before operations
/// 2. Performing migration operations
/// 3. Verifying final state matches expectations
/// 4. Testing error conditions and edge cases
mod common;

use bincode::{Decode, Encode};
use netabase_macros::{NetabaseModel, netabase_definition};
use netabase_store::databases::redb::RedbStore;
use netabase_store::errors::NetabaseResult;
use netabase_store::query::{QueryConfig, QueryResult};
use netabase_store::traits::database::transaction::{NetabaseRoTransaction, NetabaseRwTransaction};
use netabase_store::traits::migration::{
    MigrateFrom, VersionContext, VersionHeader, VersionedDecode, VersionedEncode,
};

#[netabase_definition]
mod products {
    use super::*;

    /// Version 1: Basic product with just id and name
    #[derive(Debug, Clone, PartialEq, Eq, NetabaseModel, Encode, Decode)]
    #[netabase_version(family = "Product", version = 1)]
    pub struct ProductV1 {
        #[primary]
        pub id: u64,
        pub name: String,
    }

    /// Version 2: Added price field
    #[derive(Debug, Clone, PartialEq, Eq, NetabaseModel, Encode, Decode)]
    #[netabase_version(family = "Product", version = 2)]
    pub struct ProductV2 {
        #[primary]
        pub id: u64,
        pub name: String,
        pub price: u64, // Price in cents
    }

    /// Version 3 (current): Added category and stock
    #[derive(Debug, Clone, PartialEq, Eq, NetabaseModel, Encode, Decode)]
    #[netabase_version(family = "Product", version = 3, current)]
    pub struct Product {
        #[primary]
        pub id: u64,
        pub name: String,
        pub price: u64,
        pub category: String,
        pub stock: u32,
    }
}

// Migration chain: V1 -> V2 -> V3
impl From<products::ProductV1> for products::ProductV2 {
    fn from(old: products::ProductV1) -> Self {
        products::ProductV2 {
            id: old.id,
            name: old.name,
            price: 999, // Default price: $9.99
        }
    }
}

impl From<products::ProductV2> for products::Product {
    fn from(old: products::ProductV2) -> Self {
        products::Product {
            id: old.id,
            name: old.name,
            price: old.price,
            category: String::from("Uncategorized"),
            stock: 0,
        }
    }
}

#[test]
fn test_version_header_roundtrip() {
    // Test that version header encoding/decoding is lossless
    for version in [0, 1, 100, u32::MAX] {
        let header = VersionHeader::new(version);
        let bytes = header.to_bytes();

        // Verify size
        assert_eq!(bytes.len(), VersionHeader::SIZE);

        // Verify magic bytes
        assert_eq!(bytes[0], b'N');
        assert_eq!(bytes[1], b'V');

        // Roundtrip
        let decoded = VersionHeader::from_bytes(&bytes).expect("Failed to decode header");
        assert_eq!(decoded.version, version);
        assert_eq!(decoded.magic, VersionHeader::MAGIC);
    }
}

#[test]
fn test_version_header_detection() {
    // Valid versioned data
    let versioned = VersionHeader::new(1).to_bytes();
    assert!(VersionHeader::is_versioned(&versioned));

    // Too short
    let too_short = vec![b'N', b'V', 0, 0];
    assert!(!VersionHeader::is_versioned(&too_short));

    // Wrong magic
    let wrong_magic = vec![b'X', b'Y', 1, 0, 0, 0];
    assert!(!VersionHeader::is_versioned(&wrong_magic));

    // Legacy unversioned
    let legacy = vec![0u8; 20];
    assert!(!VersionHeader::is_versioned(&legacy));
}

#[test]
fn test_version_context_creation() {
    // Default context
    let default_ctx = VersionContext::default();
    assert_eq!(default_ctx.expected_version, 0);
    assert!(default_ctx.auto_migrate);
    assert!(!default_ctx.strict);

    // Custom context
    let custom_ctx = VersionContext::new(3);
    assert_eq!(custom_ctx.expected_version, 3);
    assert!(custom_ctx.auto_migrate);

    // Strict context
    let strict_ctx = VersionContext::strict(2);
    assert_eq!(strict_ctx.expected_version, 2);
    assert!(!strict_ctx.auto_migrate);
    assert!(strict_ctx.strict);
}

#[test]
fn test_migration_v1_to_v2_manual() {
    // Create V1 product
    let v1 = products::ProductV1 {
        id: 1,
        name: String::from("Widget"),
    };

    // State before migration
    assert_eq!(v1.id, 1);
    assert_eq!(v1.name, "Widget");

    // Migrate to V2
    let v2: products::ProductV2 = v1.into();

    // State after migration
    assert_eq!(v2.id, 1);
    assert_eq!(v2.name, "Widget");
    assert_eq!(v2.price, 999); // Default value
}

#[test]
fn test_migration_v2_to_v3_manual() {
    // Create V2 product
    let v2 = products::ProductV2 {
        id: 2,
        name: String::from("Gadget"),
        price: 1999,
    };

    // State before migration
    assert_eq!(v2.id, 2);
    assert_eq!(v2.name, "Gadget");
    assert_eq!(v2.price, 1999);

    // Migrate to V3
    let v3: products::Product = v2.into();

    // State after migration
    assert_eq!(v3.id, 2);
    assert_eq!(v3.name, "Gadget");
    assert_eq!(v3.price, 1999);
    assert_eq!(v3.category, "Uncategorized"); // Default
    assert_eq!(v3.stock, 0); // Default
}

#[test]
fn test_chained_migration_v1_to_v3() {
    // Create V1 product
    let v1 = products::ProductV1 {
        id: 3,
        name: String::from("Gizmo"),
    };

    // Initial state
    assert_eq!(v1.id, 3);
    assert_eq!(v1.name, "Gizmo");

    // Chain: V1 -> V2
    let v2: products::ProductV2 = v1.into();

    // Intermediate state
    assert_eq!(v2.id, 3);
    assert_eq!(v2.name, "Gizmo");
    assert_eq!(v2.price, 999);

    // Chain: V2 -> V3
    let v3: products::Product = v2.into();

    // Final state
    assert_eq!(v3.id, 3);
    assert_eq!(v3.name, "Gizmo");
    assert_eq!(v3.price, 999);
    assert_eq!(v3.category, "Uncategorized");
    assert_eq!(v3.stock, 0);
}

#[test]
fn test_versioned_encode_decode_current() {
    let product = products::Product {
        id: 100,
        name: String::from("TestProduct"),
        price: 2999,
        category: String::from("Electronics"),
        stock: 50,
    };

    // State before encoding
    assert_eq!(product.id, 100);
    assert_eq!(product.stock, 50);

    // Encode with version header
    let encoded = product.encode_versioned();

    // Verify version header present
    assert!(VersionHeader::is_versioned(&encoded));
    let header = VersionHeader::from_bytes(&encoded).unwrap();
    assert_eq!(header.version, 3);

    // Decode with context
    let ctx = VersionContext::new(3);
    let decoded = products::Product::decode_versioned(&encoded, &ctx).unwrap();

    // State after decode - should match original
    assert_eq!(decoded.id, product.id);
    assert_eq!(decoded.name, product.name);
    assert_eq!(decoded.price, product.price);
    assert_eq!(decoded.category, product.category);
    assert_eq!(decoded.stock, product.stock);
}

#[test]
fn test_database_create_and_read() {
    let (store, db_path) = common::create_test_db::<products::Products>("migration_doctest_create")
        .expect("Failed to create test db");

    // Initial state: database is empty
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let result: Option<products::Product> = txn.read(&1u64).expect("Failed to read");
        assert!(result.is_none());
    }

    // Create a product
    {
        let txn = store.begin_write().expect("Failed to begin write");

        let product = products::Product {
            id: 1,
            name: String::from("Laptop"),
            price: 99999,
            category: String::from("Electronics"),
            stock: 10,
        };

        txn.create(&product).expect("Failed to create product");
        txn.commit().expect("Failed to commit");
    }

    // Final state: product exists with correct data
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let result: Option<products::Product> = txn.read(&1u64).expect("Failed to read");
        assert!(result.is_some());

        let product = result.unwrap();
        assert_eq!(product.id, 1);
        assert_eq!(product.name, "Laptop");
        assert_eq!(product.price, 99999);
        assert_eq!(product.category, "Electronics");
        assert_eq!(product.stock, 10);
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_database_update_state_changes() {
    let (store, db_path) = common::create_test_db::<products::Products>("migration_doctest_update")
        .expect("Failed to create test db");

    // Create initial product
    {
        let txn = store.begin_write().expect("Failed to begin write");

        let product = products::Product {
            id: 1,
            name: String::from("Phone"),
            price: 59999,
            category: String::from("Electronics"),
            stock: 20,
        };

        txn.create(&product).expect("Failed to create");
        txn.commit().expect("Failed to commit");
    }

    // State before update
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let product: products::Product = txn.read(&1u64).expect("Failed to read").unwrap();
        assert_eq!(product.stock, 20);
        assert_eq!(product.price, 59999);
    }

    // Perform update
    {
        let txn = store.begin_write().expect("Failed to begin write");

        let mut product: products::Product = txn.read(&1u64).expect("Failed to read").unwrap();
        product.stock = 15; // Sold 5 units
        product.price = 54999; // Price reduction

        txn.update(&product).expect("Failed to update");
        txn.commit().expect("Failed to commit");
    }

    // State after update
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let product: products::Product = txn.read(&1u64).expect("Failed to read").unwrap();
        assert_eq!(product.stock, 15); // Updated
        assert_eq!(product.price, 54999); // Updated
        assert_eq!(product.name, "Phone"); // Unchanged
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_database_delete_state_changes() {
    let (store, db_path) = common::create_test_db::<products::Products>("migration_doctest_delete")
        .expect("Failed to create test db");

    // Create product
    {
        let txn = store.begin_write().expect("Failed to begin write");

        let product = products::Product {
            id: 1,
            name: String::from("Tablet"),
            price: 39999,
            category: String::from("Electronics"),
            stock: 5,
        };

        txn.create(&product).expect("Failed to create");
        txn.commit().expect("Failed to commit");
    }

    // State before delete: product exists
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let result: Option<products::Product> = txn.read(&1u64).expect("Failed to read");
        assert!(result.is_some());
    }

    // Delete product
    {
        let txn = store.begin_write().expect("Failed to begin write");
        txn.delete(&1u64).expect("Failed to delete");
        txn.commit().expect("Failed to commit");
    }

    // State after delete: product does not exist
    {
        let txn = store.begin_read().expect("Failed to begin read");
        let result: Option<products::Product> = txn.read(&1u64).expect("Failed to read");
        assert!(result.is_none());
    }

    common::cleanup_test_db(db_path);
}

#[test]
fn test_query_config_inspection_utilities() {
    // Test dump_all
    let dump_config = QueryConfig::dump_all();
    assert!(dump_config.fetch_options.include_blobs);
    assert_eq!(dump_config.fetch_options.hydration_depth, 0);

    // Test first
    let first_config = QueryConfig::first();
    assert_eq!(first_config.pagination.limit, Some(1));

    // Test inspect_range
    let inspect_config = QueryConfig::inspect_range(0u64..10u64);
    assert_eq!(inspect_config.range, 0u64..10u64);
    assert!(inspect_config.fetch_options.include_blobs);
}

#[test]
fn test_query_result_utilities() {
    // Test unwrap_single
    let single = QueryResult::Single(Some(42));
    assert_eq!(single.unwrap_single(), 42);

    // Test expect_single
    let single2 = QueryResult::Single(Some(100));
    assert_eq!(single2.expect_single("should have value"), 100);

    // Test as_single
    let single3 = QueryResult::Single(Some(200));
    assert_eq!(single3.as_single(), Some(&200));

    // Test as_multiple
    let multiple = QueryResult::Multiple(vec![1, 2, 3]);
    assert_eq!(multiple.as_multiple(), Some(&vec![1, 2, 3]));

    // Test into_vec
    let multi2 = QueryResult::Multiple(vec![10, 20, 30]);
    assert_eq!(multi2.into_vec(), vec![10, 20, 30]);
}

#[test]
#[should_panic(expected = "called `QueryResult::unwrap_single()` on a `None` value")]
fn test_query_result_unwrap_single_panics_on_none() {
    let empty: QueryResult<i32> = QueryResult::Single(None);
    empty.unwrap_single();
}

#[test]
#[should_panic(expected = "called `QueryResult::unwrap_single()` on a non-Single variant")]
fn test_query_result_unwrap_single_panics_on_multiple() {
    let multiple = QueryResult::Multiple(vec![1, 2, 3]);
    multiple.unwrap_single();
}
