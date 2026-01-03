// Macro-based boilerplate library - exact duplicate of boilerplate_lib using macros
// This demonstrates 1:1 parity between manual and macro-generated code
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

// Declare models module
pub mod models;

// DefinitionTwo with Category model (defined first to avoid forward references)
#[netabase_macros::netabase_definition(DefinitionTwo, subscriptions(General))]
pub mod definition_two {
    use super::*;

    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Encode,
        Decode,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
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

// Main Definition with User, Post, and HeavyModel
#[netabase_macros::netabase_definition(Definition, subscriptions(Topic1, Topic2, Topic3, Topic4))]
pub mod definition {
    use super::definition_two::{Category, CategoryID, DefinitionTwo};
    use super::*;
    use netabase_store::blob::NetabaseBlobItem;

    #[derive(
        Debug,
        Clone,
        Encode,
        Decode,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        Default,
    )]
    pub struct LargeUserFile {
        pub data: Vec<u8>,
        pub metadata: String,
    }

    #[derive(
        Debug,
        Clone,
        Encode,
        Decode,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        Default,
    )]
    pub struct AnotherLargeUserFile(pub Vec<u8>);

    #[derive(
        Debug,
        Clone,
        Encode,
        Decode,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        Default,
    )]
    pub struct HeavyAttachment {
        pub mime_type: String,
        pub data: Vec<u8>,
    }

    // Version 1 of User model - original version with single 'name' field
    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Encode,
        Decode,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
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

    // Version 2 of User model - current version with split name fields and blob support
    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Encode,
        Decode,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
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

    // Migration from V1 to V2
    impl netabase_store::traits::migration::MigrateFrom<UserV1> for User {
        fn migrate_from(old: UserV1) -> Self {
            // Split the single name field into first_name and last_name
            let parts: Vec<&str> = old.name.split_whitespace().collect();
            User {
                id: old.id,
                first_name: parts.first().map(|s| s.to_string()).unwrap_or_default(),
                last_name: parts.get(1).map(|s| s.to_string()).unwrap_or_default(),
                age: old.age,
                partner: String::new(), // New field - default to empty
                category: old.category,
                bio: LargeUserFile::default(),
                another: AnotherLargeUserFile::default(),
            }
        }
    }

    // Optional: Support downgrade for P2P compatibility
    impl netabase_store::traits::migration::MigrateTo<UserV1> for User {
        fn migrate_to(&self) -> UserV1 {
            UserV1 {
                id: self.id.clone(),
                name: format!("{} {}", self.first_name, self.last_name),
                age: self.age,
                category: self.category.clone(),
            }
        }
    }

    // Version 1 of Post model - without published flag
    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Encode,
        Decode,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
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

    // Version 2 of Post model - current version with published flag and tags
    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Encode,
        Decode,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
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

    // Migration from V1 to V2
    impl netabase_store::traits::migration::MigrateFrom<PostV1> for Post {
        fn migrate_from(old: PostV1) -> Self {
            Post {
                id: old.id,
                title: old.title,
                author_id: old.author_id,
                content: old.content,
                published: false, // Default to unpublished
                tags: vec![],     // Default to no tags
            }
        }
    }

    // Support downgrade for P2P compatibility (marked with supports_downgrade)
    impl netabase_store::traits::migration::MigrateTo<PostV1> for Post {
        fn migrate_to(&self) -> PostV1 {
            PostV1 {
                id: self.id.clone(),
                title: self.title.clone(),
                author_id: self.author_id.clone(),
                content: self.content.clone(),
            }
        }
    }

    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Encode,
        Decode,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
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
}

// Repository that combines Definition and DefinitionTwo
// This enables cross-definition RelationalLinks (User -> Category)
#[netabase_macros::netabase_repository(MainRepository, definitions(Definition, DefinitionTwo))]
pub mod main_repository {}

// Re-export everything from definitions
pub use definition::*;
pub use definition_two::*;
pub use main_repository::*;
