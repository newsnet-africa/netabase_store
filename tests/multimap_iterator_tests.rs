//! Tests for multimap table range iteration methods.
//!
//! These tests verify the new iterator-based methods that iterate over entire
//! multimap tables (secondary, relational, blob, subscription) rather than
//! just querying by a specific key.

use netabase_store::prelude::*;
use netabase_store::databases::redb::transaction::crud::RedbModelCrud;
use netabase_store::traits::registry::models::model::redb_model::RedbNetbaseModel;
use serde::{Deserialize, Serialize};

/// Test definition with secondary keys
#[netabase_macros::netabase_definition(MultimapIterDef)]
pub mod multimap_iter_def {
    use super::*;

    /// A model with a secondary index on email
    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct User {
        #[primary_key]
        pub id: String,
        #[secondary_key]
        pub email: String,
        pub name: String,
    }

    /// A simple model without relations for basic iteration tests
    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct Article {
        #[primary_key]
        pub id: u64,
        #[secondary_key]
        pub category: String,
        pub title: String,
    }
}

#[test]
fn test_iter_entries_main_table() -> NetabaseResult<()> {
    use multimap_iter_def::*;

    let (store, _path) = RedbStore::<MultimapIterDef>::new_temporary()?;

    // Insert test data
    {
        let txn = store.begin_write()?;
        txn.create(&User {
            id: UserID("u1".to_string()),
            email: "alice@example.com".to_string(),
            name: "Alice".to_string(),
        })?;
        txn.create(&User {
            id: UserID("u2".to_string()),
            email: "bob@example.com".to_string(),
            name: "Bob".to_string(),
        })?;
        txn.create(&User {
            id: UserID("u3".to_string()),
            email: "charlie@example.com".to_string(),
            name: "Charlie".to_string(),
        })?;
        txn.commit()?;
    }

    // Test iter_entries
    {
        let txn = store.begin_read()?;
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        
        let iter = User::iter_entries(&tables)?;
        let count = iter.count();
        assert_eq!(count, 3, "Should have 3 users");
        
        drop(tables);
        txn.commit()?;
    }

    Ok(())
}

#[test]
fn test_iter_range_main_table() -> NetabaseResult<()> {
    use multimap_iter_def::*;

    let (store, _path) = RedbStore::<MultimapIterDef>::new_temporary()?;

    // Insert test data with specific IDs for range testing
    {
        let txn = store.begin_write()?;
        for i in 0..10 {
            txn.create(&User {
                id: UserID(format!("user_{:02}", i)),
                email: format!("user{}@example.com", i),
                name: format!("User {}", i),
            })?;
        }
        txn.commit()?;
    }

    // Test iter_range
    {
        let txn = store.begin_read()?;
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        
        // Full range
        let full_iter = User::iter_entries(&tables)?;
        assert_eq!(full_iter.count(), 10, "Should have 10 users total");

        // Partial range (users 03 to 07 inclusive)
        let range_iter = User::iter_range(
            &tables,
            UserID("user_03".to_string())..=UserID("user_07".to_string()),
        )?;
        let range_count = range_iter.count();
        assert_eq!(range_count, 5, "Range should have 5 users");

        drop(tables);
        txn.commit()?;
    }

    Ok(())
}

#[test]
fn test_iter_secondary_table() -> NetabaseResult<()> {
    use multimap_iter_def::*;

    let (store, _path) = RedbStore::<MultimapIterDef>::new_temporary()?;

    // Insert test data
    {
        let txn = store.begin_write()?;
        txn.create(&User {
            id: UserID("u1".to_string()),
            email: "alice@example.com".to_string(),
            name: "Alice".to_string(),
        })?;
        txn.create(&User {
            id: UserID("u2".to_string()),
            email: "bob@example.com".to_string(),
            name: "Bob".to_string(),
        })?;
        // Different email for u3
        txn.create(&User {
            id: UserID("u3".to_string()),
            email: "charlie@example.com".to_string(),
            name: "Charlie".to_string(),
        })?;
        txn.commit()?;
    }

    // Test iter_secondary_table
    {
        let txn = store.begin_read()?;
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        
        // Iterate over all secondary index entries
        if let Some(iter) = User::iter_secondary_table(&tables, 0)? {
            let entries: Vec<_> = iter.collect();
            // Should have 3 entries total (one for each user)
            assert_eq!(entries.len(), 3, "Should have 3 secondary index entries");
        } else {
            panic!("Should have secondary table at index 0");
        }
        
        drop(tables);
        txn.commit()?;
    }

    Ok(())
}

#[test]
fn test_iter_by_secondary_key() -> NetabaseResult<()> {
    use multimap_iter_def::*;

    let (store, _path) = RedbStore::<MultimapIterDef>::new_temporary()?;

    // Insert test data with duplicate emails
    {
        let txn = store.begin_write()?;
        txn.create(&User {
            id: UserID("u1".to_string()),
            email: "shared@example.com".to_string(),
            name: "User 1".to_string(),
        })?;
        txn.create(&User {
            id: UserID("u2".to_string()),
            email: "shared@example.com".to_string(),
            name: "User 2".to_string(),
        })?;
        txn.create(&User {
            id: UserID("u3".to_string()),
            email: "unique@example.com".to_string(),
            name: "User 3".to_string(),
        })?;
        txn.commit()?;
    }

    // Test iter_by_secondary_key
    {
        let txn = store.begin_read()?;
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        
        // Query for shared email
        let shared_key = UserSecondaryKeys::Email(UserEmail("shared@example.com".to_string()));
        if let Some(iter) = User::iter_by_secondary_key(&shared_key, &tables)? {
            let primary_keys: Vec<_> = iter.filter_map(|r| r.ok()).collect();
            assert_eq!(primary_keys.len(), 2, "Should find 2 users with shared email");
        }

        // Query for unique email
        let unique_key = UserSecondaryKeys::Email(UserEmail("unique@example.com".to_string()));
        if let Some(iter) = User::iter_by_secondary_key(&unique_key, &tables)? {
            let primary_keys: Vec<_> = iter.filter_map(|r| r.ok()).collect();
            assert_eq!(primary_keys.len(), 1, "Should find 1 user with unique email");
        }

        drop(tables);
        txn.commit()?;
    }

    Ok(())
}

#[test]
fn test_count_entries() -> NetabaseResult<()> {
    use multimap_iter_def::*;

    let (store, _path) = RedbStore::<MultimapIterDef>::new_temporary()?;

    // Insert test data
    {
        let txn = store.begin_write()?;
        for i in 0..50 {
            txn.create(&User {
                id: UserID(format!("user_{}", i)),
                email: format!("user{}@example.com", i),
                name: format!("User {}", i),
            })?;
        }
        txn.commit()?;
    }

    // Test count_entries
    {
        let txn = store.begin_read()?;
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        
        let count = User::count_entries(&tables)?;
        assert_eq!(count, 50, "Should have 50 users");
        
        drop(tables);
        txn.commit()?;
    }

    Ok(())
}

#[test]
fn test_table_index_bounds() -> NetabaseResult<()> {
    use multimap_iter_def::*;

    let (store, _path) = RedbStore::<MultimapIterDef>::new_temporary()?;

    // Insert some data
    {
        let txn = store.begin_write()?;
        txn.create(&User {
            id: UserID("u1".to_string()),
            email: "test@example.com".to_string(),
            name: "Test".to_string(),
        })?;
        txn.commit()?;
    }

    // Test out-of-bounds table indices return None
    {
        let txn = store.begin_read()?;
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        
        // Valid index should return Some
        let valid = User::iter_secondary_table(&tables, 0)?;
        assert!(valid.is_some(), "Index 0 should be valid");

        // Invalid index should return None (not panic)
        let invalid = User::iter_secondary_table(&tables, 999)?;
        assert!(invalid.is_none(), "Index 999 should be None");

        drop(tables);
        txn.commit()?;
    }

    Ok(())
}

#[test]
fn test_empty_table_iteration() -> NetabaseResult<()> {
    use multimap_iter_def::*;

    let (store, _path) = RedbStore::<MultimapIterDef>::new_temporary()?;

    // Don't insert any data - test empty table iteration
    {
        let txn = store.begin_read()?;
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        
        // Iterate over empty main table
        let iter = User::iter_entries(&tables)?;
        assert_eq!(iter.count(), 0, "Empty table should have 0 entries");

        // Count should also be 0
        let count = User::count_entries(&tables)?;
        assert_eq!(count, 0, "Empty table count should be 0");

        drop(tables);
        txn.commit()?;
    }

    Ok(())
}

#[test]
fn test_multiple_secondary_keys_iteration() -> NetabaseResult<()> {
    use multimap_iter_def::*;

    let (store, _path) = RedbStore::<MultimapIterDef>::new_temporary()?;

    // Insert articles with categories
    {
        let txn = store.begin_write()?;
        txn.create(&Article {
            id: ArticleID(1),
            category: "tech".to_string(),
            title: "Tech Article 1".to_string(),
        })?;
        txn.create(&Article {
            id: ArticleID(2),
            category: "tech".to_string(),
            title: "Tech Article 2".to_string(),
        })?;
        txn.create(&Article {
            id: ArticleID(3),
            category: "sports".to_string(),
            title: "Sports Article".to_string(),
        })?;
        txn.commit()?;
    }

    // Test iteration over secondary table
    {
        let txn = store.begin_read()?;
        let table_defs = Article::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        
        // Iterate all entries in main table
        let iter = Article::iter_entries(&tables)?;
        assert_eq!(iter.count(), 3, "Should have 3 articles");
        
        // Query by category
        let tech_key = ArticleSecondaryKeys::Category(ArticleCategory("tech".to_string()));
        if let Some(iter) = Article::iter_by_secondary_key(&tech_key, &tables)? {
            let articles: Vec<_> = iter.filter_map(|r| r.ok()).collect();
            assert_eq!(articles.len(), 2, "Should find 2 tech articles");
        }
        
        drop(tables);
        txn.commit()?;
    }

    Ok(())
}
