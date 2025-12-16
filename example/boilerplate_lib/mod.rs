// Boilerplate example - Main entry point
//
// This example has been restructured into modules:
// - boilerplate_lib/models/user.rs - User model
// - boilerplate_lib/models/post.rs - Post model
// - boilerplate_lib/mod.rs - Definitions
//
// Run with: cargo run --example boilerplate

pub mod models;

use bincode::{BorrowDecode, Decode, Encode};
use models::category::{Category, CategoryID, CategoryKeys};
use models::post::{Post, PostID, PostKeys};
use models::user::{User, UserID, UserKeys};
use netabase_store::relational::GlobalDefinitionEnum;
use netabase_store::traits::registery::definition::NetabaseDefinition;
use netabase_store::traits::registery::definition::NetabaseDefinitionKeys;
use netabase_store::traits::registery::definition::NetabaseDefinitionTreeNames;
use netabase_store::traits::registery::definition::redb_definition::RedbDefinition;
use netabase_store::traits::registery::models::model::RedbModelTableDefinitions;
use netabase_store::traits::registery::models::treenames::ModelTreeNames;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumDiscriminants};

pub mod definition_two;
pub use definition_two::*;

// --- Global Enums ---

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Encode,
)]
pub enum GlobalDefinition {
    Def1(Definition),
    Def2(DefinitionTwo),
}

// Manual Decode impl for GlobalDefinition
impl Decode<()> for GlobalDefinition {
    fn decode<D: bincode::de::Decoder<Context = ()>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        use bincode::Decode;
        let variant = u32::decode(decoder)?;
        match variant {
            0 => Ok(GlobalDefinition::Def1(Definition::decode(decoder)?)),
            1 => Ok(GlobalDefinition::Def2(DefinitionTwo::decode(decoder)?)),
            _ => Err(bincode::error::DecodeError::Other("Invalid GlobalDefinition variant")),
        }
    }
}

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Encode, Decode,
)]
pub enum GlobalDefinitionKeys {
    Def1,
    Def2,
}

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Encode, Decode,
)]
pub enum GlobalKeys {
    Def1User,
    Def1Post,
    Def2Category,
}

// --- Definition One ---

#[derive(
    Clone,
    EnumDiscriminants,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    PartialOrd,
    Ord,
)]
#[strum_discriminants(name(DefinitionDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum Definition {
    User(User),
    Post(Post),
}

impl Decode<()> for Definition {
    fn decode<D: bincode::de::Decoder<Context = ()>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        if let Ok(user) = User::decode(decoder) {
            return Ok(Self::User(user))
        } else if let Ok(post) = Post::decode(decoder) {
            return  Ok(Self::Post(post))
        } else {
            return  Err(bincode::error::DecodeError::Other("Failed to decode"));
        }
    }
}

impl<'de> BorrowDecode<'de, ()> for Definition {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = ()>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        if let Ok(user) = User::borrow_decode(decoder) {
            return Ok(Self::User(user))
        } else if let Ok(post) = Post::borrow_decode(decoder) {
            return  Ok(Self::Post(post))
        } else {
            return  Err(bincode::error::DecodeError::Other("Failed to decode"));
        }
    }
}

impl GlobalDefinitionEnum for Definition {
    type GlobalDefinition = GlobalDefinition;
    type GlobalDefinitionKeys = GlobalDefinitionKeys;
    type GlobalKeys = GlobalKeys;

    fn into_global_definition(self) -> Self::GlobalDefinition {
        GlobalDefinition::Def1(self)
    }

    fn from_global_definition(global: Self::GlobalDefinition) -> Option<Self> {
        match global {
            GlobalDefinition::Def1(def) => Some(def),
            _ => None,
        }
    }

    fn discriminant_into_global(discriminant: Self::Discriminant) -> Self::GlobalKeys {
        match discriminant {
            DefinitionDiscriminants::User => GlobalKeys::Def1User,
            DefinitionDiscriminants::Post => GlobalKeys::Def1Post,
        }
    }

    fn discriminant_from_global(global: Self::GlobalKeys) -> Option<Self::Discriminant> {
        match global {
            GlobalKeys::Def1User => Some(DefinitionDiscriminants::User),
            GlobalKeys::Def1Post => Some(DefinitionDiscriminants::Post),
            _ => None,
        }
    }

    fn definition_discriminant_to_global() -> Self::GlobalDefinitionKeys {
        GlobalDefinitionKeys::Def1
    }

    fn global_to_definition_discriminant(global: Self::GlobalDefinitionKeys) -> bool {
        matches!(global, GlobalDefinitionKeys::Def1)
    }
}

impl GlobalDefinitionEnum for DefinitionTwo {
    type GlobalDefinition = GlobalDefinition;
    type GlobalDefinitionKeys = GlobalDefinitionKeys;
    type GlobalKeys = GlobalKeys;

    fn into_global_definition(self) -> Self::GlobalDefinition {
        GlobalDefinition::Def2(self)
    }

    fn from_global_definition(global: Self::GlobalDefinition) -> Option<Self> {
        match global {
            GlobalDefinition::Def2(def) => Some(def),
            _ => None,
        }
    }

    fn discriminant_into_global(discriminant: Self::Discriminant) -> Self::GlobalKeys {
        match discriminant {
            DefinitionTwoDiscriminants::Category => GlobalKeys::Def2Category,
        }
    }

    fn discriminant_from_global(global: Self::GlobalKeys) -> Option<Self::Discriminant> {
        match global {
            GlobalKeys::Def2Category => Some(DefinitionTwoDiscriminants::Category),
            _ => None,
        }
    }

    fn definition_discriminant_to_global() -> Self::GlobalDefinitionKeys {
        GlobalDefinitionKeys::Def2
    }

    fn global_to_definition_discriminant(global: Self::GlobalDefinitionKeys) -> bool {
        matches!(global, GlobalDefinitionKeys::Def2)
    }
}

use netabase_store::traits::registery::definition::subscription::{
    DefinitionSubscriptionRegistry, SubscriptionEntry,
};
use netabase_store::traits::permissions::{
    DefinitionPermissions, ModelAccessLevel,
    definition::CrossDefinitionAccess,
};

impl NetabaseDefinition for Definition {
    type TreeNames = DefinitionTreeNames;
    type DefKeys = DefinitionKeys;

    const SUBSCRIPTION_REGISTRY: DefinitionSubscriptionRegistry<'static, Self> =
        DefinitionSubscriptionRegistry::new(&[
            SubscriptionEntry {
                topic: "Topic1",
                subscribers: &[DefinitionDiscriminants::User],
            },
            SubscriptionEntry {
                topic: "Topic2",
                subscribers: &[DefinitionDiscriminants::User],
            },
            SubscriptionEntry {
                topic: "Topic3",
                subscribers: &[DefinitionDiscriminants::Post],
            },
            SubscriptionEntry {
                topic: "Topic4",
                subscribers: &[DefinitionDiscriminants::Post],
            },
        ]);

    const PERMISSIONS: DefinitionPermissions<'static, Self> = DefinitionPermissions {
        model_access: &[
            (DefinitionDiscriminants::User, ModelAccessLevel::ReadWrite),
            (DefinitionDiscriminants::Post, ModelAccessLevel::ReadWrite),
        ],
        cross_definition_access: &[
            (GlobalDefinitionKeys::Def2, CrossDefinitionAccess::READ_ONLY),
        ],
    };
}

#[derive(Clone, Debug)]
pub enum DefinitionTreeNames {
    User(ModelTreeNames<'static, Definition, User>),
    Post(ModelTreeNames<'static, Definition, Post>),
}

impl NetabaseDefinitionTreeNames<Definition> for DefinitionTreeNames {}

#[derive(Clone, Debug)]
pub enum DefinitionKeys {
    User(UserKeys),
    Post(PostKeys),
}

impl NetabaseDefinitionKeys<Definition> for DefinitionKeys {}

impl RedbDefinition for Definition {
    type ModelTableDefinition<'db> = RedbModelTableDefinitions<'db, User, Self>; // Using User as a representative model
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Encode,
    Decode,
    Serialize,
    Deserialize,
    AsRefStr,
)]
pub enum DefinitionSubscriptions {
    Topic1,
    Topic2,
    Topic3,
    Topic4,
}
