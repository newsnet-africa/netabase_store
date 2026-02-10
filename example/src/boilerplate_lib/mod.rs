//! Macro-based boilerplate library and examples.
//!
//! This library serves as a reference implementation and testbed for `netabase_store`.
//! It demonstrates how to define models, schemas, and repositories using the macro system.

use serde::{Deserialize, Serialize};

// Declare models module
pub mod models;
pub mod repository_example;
pub mod simple_def_example;

// Proper Pattern: Encapsulated Repository
// The repository module contains the definitions it manages.
#[netabase_macros::netabase_repository(MainRepository)]
pub mod main_repository {
    use super::*;

    /// DefinitionTwo module containing Category model.
    #[netabase_macros::netabase_networking]
    #[netabase_macros::netabase_definition(DefinitionTwo, subscriptions(General), repos(MainRepository))]
    pub mod definition_two {
        use super::*;
        use serde::{Serialize, Deserialize};

        #[derive(
            netabase_macros::NetabaseModel,
            Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord,
        )]
        #[subscribe(General)]
        pub struct Category {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub name: String,
            pub description: String,
        }
    }

    /// Main Definition module containing User, Post, and HeavyModel.
    #[netabase_macros::netabase_networking]
    #[netabase_macros::netabase_definition(Definition, subscriptions(Topic1, Topic2, Topic3, Topic4), repos(MainRepository))]
    pub mod definition {
        use super::definition_two::{Category, CategoryID, DefinitionTwo};
        use super::*;
        use serde::{Serialize, Deserialize};
        use netabase_store::blob::NetabaseBlobItem;

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default, netabase_macros::NetabaseBlobItem)]
        pub struct LargeUserFile {
            pub data: Vec<u8>,
            pub metadata: String,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default, netabase_macros::NetabaseBlobItem)]
        pub struct AnotherLargeUserFile(pub Vec<u8>);

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default, netabase_macros::NetabaseBlobItem)]
        pub struct HeavyAttachment {
            pub mime_type: String,
            pub data: Vec<u8>,
        }

        #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[netabase_version(family = "User", version = 1)]
        #[subscribe(Topic1, Topic2)]
        pub struct UserV1 {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub name: String,
            #[secondary_key]
            pub age: u8,
            #[link(DefinitionTwo, Category)]
            pub category: String,
        }

        #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[netabase_version(family = "User", version = 2, current)]
        #[subscribe(Topic1, Topic2)]
        pub struct User {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub first_name: String,
            #[secondary_key]
            pub last_name: String,
            #[secondary_key]
            pub age: u8,
            #[link(Definition, User)]
            pub partner: String,
            #[link(DefinitionTwo, Category)]
            pub category: String,
            #[blob]
            pub bio: LargeUserFile,
            #[blob]
            pub another: AnotherLargeUserFile,
        }

        impl netabase_store::traits::migration::MigrateFrom<UserV1> for User {
            fn migrate_from(old: UserV1) -> Self {
                use netabase_store::relational::RelationalLink;
                let parts: Vec<&str> = old.name.split_whitespace().collect();
                User {
                    id: old.id.clone(),
                    first_name: parts.first().map(|s| s.to_string()).unwrap_or_default(),
                    last_name: parts.get(1).map(|s| s.to_string()).unwrap_or_default(),
                    age: old.age,
                    partner: RelationalLink::new_dehydrated(old.id),
                    category: old.category,
                    bio: LargeUserFile::default(),
                    another: AnotherLargeUserFile::default(),
                }
            }
        }

        impl netabase_store::traits::migration::MigrateTo<UserV1> for User {
            fn migrate_to(&self) -> UserV1 {
                UserV1 {
                    id: self.id.clone(),
                    name: format!("{} {}", self.first_name, self.last_name).trim().to_string(),
                    age: self.age,
                    category: self.category.clone(),
                }
            }
        }

        #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[netabase_version(family = "Post", version = 1)]
        #[subscribe(Topic3, Topic4)]
        pub struct PostV1 {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub title: String,
            #[secondary_key]
            pub author_id: String,
            pub content: String,
        }

        #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[netabase_version(family = "Post", version = 2, current, supports_downgrade)]
        #[subscribe(Topic3, Topic4)]
        pub struct Post {
            #[primary_key]
            pub id: String,
            #[secondary_key]
            pub title: String,
            #[secondary_key]
            pub author_id: String,
            pub content: String,
            pub published: bool,
            pub tags: Vec<String>,
        }

        impl netabase_store::traits::migration::MigrateFrom<PostV1> for Post {
            fn migrate_from(old: PostV1) -> Self {
                Post {
                    id: old.id,
                    title: old.title,
                    author_id: old.author_id,
                    content: old.content,
                    published: false,
                    tags: vec![],
                }
            }
        }

        #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[subscribe(Topic1, Topic2, Topic3, Topic4)]
        pub struct HeavyModel {
            #[primary_key]
            pub id: String,
            pub name: String,
            pub title: String,
            #[secondary_key]
            pub category_label: String,
            #[secondary_key]
            pub score: u64,
            #[link(Definition, User)]
            pub creator: String,
            #[link(Definition, HeavyModel)]
            pub related_heavy: String,
            #[blob]
            pub attachment: HeavyAttachment,
            pub matrix: Vec<u64>,
        }

        #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[netabase_content_addressed(
            hasher = "crate::boilerplate_lib::models::FastHasher",
            function = "crate::boilerplate_lib::models::hash_model",
            key_type = "u64"
        )]
        #[subscribe(Topic1, Topic2)]
        pub struct ImmutablePost {
            #[secondary_key]
            pub author: String,
            pub content: String,
            pub timestamp: u64,
        }
    }
}

// Re-export for compatibility
pub use main_repository::definition::*;
pub use main_repository::definition_two::*;
pub use main_repository::{MainRepository, MainRepositoryStores};
pub use simple_def_example::*;