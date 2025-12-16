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

#[derive(Debug, Clone, Encode, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Post {
    pub id: PostID,
    pub title: String,
    pub author: RelationalLink<'static, Definition, Definition, User>,
}

// Manual impl for Deserialize to handle 'static lifetimes
impl<'de> serde::Deserialize<'de> for Post {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Id,
            Title,
            Author,
        }

        struct PostVisitor;

        impl<'de> serde::de::Visitor<'de> for PostVisitor {
            type Value = Post;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Post")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Post, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut title = None;
                let mut author = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        Field::Title => {
                            if title.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title = Some(map.next_value()?);
                        }
                        Field::Author => {
                            if author.is_some() {
                                return Err(serde::de::Error::duplicate_field("author"));
                            }
                            // Deserialize ignoring lifetime - safe because RelationalLink deserializes to Dehydrated/Owned
                            let link: RelationalLink<'_, Definition, Definition, User> = map.next_value()?;
                            author = Some(unsafe { std::mem::transmute(link) });
                        }
                    }
                }

                let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                let title = title.ok_or_else(|| serde::de::Error::missing_field("title"))?;
                let author = author.ok_or_else(|| serde::de::Error::missing_field("author"))?;

                Ok(Post {
                    id,
                    title,
                    author,
                })
            }
        }

        const FIELDS: &[&str] = &["id", "title", "author"];
        deserializer.deserialize_struct("Post", FIELDS, PostVisitor)
    }
}

impl Decode<()> for Post {
    fn decode<D: bincode::de::Decoder<Context = ()>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let id: PostID = PostID::decode(decoder)?;
        let title: String = String::decode(decoder)?;
        let author: RelationalLink<'static, Definition, Definition, User> =
            RelationalLink::<'static, Definition, Definition, User>::decode(decoder)?;
        Ok(Self { id, title, author })
    }
}

impl<'a> BorrowDecode<'a, ()> for Post {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'a, Context = ()>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let id: PostID = PostID::decode(decoder)?;
        let title: String = String::decode(decoder)?;
        let author: RelationalLink<'static, Definition, Definition, User> =
            RelationalLink::<'static, Definition, Definition, User>::decode(decoder)?;
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

impl NetabaseModelSubscriptionKey<Definition, Post, PostKeys> for PostSubscriptions {}

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

use netabase_store::traits::permissions::{ModelPermissions, AccessLevel, CrossAccessLevel};

impl NetabaseModel<Definition> for Post {
    type Keys = PostKeys;

    const TREE_NAMES: ModelTreeNames<'static, Definition, Self> = ModelTreeNames {
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

    const PERMISSIONS: ModelPermissions<'static, Definition> = ModelPermissions {
        // Outbound: Which models Post can access
        outbound: &[
            (crate::boilerplate_lib::DefinitionDiscriminants::User, AccessLevel::READ_ONLY),  // Post->author
        ],

        // Inbound: Which models can access Post
        inbound: &[
            (crate::boilerplate_lib::DefinitionDiscriminants::User, AccessLevel::READ_ONLY),  // User can read posts
        ],

        // Cross-definition access
        cross_definition: &[],  // No cross-definition relations
    };

    fn get_primary_key(&self) -> PostID {
        self.id.clone()
    }

    fn get_secondary_keys(&self) -> Vec<PostSecondaryKeys> {
        vec![PostSecondaryKeys::Title(PostTitle(self.title.clone()))]
    }

    fn get_relational_keys(&self) -> Vec<PostRelationalKeys> {
        vec![PostRelationalKeys::Author(
            self.author.get_primary_key().clone(),
        )]
    }

    fn get_subscription_keys(&self) -> Vec<PostSubscriptions> {
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

// Manual implementations for Post with lifetime
impl Value for Post {
    type SelfType<'a> = Self;
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
        redb::TypeName::new(std::any::type_name::<Self>())
    }
}

impl Key for Post {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl_redb_value_key_for_owned!(PostSecondaryKeys);
impl_redb_value_key_for_owned!(PostRelationalKeys);
impl_redb_value_key_for_owned!(PostSubscriptions);

// RedbNetbaseModel impls - only needs type def as TREE_NAMES is in NetabaseModel
impl<'db> RedbNetbaseModel<'db, Definition> for Post {
    type RedbTables = ModelOpenTables<'db, 'db, Definition, Self>;
}
