// Boilerplate example - Main entry point
//
// This example has been restructured into modules:
// - boilerplate_lib/models/user.rs - User model
// - boilerplate_lib/models/post.rs - Post model
// - boilerplate_lib/mod.rs - Definitions
//
// Run with: cargo run --example boilerplate

// Basic imports needed for early type definitions
use bincode::{BorrowDecode, Decode, Encode};
use derive_more::{From, TryInto};
use netabase_store::databases::redb::transaction::ModelOpenTables;
use netabase_store::blob::NetabaseBlobItem;
use netabase_store::traits::registery::models::{
    StoreKey, StoreKeyMarker, StoreValue, StoreValueMarker,
    keys::{
        NetabaseModelKeys, NetabaseModelPrimaryKey, NetabaseModelRelationalKey,
        NetabaseModelSecondaryKey, NetabaseModelSubscriptionKey, blob::NetabaseModelBlobKey,
    },
    model::{NetabaseModel, NetabaseModelMarker, RedbNetbaseModel},
    treenames::{DiscriminantTableName, ModelTreeNames},
};
use redb::{Key, Value};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use strum::{AsRefStr, EnumDiscriminants, IntoDiscriminant};

// Declare modules first
pub mod models;

// Define DefinitionTwo subscriptions early so models can import them
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
    EnumDiscriminants,
)]
#[strum_discriminants(name(DefinitionTwoSubscriptionsDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum DefinitionTwoSubscriptions {
    General,
}

// --- Category Model (inlined to resolve circular dependency) ---

#[derive(
    Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub struct Category {
    pub id: CategoryID,
    pub name: String,
}

#[derive(
    Clone,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Debug,
    Encode,
    Decode,
    Serialize,
    Deserialize,
    EnumDiscriminants,
    Hash,
)]
#[strum_discriminants(name(CategorySubscriptionsDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum CategorySubscriptions {
    General(DefinitionTwoSubscriptions),
}

// Note: impl NetabaseModelSubscriptionKey will be added after DefinitionTwo is defined

#[derive(
    Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash,
)]
pub struct CategoryID(pub String);
#[derive(
    Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash,
)]
pub struct CategoryName(pub String);

#[derive(
    Clone,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Debug,
    Encode,
    Decode,
    EnumDiscriminants,
    Serialize,
    Deserialize,
    Hash,
)]
#[strum_discriminants(name(CategorySecondaryKeysDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum CategorySecondaryKeys {
    Name(CategoryName),
}

#[derive(
    Clone,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Debug,
    Encode,
    Decode,
    EnumDiscriminants,
    Serialize,
    Deserialize,
    Hash,
)]
#[strum_discriminants(name(CategoryRelationalKeysDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum CategoryRelationalKeys {
    None,
}

#[derive(
    Clone,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Debug,
    Encode,
    Decode,
    EnumDiscriminants,
    Serialize,
    Deserialize,
    Hash,
)]
#[strum_discriminants(name(CategoryBlobKeysDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum CategoryBlobKeys {
    None,
}

impl<'a> NetabaseModelBlobKey<'a, DefinitionTwo, Category, CategoryKeys> for CategoryBlobKeys {
    type PrimaryKey = CategoryID;
    type BlobItem = CategoryBlobItem;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, Serialize, Deserialize)]
pub struct CategoryBlobItem;

impl NetabaseBlobItem for CategoryBlobItem {
    type Blobs = ();
    fn split_into_blobs(&self) -> Vec<Self::Blobs> {
        vec![]
    }
    fn reconstruct_from_blobs(_blobs: Vec<Self::Blobs>) -> Self {
        CategoryBlobItem
    }
}

#[derive(Clone, Debug)]
pub enum CategoryKeys {
    Primary(CategoryID),
    Secondary(CategorySecondaryKeys),
    Relational(CategoryRelationalKeys),
    Subscription(CategorySubscriptions),
    Blob(CategoryBlobKeys),
}

// Import from modules
use models::post::{Post, PostID, PostKeys};
use models::user::{User, UserID, UserKeys};
use netabase_store::relational::GlobalDefinitionEnum;
use netabase_store::traits::registery::definition::NetabaseDefinition;
use netabase_store::traits::registery::definition::NetabaseDefinitionKeys;
use netabase_store::traits::registery::definition::NetabaseDefinitionTreeNames;
use netabase_store::traits::registery::definition::redb_definition::RedbDefinition;
use netabase_store::traits::registery::models::model::RedbModelTableDefinitions;

// Define DefinitionTwo enum (now that Category is defined)
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
    Decode,
    PartialOrd,
    Ord,
    TryInto,
    From,
)]
#[strum_discriminants(name(DefinitionTwoDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum DefinitionTwo {
    Category(Category),
}

// --- Category trait implementations (must come after DefinitionTwo is defined) ---

impl NetabaseModelSubscriptionKey<DefinitionTwo, Category, CategoryKeys> for CategorySubscriptions {}

impl From<DefinitionTwoSubscriptions> for CategorySubscriptions {
    fn from(value: DefinitionTwoSubscriptions) -> Self {
        match value {
            DefinitionTwoSubscriptions::General => CategorySubscriptions::General(value),
        }
    }
}

impl TryInto<DefinitionTwoSubscriptions> for CategorySubscriptions {
    type Error = ();

    fn try_into(self) -> Result<DefinitionTwoSubscriptions, Self::Error> {
        match self {
            CategorySubscriptions::General(def_sub) => Ok(def_sub),
        }
    }
}

impl NetabaseModel<DefinitionTwo> for Category {
    type Keys = CategoryKeys;

    const TREE_NAMES: ModelTreeNames<'static, DefinitionTwo, Self> = ModelTreeNames {
        main: DiscriminantTableName::new(
            DefinitionTwoDiscriminants::Category,
            "DefinitionTwo:Category:Primary:Main",
        ),
        secondary: &[DiscriminantTableName::new(
            CategorySecondaryKeysDiscriminants::Name,
            "DefinitionTwo:Category:Secondary:Name",
        )],
        relational: &[],
        subscription: Some(&[DiscriminantTableName::new(
            CategorySubscriptionsDiscriminants::General,
            "DefinitionTwo:Subscription:General",
        )]),
        blob: &[DiscriminantTableName::new(
            CategoryBlobKeysDiscriminants::None,
            "DefinitionTwo:Blob:None",
        )],
    };

    fn get_primary_key<'a>(&'a self) -> CategoryID {
        self.id.clone()
    }

    fn get_secondary_keys<'a>(&'a self) -> Vec<CategorySecondaryKeys> {
        vec![CategorySecondaryKeys::Name(CategoryName(self.name.clone()))]
    }

    fn get_relational_keys<'a>(&'a self) -> Vec<CategoryRelationalKeys> {
        vec![]
    }

    fn get_subscription_keys<'a>(&'a self) -> Vec<CategorySubscriptions> {
        vec![CategorySubscriptions::General(
            DefinitionTwoSubscriptions::General,
        )]
    }

    fn get_blob_entries<'a>(
        &'a self,
    ) -> Vec<(
        <Self::Keys as NetabaseModelKeys<DefinitionTwo, Self>>::Blob<'a>,
        <<Self::Keys as NetabaseModelKeys<DefinitionTwo, Self>>::Blob<'a> as NetabaseModelBlobKey<'a, DefinitionTwo, Self, Self::Keys>>::BlobItem,
    )> {
        vec![]
    }
}

impl StoreValueMarker<DefinitionTwo> for Category {}
impl StoreValueMarker<DefinitionTwo> for CategoryID {}

impl StoreKeyMarker<DefinitionTwo> for CategoryID {}
impl StoreKey<DefinitionTwo, Category> for CategoryID {}
impl StoreValue<DefinitionTwo, CategoryID> for Category {}

impl StoreKeyMarker<DefinitionTwo> for CategorySecondaryKeys {}
impl StoreKeyMarker<DefinitionTwo> for CategoryRelationalKeys {}
impl StoreKeyMarker<DefinitionTwo> for CategorySubscriptions {}
impl StoreKeyMarker<DefinitionTwo> for CategoryBlobKeys {}

impl StoreKey<DefinitionTwo, CategoryID> for CategorySecondaryKeys {}
impl StoreKey<DefinitionTwo, CategoryID> for CategoryRelationalKeys {}
impl StoreKey<DefinitionTwo, CategoryID> for CategorySubscriptions {}

impl StoreValue<DefinitionTwo, CategorySecondaryKeys> for CategoryID {}
impl StoreValue<DefinitionTwo, CategoryRelationalKeys> for CategoryID {}
impl StoreValue<DefinitionTwo, CategorySubscriptions> for CategoryID {}

impl NetabaseModelMarker<DefinitionTwo> for Category {}

impl NetabaseModelKeys<DefinitionTwo, Category> for CategoryKeys {
    type Primary<'a> = CategoryID;
    type Secondary<'a> = CategorySecondaryKeys;
    type Relational<'a> = CategoryRelationalKeys;
    type Subscription<'a> = CategorySubscriptions;
    type Blob<'a> = CategoryBlobKeys;
}

impl<'a> NetabaseModelPrimaryKey<'a, DefinitionTwo, Category, CategoryKeys> for CategoryID {}
impl<'a> NetabaseModelSecondaryKey<'a, DefinitionTwo, Category, CategoryKeys>
    for CategorySecondaryKeys
{
    type PrimaryKey = CategoryID;
}
impl<'a> NetabaseModelRelationalKey<'a, DefinitionTwo, Category, CategoryKeys>
    for CategoryRelationalKeys
{
}

// --- Category redb implementations ---

impl Value for CategoryID {
    type SelfType<'a> = Self;
    type AsBytes<'a> = Cow<'a, [u8]>;

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        CategoryID(String::from_utf8(data.to_vec()).unwrap())
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
    {
        Cow::Owned(value.0.as_bytes().to_vec())
    }

    fn fixed_width() -> Option<usize> {
        None
    }
    fn type_name() -> redb::TypeName {
        redb::TypeName::new(std::any::type_name::<Self>())
    }
}
impl Key for CategoryID {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

// Helper macro for implementing redb::Value and redb::Key for owned types
macro_rules! impl_redb_value_key_for_owned {
    ($type:ty) => {
        impl Value for $type {
            type SelfType<'a> = $type;
            type AsBytes<'a> = Cow<'a, [u8]>;

            fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
            where
                Self: 'a,
            {
                bincode::decode_from_slice(data, bincode::config::standard())
                    .unwrap()
                    .0
            }

            fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
            where
                Self: 'a,
            {
                Cow::Owned(bincode::encode_to_vec(value, bincode::config::standard()).unwrap())
            }

            fn fixed_width() -> Option<usize> {
                None
            }
            fn type_name() -> redb::TypeName {
                redb::TypeName::new(std::any::type_name::<$type>())
            }
        }

        impl Key for $type {
            fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                data1.cmp(data2)
            }
        }
    };
}

impl_redb_value_key_for_owned!(DefinitionTwoSubscriptions);
impl_redb_value_key_for_owned!(Category);
impl_redb_value_key_for_owned!(CategorySecondaryKeys);
impl_redb_value_key_for_owned!(CategoryRelationalKeys);
impl_redb_value_key_for_owned!(CategorySubscriptions);
impl_redb_value_key_for_owned!(CategoryBlobKeys);
impl_redb_value_key_for_owned!(CategoryBlobItem);

impl<'db> RedbNetbaseModel<'db, DefinitionTwo> for Category {
    type RedbTables = ModelOpenTables<'db, 'db, DefinitionTwo, Self>;
    type TableV = Category;
}

// --- Global Enums ---

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Encode)]
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
            _ => Err(bincode::error::DecodeError::Other(
                "Invalid GlobalDefinition variant",
            )),
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
    TryInto,
    From,
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
            return Ok(Self::User(user));
        } else if let Ok(post) = Post::decode(decoder) {
            return Ok(Self::Post(post));
        } else {
            return Err(bincode::error::DecodeError::Other("Failed to decode"));
        }
    }
}

impl<'de> BorrowDecode<'de, ()> for Definition {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = ()>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        if let Ok(user) = User::borrow_decode(decoder) {
            return Ok(Self::User(user));
        } else if let Ok(post) = Post::borrow_decode(decoder) {
            return Ok(Self::Post(post));
        } else {
            return Err(bincode::error::DecodeError::Other("Failed to decode"));
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
    DefinitionSubscriptionRegistry, NetabaseDefinitionSubscriptionKeys, SubscriptionEntry,
};

// Implement NetabaseDefinitionSubscriptionKeys for DefinitionSubscriptions
impl NetabaseDefinitionSubscriptionKeys for DefinitionSubscriptions {}

impl NetabaseDefinition for Definition {
    type TreeNames = DefinitionTreeNames;
    type DefKeys = DefinitionKeys;
    type SubscriptionKeys = DefinitionSubscriptions;
    type SubscriptionKeysDiscriminant = DefinitionSubscriptionsDiscriminants;

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
}

#[derive(Clone, Debug, PartialEq)]
pub enum DefinitionTreeNames {
    User(ModelTreeNames<'static, Definition, User>),
    Post(ModelTreeNames<'static, Definition, Post>),
}

impl Default for DefinitionTreeNames {
    fn default() -> Self {
        // Return User variant as default (arbitrary choice since this is a ZST-like enum)
        DefinitionTreeNames::User(User::TREE_NAMES)
    }
}

impl TryInto<DiscriminantTableName<Definition>> for DefinitionTreeNames {
    type Error = ();

    fn try_into(self) -> Result<DiscriminantTableName<Definition>, Self::Error> {
        // Convert the discriminant type to match the expected type
        // This trait bound seems to expect DiscriminantTableName<Definition>
        // but ModelTreeNames.main is DiscriminantTableName<DefinitionDiscriminants>
        // For now, we'll return an error as this conversion doesn't seem to be used
        Err(())
    }
}

impl NetabaseDefinitionTreeNames<Definition> for DefinitionTreeNames {
    fn get_tree_names(discriminant: DefinitionDiscriminants) -> Vec<Self> {
        match discriminant {
            DefinitionDiscriminants::User => vec![DefinitionTreeNames::User(User::TREE_NAMES)],
            DefinitionDiscriminants::Post => vec![DefinitionTreeNames::Post(Post::TREE_NAMES)],
        }
    }

    fn get_model_tree<M: NetabaseModel<Definition>>(&self) -> Option<M>
    where
        for<'a> Self: From<ModelTreeNames<'a, Self, M>>,
        for<'a> <<M as NetabaseModel<Definition>>::Keys as NetabaseModelKeys<Definition, M>>::Secondary<'a>:
            IntoDiscriminant,
        for<'a> <<M as NetabaseModel<Definition>>::Keys as NetabaseModelKeys<Definition, M>>::Relational<'a>:
            IntoDiscriminant,
        for<'a> <<M as NetabaseModel<Definition>>::Keys as NetabaseModelKeys<Definition, M>>::Subscription<'a>:
            IntoDiscriminant,
        for<'a> <<<M as NetabaseModel<Definition>>::Keys as NetabaseModelKeys<Definition, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<Definition>>::Keys as NetabaseModelKeys<Definition, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<Definition>>::Keys as NetabaseModelKeys<Definition, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<Definition>>::Keys as NetabaseModelKeys<Definition, M>>::Blob<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
         <<M as NetabaseModel<Definition>>::Keys as NetabaseModelKeys<Definition, M>>::Subscription<'static>: 'static
    {
        // This is a type-level operation that extracts the model if it matches type M
        // The actual implementation is not straightforward without specialization
        // For now, returning None as this seems to be unused in practice
        None
    }
}

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
    EnumDiscriminants,
)]
#[strum_discriminants(name(DefinitionSubscriptionsDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum DefinitionSubscriptions {
    Topic1,
    Topic2,
    Topic3,
    Topic4,
}

impl_redb_value_key_for_owned!(DefinitionSubscriptions);

// --- Definition Two trait implementations ---

// Implement NetabaseDefinitionSubscriptionKeys for DefinitionTwoSubscriptions
impl NetabaseDefinitionSubscriptionKeys for DefinitionTwoSubscriptions {}

impl NetabaseDefinition for DefinitionTwo {
    type TreeNames = DefinitionTwoTreeNames;
    type DefKeys = DefinitionTwoKeys;
    type SubscriptionKeys = DefinitionTwoSubscriptions;
    type SubscriptionKeysDiscriminant = DefinitionTwoSubscriptionsDiscriminants;

    const SUBSCRIPTION_REGISTRY: DefinitionSubscriptionRegistry<'static, Self> =
        DefinitionSubscriptionRegistry::new(&[SubscriptionEntry {
            topic: "General",
            subscribers: &[DefinitionTwoDiscriminants::Category],
        }]);
}

#[derive(Clone, Debug, PartialEq)]
pub enum DefinitionTwoTreeNames {
    Category(ModelTreeNames<'static, DefinitionTwo, Category>),
}

impl Default for DefinitionTwoTreeNames {
    fn default() -> Self {
        DefinitionTwoTreeNames::Category(Category::TREE_NAMES)
    }
}

impl TryInto<DiscriminantTableName<DefinitionTwo>> for DefinitionTwoTreeNames {
    type Error = ();

    fn try_into(self) -> Result<DiscriminantTableName<DefinitionTwo>, Self::Error> {
        // Convert the discriminant type to match the expected type
        // This trait bound seems to expect DiscriminantTableName<DefinitionTwo>
        // but ModelTreeNames.main is DiscriminantTableName<DefinitionTwoDiscriminants>
        // For now, we'll return an error as this conversion doesn't seem to be used
        Err(())
    }
}

impl NetabaseDefinitionTreeNames<DefinitionTwo> for DefinitionTwoTreeNames {
    fn get_tree_names(discriminant: DefinitionTwoDiscriminants) -> Vec<Self> {
        match discriminant {
            DefinitionTwoDiscriminants::Category => vec![DefinitionTwoTreeNames::Category(Category::TREE_NAMES)],
        }
    }

    fn get_model_tree<M: NetabaseModel<DefinitionTwo>>(&self) -> Option<M>
    where
        for<'a> Self: From<ModelTreeNames<'a, Self, M>>,
        for<'a> <<M as NetabaseModel<DefinitionTwo>>::Keys as NetabaseModelKeys<DefinitionTwo, M>>::Secondary<'a>:
            IntoDiscriminant,
        for<'a> <<M as NetabaseModel<DefinitionTwo>>::Keys as NetabaseModelKeys<DefinitionTwo, M>>::Relational<'a>:
            IntoDiscriminant,
        for<'a> <<M as NetabaseModel<DefinitionTwo>>::Keys as NetabaseModelKeys<DefinitionTwo, M>>::Subscription<'a>:
            IntoDiscriminant,
        for<'a> <<<M as NetabaseModel<DefinitionTwo>>::Keys as NetabaseModelKeys<DefinitionTwo, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<DefinitionTwo>>::Keys as NetabaseModelKeys<DefinitionTwo, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<DefinitionTwo>>::Keys as NetabaseModelKeys<DefinitionTwo, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<DefinitionTwo>>::Keys as NetabaseModelKeys<DefinitionTwo, M>>::Blob<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
         <<M as NetabaseModel<DefinitionTwo>>::Keys as NetabaseModelKeys<DefinitionTwo, M>>::Subscription<'static>: 'static
    {
        // This is a type-level operation that extracts the model if it matches type M
        // The actual implementation is not straightforward without specialization
        // For now, returning None as this seems to be unused in practice
        None
    }
}

#[derive(Clone, Debug)]
pub enum DefinitionTwoKeys {
    Category(CategoryKeys),
}

impl NetabaseDefinitionKeys<DefinitionTwo> for DefinitionTwoKeys {}

impl RedbDefinition for DefinitionTwo {
    type ModelTableDefinition<'db> = RedbModelTableDefinitions<'db, Category, Self>;
}
