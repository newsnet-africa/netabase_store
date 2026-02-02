//! Common test utilities, fixtures, and helpers.
//!
//! This module provides shared infrastructure for integration tests:
//!
//! - **Fixtures**: Test data generation and database setup
//! - **Helpers**: Common operations and assertions
//! - **Models**: Reusable test model definitions
//!
//! # Usage
//!
//! Import in integration tests with:
//! ```ignore
//! mod common;
//! use common::*;
//! ```

use netabase_store::databases::redb::RedbStore;
use netabase_store::errors::NetabaseResult;
use netabase_store::traits::database::store::NBStore;
use std::path::PathBuf;
use std::sync::Once;
use strum::IntoDiscriminant;

// ============================================================================
// Test Database Creation
// ============================================================================

/// Create a temporary database for testing.
///
/// Returns the store and the database path. The caller is responsible for
/// cleanup using [`cleanup_test_db`].
///
/// # Example
///
/// ```ignore
/// let (store, db_path) = create_test_db::<MyDef>("my_test")?;
/// // ... run tests ...
/// cleanup_test_db(db_path);
/// ```
pub fn create_test_db<D>(name: &str) -> NetabaseResult<(RedbStore<D>, PathBuf)>
where
    D: netabase_store::traits::registry::definition::redb_definition::RedbDefinition + Clone,
    D::TreeNames: Default,
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <D as IntoDiscriminant>::Discriminant: PartialEq,
{
    let db_path = PathBuf::from(format!("/tmp/netabase_test_{}", name));

    // Clean up any existing database folder or file
    if db_path.exists() {
        if db_path.is_dir() {
            std::fs::remove_dir_all(&db_path).ok();
        } else {
            std::fs::remove_file(&db_path).ok();
        }
    }

    // Also clean up old-style .redb files if they exist
    let old_style_path = PathBuf::from(format!("/tmp/netabase_test_{}.redb", name));
    if old_style_path.exists() {
        std::fs::remove_file(&old_style_path).ok();
    }

    let store = RedbStore::<D>::new(&db_path)?;

    Ok((store, db_path))
}

/// Clean up test database folder.
pub fn cleanup_test_db(path: PathBuf) {
    if path.is_dir() {
        std::fs::remove_dir_all(&path).ok();
    } else if path.exists() {
        std::fs::remove_file(&path).ok();
    }
}

// ============================================================================
// Fixture Infrastructure
// ============================================================================

/// Fixture initialization guard.
///
/// Use with `Once` to ensure fixtures are created only once per test run.
pub static FIXTURES_INIT: Once = Once::new();

/// Fixture directory path.
pub const FIXTURE_DIR: &str = "/tmp/netabase_test_fixtures";

/// Ensure the fixture directory exists.
pub fn ensure_fixture_dir() -> PathBuf {
    let path = PathBuf::from(FIXTURE_DIR);
    if !path.exists() {
        std::fs::create_dir_all(&path).expect("Failed to create fixture directory");
    }
    path
}

/// Generate a schema TOML fixture file.
///
/// This function is idempotent - it only creates the file if it doesn't exist.
/// Use for tests that depend on schema import.
pub fn ensure_schema_fixture<D>(name: &str) -> PathBuf
where
    D: netabase_store::traits::registry::definition::NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    FIXTURES_INIT.call_once(|| {
        ensure_fixture_dir();
    });

    let path = PathBuf::from(FIXTURE_DIR).join(format!("{}_schema.toml", name));
    
    if !path.exists() {
        let schema_toml = D::export_toml();
        std::fs::write(&path, schema_toml).expect("Failed to write schema fixture");
    }
    
    path
}

/// Generate a test database fixture with sample data.
///
/// This function is idempotent - it only creates the database if it doesn't exist.
/// Useful for migration tests that need an existing database.
pub fn ensure_database_fixture<D, F>(name: &str, setup: F) -> PathBuf
where
    D: netabase_store::traits::registry::definition::redb_definition::RedbDefinition + Clone,
    D::TreeNames: Default,
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug + PartialEq,
    F: FnOnce(&RedbStore<D>) -> NetabaseResult<()>,
{
    FIXTURES_INIT.call_once(|| {
        ensure_fixture_dir();
    });

    let path = PathBuf::from(FIXTURE_DIR).join(name);
    
    if !path.exists() {
        let store = RedbStore::<D>::new(&path).expect("Failed to create fixture database");
        setup(&store).expect("Failed to setup fixture data");
    }
    
    path
}

// ============================================================================
// Test Assertions
// ============================================================================

/// Assert that two values are approximately equal (for floating point comparisons).
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {
        let diff = ($left - $right).abs();
        assert!(
            diff < $epsilon,
            "assertion failed: `(left ≈ right)`\n  left: `{:?}`\n right: `{:?}`\n  diff: `{:?}` (> epsilon: `{:?}`)",
            $left,
            $right,
            diff,
            $epsilon
        );
    };
}

/// Assert that a result is Ok.
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match &$result {
            Ok(_) => {}
            Err(e) => panic!("expected Ok, got Err: {:?}", e),
        }
    };
}

/// Assert that a result is Err.
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        match &$result {
            Ok(v) => panic!("expected Err, got Ok: {:?}", v),
            Err(_) => {}
        }
    };
}

// ============================================================================
// Test Data Generation
// ============================================================================

/// Generate a unique test identifier.
pub fn unique_id(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}_{}", prefix, timestamp)
}

/// Generate random test data of specified size.
pub fn random_bytes(size: usize) -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..size).map(|_| rng.random::<u8>()).collect()
}

/// Generate a random string of specified length.
pub fn random_string(len: usize) -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..len)
        .map(|_| rng.random_range(b'a'..=b'z') as char)
        .collect()
}

// ============================================================================
// Shared Test Models
// ============================================================================

/// A simple User model for basic CRUD tests.
///
/// This model is intentionally minimal to focus on core functionality.
/// Use for tests that don't need complex features like blobs or subscriptions.
pub mod simple_models {
    use serde::{Deserialize, Serialize};

    #[netabase_macros::netabase_definition(SimpleDef)]
    pub mod simple_def {
        use super::*;

        /// A basic user model with string ID.
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
            pub email: String,
        }

        /// A basic item model with numeric ID.
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
        pub struct Item {
            #[primary_key]
            pub id: u64,
            pub name: String,
            pub quantity: u32,
        }
    }

    pub use simple_def::*;
}

/// Models with secondary key indexing.
pub mod indexed_models {
    use serde::{Deserialize, Serialize};

    #[netabase_macros::netabase_definition(IndexedDef)]
    pub mod indexed_def {
        use super::*;

        /// A product with secondary key on category.
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
            pub name: String,
            #[secondary_key]
            pub category: String,
            pub price: u64,
        }

        /// A document with secondary key on author.
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
        pub struct Document {
            #[primary_key]
            pub id: String,
            pub title: String,
            #[secondary_key]
            pub author: String,
            pub content: String,
        }
    }

    pub use indexed_def::*;
}
