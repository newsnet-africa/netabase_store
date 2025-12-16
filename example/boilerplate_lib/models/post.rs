use crate::boilerplate_lib::models::user::{User, UserID};
use crate::boilerplate_lib::{Definition, DefinitionDiscriminants, DefinitionSubscriptions};
use netabase_store::databases::redb::transaction::ModelOpenTables;
use netabase_store::relational::RelationalLink;
use netabase_store::traits::registery::models::{
    StoreKey, StoreKeyMarker, StoreValue, StoreValueMarker,
    keys::{
        NetabaseModelKeys, NetabaseModelPrimaryKey, NetabaseModelRelationalKey,
        NetabaseModelSecondaryKey, NetabaseModelSubscriptionKey,
    },
    model::{NetabaseModel, NetabaseModelMarker, RedbNetbaseModel},
    treenames::{DiscriminantTableName, ModelTreeNames},
};

use bincode::{BorrowDecode, Decode, Encode};
use redb::{Key, Value};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use strum::{AsRefStr, EnumDiscriminants};

// --- Post Model ---

#[derive(Debug, Clone, Encode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Post<'a> {
    pub id: PostID,
    pub title: String,
    pub author: RelationalLink<'a, Definition<'a>, Definition<'a>, User<'a>>,
}

impl<'a, Context> BorrowDecode<'a, Context> for Post<'a> {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'a, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let id: PostID = PostID::decode(decoder)?;
        let title: String = String::decode(decoder)?;
        let author: RelationalLink<'a, Definition<'a>, Definition<'a>, User<'a>> =
            RelationalLink::<'a, Definition<'a>, Definition<'a>, User<'a>>::borrow_decode(decoder)?;
        Ok(Self { id, title, author })
    }
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
#[strum_discriminants(name(PostSubscriptionsDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum PostSubscriptions {
    Topic3(DefinitionSubscriptions),
    Topic4(DefinitionSubscriptions),
}

impl<'a> NetabaseModelSubscriptionKey<Definition<'a>, Post<'a>, PostKeys> for PostSubscriptions {}

#[derive(
    Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash,
)]
pub struct PostID(pub String);
#[derive(
    Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash,
)]
pub struct PostTitle(pub String);

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
#[strum_discriminants(name(PostSecondaryKeysDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum PostSecondaryKeys {
    Title(PostTitle),
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
#[strum_discriminants(name(PostRelationalKeysDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum PostRelationalKeys {
    Author(UserID),
}

#[derive(Clone, Debug)]
pub enum PostKeys {
    Primary(PostID),
    Secondary(PostSecondaryKeys),
    Relational(PostRelationalKeys),
    Subscription(PostSubscriptions),
}

// --- Post Implementation ---

impl<'a> NetabaseModel<Definition<'a>> for Post<'a> {
    type Keys = PostKeys;

    const TREE_NAMES: ModelTreeNames<'static, Definition<'a>, Self> = ModelTreeNames {
        main: DiscriminantTableName::new(
            DefinitionDiscriminants::Post,
            "Definition:Post:Primary:Main",
        ),
        secondary: &[DiscriminantTableName::new(
            PostSecondaryKeysDiscriminants::Title,
            "Definition:Post:Secondary:Title",
        )],
        relational: &[DiscriminantTableName::new(
            PostRelationalKeysDiscriminants::Author,
            "Definition:Post:Relational:Author",
        )],
        subscription: Some(&[
            DiscriminantTableName::new(
                PostSubscriptionsDiscriminants::Topic3,
                "Definition:Subscription:Topic3",
            ),
            DiscriminantTableName::new(
                PostSubscriptionsDiscriminants::Topic4,
                "Definition:Subscription:Topic4",
            ),
        ]),
    };

    fn get_primary_key<'a>(&'a self) -> PostID {
        self.id.clone()
    }

    fn get_secondary_keys<'a>(&'a self) -> Vec<PostSecondaryKeys> {
        vec![PostSecondaryKeys::Title(PostTitle(self.title.clone()))]
    }

    fn get_relational_keys<'a>(&'a self) -> Vec<PostRelationalKeys> {
        vec![PostRelationalKeys::Author(
            self.author.get_primary_key().clone(),
        )]
    }

    fn get_subscription_keys<'a>(&'a self) -> Vec<PostSubscriptions> {
        // Example: Post doesn't subscribe to anything for now
        vec![]
    }
}

impl StoreValueMarker<Definition> for Post {}
impl StoreValueMarker<Definition> for PostID {}

impl StoreKeyMarker<Definition> for PostID {}
impl StoreKey<Definition, Post> for PostID {}
impl StoreValue<Definition, PostID> for Post {}

impl StoreKeyMarker<Definition> for PostSecondaryKeys {}
impl StoreKeyMarker<Definition> for PostRelationalKeys {}
impl StoreKeyMarker<Definition> for PostSubscriptions {}

impl StoreKey<Definition, PostID> for PostSecondaryKeys {}
impl StoreKey<Definition, PostID> for PostRelationalKeys {}
impl StoreKey<Definition, PostID> for PostSubscriptions {}

impl StoreValue<Definition, PostSecondaryKeys> for PostID {}
impl StoreValue<Definition, PostRelationalKeys> for PostID {}
impl StoreValue<Definition, PostSubscriptions> for PostID {}

impl NetabaseModelMarker<Definition> for Post {}

impl NetabaseModelKeys<Definition, Post> for PostKeys {
    type Primary<'a> = PostID;
    type Secondary<'a> = PostSecondaryKeys;
    type Relational<'a> = PostRelationalKeys;
    type Subscription<'a> = PostSubscriptions;
}

impl<'a> NetabaseModelPrimaryKey<'a, Definition, Post, PostKeys> for PostID {}
impl<'a> NetabaseModelSecondaryKey<'a, Definition, Post, PostKeys> for PostSecondaryKeys {
    type PrimaryKey = PostID;
}
impl<'a> NetabaseModelRelationalKey<'a, Definition, Post, PostKeys> for PostRelationalKeys {}

// --- Helpers ---

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

impl Value for PostID {
    type SelfType<'a> = Self;
    type AsBytes<'a> = Cow<'a, [u8]>;

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        PostID(String::from_utf8(data.to_vec()).unwrap())
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
impl Key for PostID {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl_redb_value_key_for_owned!(Post);
impl_redb_value_key_for_owned!(PostSecondaryKeys);
impl_redb_value_key_for_owned!(PostRelationalKeys);
impl_redb_value_key_for_owned!(PostSubscriptions);

// RedbNetbaseModel impls - only needs type def as TREE_NAMES is in NetabaseModel
impl<'db> RedbNetbaseModel<'db, Definition> for Post {
    type RedbTables = ModelOpenTables<'db, 'db, Definition, Self>;
}
