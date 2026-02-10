//! Exhaustive integration tests for netabase_store
//! This test suite covers almost all features in combination using the proper encapsulated repository pattern.

pub mod common;

use netabase_store::prelude::*;
use netabase_store::relational::RelationalLink;
use netabase_store::traits::migration::MigrateFrom;
use netabase_store::traits::registry::models::model::RedbNetbaseModel;
use netabase_store::databases::redb::transaction::RedbModelCrud;
use netabase_store::subscription_hash::SubscriptionMerkleTree;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

// ============================================================================
// Schema Definition (Encapsulated)
// ============================================================================

#[derive(netabase_macros::NetabaseBlobItem, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct LargeText {
    pub content: String,
    pub timestamp: u64,
}

#[netabase_macros::netabase_repository(ExhaustiveRepo)]
pub mod exhaustive_repo {
    use super::*;

    #[netabase_macros::netabase_networking]
    #[netabase_macros::netabase_definition(PrimaryDef, subscriptions(TopicA), repos(ExhaustiveRepo))]
    pub mod primary_def {
        use super::*;
        use netabase_store::blob::NetabaseBlobItem;
        use super::secondary_def::CategoryID;

        #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[netabase_version(family = "Author", version = 1)]
        pub struct AuthorV1 {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub name: String,
        }

        #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[netabase_version(family = "Author", version = 2, current)]
        #[netabase_libp2p]
        pub struct Author {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub name: String,
            #[secondary_key]
            pub age: u32,
            #[blob]
            pub bio: LargeText,
        }

        pub type PostEnvelopeID = PostID;

        impl MigrateFrom<AuthorV1> for Author {
            fn migrate_from(old: AuthorV1) -> Self {
                Author {
                    id: old.id,
                    name: old.name,
                    age: 0,
                    bio: LargeText::default(),
                    libp2p_metadata: None,
                }
            }
        }

        #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[netabase_content_addressed(
            hasher = "netabase_store::traits::database::hash::CryptoHash",
            function = "crate::exhaustive_hash_post",
            key_type = "u64"
        )]
        #[subscribe(TopicA)]
        pub struct Post {
            #[secondary_key]
            pub title: String,
            #[link(PrimaryDef, Author)]
            pub author: String,
            #[blob]
            pub content: LargeText,
            pub tags: Vec<String>,
            pub category_ids: Vec<CategoryID>,
        }
    }

    #[netabase_macros::netabase_definition(SecondaryDef, repos(ExhaustiveRepo))]
    pub mod secondary_def {
        use super::*;
        use netabase_store::blob::NetabaseBlobItem;
        use super::primary_def::{PostEnvelope, PostID, PrimaryDef};

        #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct Category {
            #[primary_key]
            pub id: u64,
            #[secondary_key]
            pub name: String,
        }

        #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct Comment {
            #[primary_key]
            pub id: u128,
            #[link(PrimaryDef, PostEnvelope)]
            pub post: u64,
            #[secondary_key]
            pub author_name: String,
            pub likes: u32,
        }
    }
}

// Re-export for test convenience
pub use exhaustive_repo::primary_def::*;
pub use exhaustive_repo::secondary_def::*;
pub use exhaustive_repo::{ExhaustiveRepo, ExhaustiveRepoStores};

pub fn exhaustive_hash_post(post: &exhaustive_repo::primary_def::Post) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    post.title.hash(&mut hasher);
    post.author.get_primary_key().hash(&mut hasher);
    post.tags.hash(&mut hasher);
    hasher.finish()
}

use netabase_store::traits::registry::models::keys::ModelKeyRange;
use netabase_store::databases::redb::transaction::CrudOptions;
use netabase_store::databases::redb::repository::RedbRepositoryStore;

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_exhaustive_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let repo_path = temp_dir.path();

    // 1. Repository & Store Creation
    let stores = ExhaustiveRepoStores::new(repo_path)?;
    
    // 2. CRUD & Secondary Keys
    let author_id = AuthorID("auth1".into());
    let author = Author {
        id: author_id.clone(),
        name: "Author One".into(),
        age: 40,
        bio: LargeText {
            content: "Encapsulated bio content...".to_string(),
            timestamp: 12345,
        },
        libp2p_metadata: None,
    };

    {
        let txn = stores.primary_def.begin_write()?;
        txn.create(&author)?;
        txn.create(&Author {
            id: AuthorID("auth2".into()),
            name: "Author Two".into(),
            age: 30,
            bio: LargeText::default(),
            libp2p_metadata: None,
        })?;
        txn.commit()?;
    }

    {
        let txn = stores.primary_def.begin_read()?;
        let retrieved = txn.read::<Author>(&author_id)?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Author One");

        let by_name = txn.query_by_secondary_key::<Author>(
            &AuthorSecondaryKeys::Name(AuthorName("Author One".into()))
        )?;
        assert_eq!(by_name.len(), 1);
    }

    // 3. Range Queries
    {
        let txn = stores.primary_def.begin_read()?;
        let table_defs = Author::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        
        use netabase_store::traits::registry::models::keys::SimpleKeyRange;
        
        let range = ModelKeyRange::<PrimaryDef, Author>::new()
            .and_secondary(SimpleKeyRange::Between {
                start: AuthorSecondaryKeys::Age(AuthorAge(30)),
                end: AuthorSecondaryKeys::Age(AuthorAge(45)),
                start_inclusive: true,
                end_inclusive: true,
            });
            
        let results = Author::list_with_key_ranges(&tables, &range, CrudOptions::default())?;
        assert_eq!(results.len(), 2);
    }

    // 4. Update Index Maintenance
    {
        let txn = stores.primary_def.begin_write()?;
        let mut author_mut = txn.read::<Author>(&author_id)?.unwrap();
        author_mut.name = "Updated Author".into();
        txn.update(&author_mut)?;
        txn.commit()?;
    }

    // 5. Content-Addressed Model & Hydration
    let post = Post {
        title: "Exhaustive Post".into(),
        author: RelationalLink::new_dehydrated(author_id.clone()),
        content: LargeText {
            content: "Post content...".to_string(),
            timestamp: 67890,
        },
        tags: vec!["test".into()],
        category_ids: vec![CategoryID(100)],
    };
    
    let post_envelope = PostEnvelope::from(post.clone());
    let post_hash = post_envelope.hash.clone();

    {
        let txn = stores.primary_def.begin_write()?;
        txn.create(&post_envelope)?;
        txn.commit()?;
    }

    // 6. Cross-Definition Hydration
    let comment_id = CommentID(1);
    let comment = Comment {
        id: comment_id.clone(),
        post: RelationalLink::new_dehydrated(post_hash.clone()),
        author_name: "Guest".into(),
        likes: 10,
    };

    {
        let txn = stores.secondary_def.begin_write()?;
        txn.create(&comment)?;
        txn.commit()?;
    }

    // Drop stores to release redb locks for RepositoryStore
    drop(stores);

    {
        let repo_store = RedbRepositoryStore::<ExhaustiveRepo>::new(repo_path)?;
        let txn_sec = repo_store.begin_read_for::<SecondaryDef>()?;
        let c = txn_sec.read::<Comment>(&comment_id)?.unwrap();
        
        // Hydrate from repo handle
        use netabase_store::databases::redb::repository::RepositoryHydrate;
        let hydrated_post = c.post.hydrate_from_repo(&repo_store)?;
        assert_eq!(hydrated_post.get_model().unwrap().inner.title, "Exhaustive Post");
    }

    Ok(())
}
