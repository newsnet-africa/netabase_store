// Macro-based boilerplate library - exact duplicate of boilerplate_lib using macros
// This demonstrates 1:1 parity between manual and macro-generated code
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

// Declare models module
pub mod models;
use netabase_store::blob::NetabaseBlobItem;
use netabase_store::traits::registery::models::model::NetabaseModel;
// Import blob types
use models::blob_types::*;

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
        id: String,

        #[secondary_key]
        name: String,

        description: String,
    }
}

// Main Definition with User, Post, and HeavyModel
#[netabase_macros::netabase_definition(Definition, subscriptions(Topic1, Topic2, Topic3, Topic4))]
pub mod definition {
    use super::definition_two::{Category, CategoryID, DefinitionTwo};
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
    #[subscribe(Topic1, Topic2)]
    pub struct User {
        #[primary_key]
        id: String,

        #[secondary_key]
        name: String,

        #[secondary_key]
        age: u8,

        #[link(Definition, User)]
        partner: String,

        #[link(DefinitionTwo, Category)]
        category: String,

        #[blob]
        bio: LargeUserFile,

        #[blob]
        another: AnotherLargeUserFile,
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
        id: String,

        #[secondary_key]
        title: String,

        #[secondary_key]
        author_id: String,

        content: String,

        published: bool,
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
        id: String,

        #[secondary_key]
        field1: String,

        #[secondary_key]
        field2: String,

        #[secondary_key]
        field3: String,

        field4: String,
        field5: String,

        #[blob]
        heavy_blob: HeavyBlob,
    }
}

// Re-export everything from definitions
pub use definition::*;
pub use definition_two::*;
