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
    #[subscribe(Topic1, Topic2)]
    pub struct User {
        #[primary_key]
        pub id: String,

        #[secondary_key]
        pub name: String,

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

// Re-export everything from definitions
pub use definition::*;
pub use definition_two::*;
