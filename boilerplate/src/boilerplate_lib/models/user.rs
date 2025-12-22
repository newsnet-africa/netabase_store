use crate::boilerplate_lib::models::post::Post;
use crate::{
    Category,
    CategoryID,
    Definition,
    DefinitionDiscriminants,
    DefinitionSubscriptions,
    DefinitionTreeNames,
    DefinitionTwo,
};
use netabase_store::blob::NetabaseBlobItem;
use netabase_store::databases::redb::transaction::ModelOpenTables;
use netabase_store::relational::RelationalLink;
use netabase_store::traits::registery::models::{
    StoreKey,
    StoreKeyMarker,
    StoreValue,
    StoreValueMarker,
    keys::{
        NetabaseModelKeys,
        NetabaseModelPrimaryKey,
        NetabaseModelRelationalKey,
        NetabaseModelSecondaryKey,
        NetabaseModelSubscriptionKey,
        blob::NetabaseModelBlobKey,
    },
    model::{NetabaseModel, NetabaseModelMarker, RedbNetbaseModel},
    treenames::{DiscriminantTableName, ModelTreeNames},
};
use std::fmt::Display;

use bincode::{BorrowDecode, Decode, Encode};
use derive_more::Display;
use redb::{Key, Value};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use strum::{AsRefStr, EnumDiscriminants};

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

// --- User Model ---

#[derive(Debug, Clone, Encode, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct User {
    pub id: UserID,
    pub name: String,
    pub age: u8,
    pub partner: RelationalLink<'static, Definition, Definition, User>,
    pub category: RelationalLink<'static, Definition, DefinitionTwo, Category>,
    pub subscriptions: Vec<DefinitionSubscriptions>,
    pub bio: LargeUserFile,
    pub another: AnotherLargeUserFile,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct LargeUserFile {
    pub data: Vec<u8>,
    pub metadata: String,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct AnotherLargeUserFile(pub Vec<u8>);

use std::fmt;

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Extract simple fields
        let id = format!("{:?}", self.id); // requires Display on UserID
        let name = &self.name;
        let age = self.age;

        // For RelationalLink, show either "none" or the referenced id if available.
        let partner = &self.partner.clone().dehydrate();
        let category = &self.category.clone().dehydrate();

        // Subscriptions as comma-separated list
        let subs: Vec<String> = self
            .subscriptions
            .iter()
            .map(|s| match s {
                DefinitionSubscriptions::Topic1 => "Topic1".to_string(),
                DefinitionSubscriptions::Topic2 => "Topic2".to_string(),
                DefinitionSubscriptions::Topic3 => "Topic3".to_string(),
                DefinitionSubscriptions::Topic4 => "Topic4".to_string(),
            })
            .collect();
        let subs = subs.join(", ");

        write!(
            f,
            "User {{ id: {}, name: \"{}\", age: {}, partner: {:?}, category: {:?}, subscriptions: [{}] }}",
            id, name, age, partner, category, subs
        )
    }
}

// Manual impl for Deserialize to handle 'static lifetimes
impl<'de> serde::Deserialize<'de> for User {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Id,
            Name,
            Age,
            Partner,
            Category,
            Subscriptions,
            Bio,
            Another,
        }

        struct UserVisitor;

        impl<'de> serde::de::Visitor<'de> for UserVisitor {
            type Value = User;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct User")
            }

            fn visit_map<V>(self, mut map: V) -> Result<User, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut name = None;
                let mut age = None;
                let mut partner = None;
                let mut category = None;
                let mut subscriptions = None;
                let mut bio = None;
                let mut another = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        Field::Name => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Age => {
                            if age.is_some() {
                                return Err(serde::de::Error::duplicate_field("age"));
                            }
                            age = Some(map.next_value()?);
                        }
                        Field::Partner => {
                            if partner.is_some() {
                                return Err(serde::de::Error::duplicate_field("partner"));
                            }
                            // Deserialize ignoring lifetime
                            let link: RelationalLink<'static, Definition, Definition, User> =
                                map.next_value()?;
                            partner = Some(link);
                        }
                        Field::Category => {
                            if category.is_some() {
                                return Err(serde::de::Error::duplicate_field("category"));
                            }
                            // Deserialize ignoring lifetime
                            let link: RelationalLink<'static, Definition, DefinitionTwo, Category> =
                                map.next_value()?;
                            category = Some(link);
                        }
                        Field::Subscriptions => {
                            if subscriptions.is_some() {
                                return Err(serde::de::Error::duplicate_field("subscriptions"));
                            }
                            subscriptions = Some(map.next_value()?);
                        }
                        Field::Bio => {
                            if bio.is_some() {
                                return Err(serde::de::Error::duplicate_field("bio"));
                            }
                            bio = Some(map.next_value()?);
                        }
                        Field::Another => {
                            if another.is_some() {
                                return Err(serde::de::Error::duplicate_field("another"));
                            }
                            another = Some(map.next_value()?);
                        }
                    }
                }

                let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                let name = name.ok_or_else(|| serde::de::Error::missing_field("name"))?;
                let age = age.ok_or_else(|| serde::de::Error::missing_field("age"))?;
                let partner = partner.ok_or_else(|| serde::de::Error::missing_field("partner"))?;
                let category =
                    category.ok_or_else(|| serde::de::Error::missing_field("category"))?;
                let subscriptions = subscriptions
                    .ok_or_else(|| serde::de::Error::missing_field("subscriptions"))?;
                let bio = bio.unwrap_or_default();
                let another = another.unwrap_or_default();

                Ok(User {
                    id,
                    name,
                    age,
                    partner,
                    category,
                    subscriptions,
                    bio,
                    another,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "id",
            "name",
            "age",
            "partner",
            "category",
            "subscriptions",
            "bio",
            "another",
        ];
        deserializer.deserialize_struct("User", FIELDS, UserVisitor)
    }
}

impl Decode<()> for User {
    fn decode<D: bincode::de::Decoder<Context = ()>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let id: UserID = UserID::decode(decoder)?;
        let name: String = String::decode(decoder)?;
        let age: u8 = u8::decode(decoder)?;
        let partner: RelationalLink<'static, Definition, Definition, User> =
            RelationalLink::<'static, Definition, Definition, User>::decode(decoder)?;
        let category: RelationalLink<'static, Definition, DefinitionTwo, Category> =
            RelationalLink::<'static, Definition, DefinitionTwo, Category>::decode(decoder)?;
        let subscriptions: Vec<DefinitionSubscriptions> =
            Vec::<DefinitionSubscriptions>::decode(decoder)?;
        let bio: LargeUserFile = LargeUserFile::decode(decoder)?;
        let another: AnotherLargeUserFile = AnotherLargeUserFile::decode(decoder)?;

        Ok(Self {
            id,
            name,
            age,
            partner,
            category,
            subscriptions,
            bio,
            another,
        })
    }
}

impl<'de> BorrowDecode<'de, ()> for User {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = ()>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let id: UserID = UserID::decode(decoder)?;
        let name: String = String::decode(decoder)?;
        let age: u8 = u8::decode(decoder)?;
        let partner: RelationalLink<'static, Definition, Definition, User> =
            RelationalLink::<'static, Definition, Definition, User>::decode(decoder)?;
        let category: RelationalLink<'static, Definition, DefinitionTwo, Category> =
            RelationalLink::<'static, Definition, DefinitionTwo, Category>::decode(decoder)?;
        let subscriptions: Vec<DefinitionSubscriptions> =
            Vec::<DefinitionSubscriptions>::decode(decoder)?;
        let bio: LargeUserFile = LargeUserFile::decode(decoder)?;
        let another: AnotherLargeUserFile = AnotherLargeUserFile::decode(decoder)?;

        Ok(Self {
            id,
            name,
            age,
            partner,
            category,
            subscriptions,
            bio,
            another,
        })
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
#[strum_discriminants(name(UserSubscriptionsDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum UserSubscriptions {
    Topic1(DefinitionSubscriptions),
    Topic2(DefinitionSubscriptions),
}

impl NetabaseModelSubscriptionKey<Definition, User, UserKeys> for UserSubscriptions {}

impl From<DefinitionSubscriptions> for UserSubscriptions {
    fn from(value: DefinitionSubscriptions) -> Self {
        match value {
            DefinitionSubscriptions::Topic1 => UserSubscriptions::Topic1(value),
            DefinitionSubscriptions::Topic2 => UserSubscriptions::Topic2(value),
            _ => panic!("Unsupported subscription topic for User model"),
        }
    }
}

impl TryInto<DefinitionSubscriptions> for UserSubscriptions {
    type Error = ();

    fn try_into(self) -> Result<DefinitionSubscriptions, Self::Error> {
        match self {
            UserSubscriptions::Topic1(def_sub) => Ok(def_sub),
            UserSubscriptions::Topic2(def_sub) => Ok(def_sub),
        }
    }
}

#[derive(
    Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash,
)]
pub struct UserID(pub String);
#[derive(
    Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash,
)]
pub struct UserName(pub String);
#[derive(
    Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash,
)]
pub struct UserAge(pub u8);

#[derive(
    Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash,
)]
pub struct UserPartner(pub UserID);
#[derive(
    Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash,
)]
pub struct UserCategory(pub CategoryID);

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
#[strum_discriminants(name(UserSecondaryKeysDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum UserSecondaryKeys {
    Name(UserName),
    Age(UserAge),
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
#[strum_discriminants(name(UserRelationalKeysDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum UserRelationalKeys {
    Partner(UserPartner),
    Category(UserCategory),
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
#[strum_discriminants(name(UserBlobKeysDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum UserBlobKeys {
    LargeUserFile { owner: UserID },
    AnotherLargeUserFile {  owner: UserID },
}

impl<'a> NetabaseModelBlobKey<'a, Definition, User, UserKeys> for UserBlobKeys {
    type PrimaryKey = UserID;
    type BlobItem = UserBlobItem;
}

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, Serialize, Deserialize,
)]
pub enum UserBlobItem {
    LargeUserFile{index: u8, value: Vec<u8>},
    AnotherLargeUserFile{index: u8, value: Vec<u8>},
}

#[derive(Clone, Debug)]
pub enum UserKeys {
    Primary(UserID),
    Secondary(UserSecondaryKeys),
    Relational(UserRelationalKeys),
    Subscription(UserSubscriptions),
    Blob(UserBlobKeys),
}

// --- User Implementation ---

impl NetabaseModel<Definition> for User {
    type Keys = UserKeys;

    const TREE_NAMES: ModelTreeNames<'static, Definition, Self> = ModelTreeNames {
        main: DiscriminantTableName::new(DefinitionDiscriminants::User, "User:User:Primary:Main"),
        secondary: &[
            DiscriminantTableName::new(
                UserSecondaryKeysDiscriminants::Name,
                "Definition:User:Secondary:Name",
            ),
            DiscriminantTableName::new(
                UserSecondaryKeysDiscriminants::Age,
                "Definition:User:Secondary:Age",
            ),
        ],
        relational: &[
            DiscriminantTableName::new(
                UserRelationalKeysDiscriminants::Partner,
                "Definition:User:Relational:Partner",
            ),
            DiscriminantTableName::new(
                UserRelationalKeysDiscriminants::Category,
                "Definition:User:Relational:Category",
            ),
        ],
        subscription: Some(&[
            DiscriminantTableName::new(
                UserSubscriptionsDiscriminants::Topic1,
                "Definition:Subscription:Topic1",
            ),
            DiscriminantTableName::new(
                UserSubscriptionsDiscriminants::Topic2,
                "Definition:Subscription:Topic2",
            ),
        ]),
        blob: &[
            DiscriminantTableName::new(
                UserBlobKeysDiscriminants::LargeUserFile,
                "Definition:User:Blob:LargeUserFile",
            ),
            DiscriminantTableName::new(
                UserBlobKeysDiscriminants::AnotherLargeUserFile,
                "Definition:User:Blob:AnotherLargeUserFile",
            ),
        ],
    };

    fn get_primary_key<'b>(&'b self) -> UserID {
        self.id.clone()
    }

    fn get_secondary_keys<'b>(&'b self) -> Vec<UserSecondaryKeys> {
        vec![
            UserSecondaryKeys::Name(UserName(self.name.clone())),
            UserSecondaryKeys::Age(UserAge(self.age)),
        ]
    }

    fn get_relational_keys<'b>(&'b self) -> Vec<UserRelationalKeys> {
        vec![
            UserRelationalKeys::Partner(UserPartner(self.partner.get_primary_key().clone())),
            UserRelationalKeys::Category(UserCategory(self.category.get_primary_key().clone())),
        ]
    }

    fn get_subscription_keys<'b>(&'b self) -> Vec<UserSubscriptions> {
        self.subscriptions
            .iter()
            .map(|sub| {
                match sub {
                    DefinitionSubscriptions::Topic1 => UserSubscriptions::Topic1(sub.clone()),
                    DefinitionSubscriptions::Topic2 => UserSubscriptions::Topic2(sub.clone()),
                    _ => panic!("Subscription not supported by User model"), // Or handle gracefully
                }
            })
            .collect()
    }

    fn get_blob_entries<'a>(
        &'a self,
    ) -> Vec<Vec<(
        <Self::Keys as NetabaseModelKeys<Definition, Self>>::Blob<'a>,
        <<Self::Keys as NetabaseModelKeys<Definition, Self>>::Blob<'a> as NetabaseModelBlobKey< 
            'a,
            Definition,
            Self,
            Self::Keys,
        >>::BlobItem,
    )>> {
        let mut bio_entries = Vec::new();
        for blob in self.bio.split_into_blobs() {
            bio_entries.push((
                UserBlobKeys::LargeUserFile {
                    owner: self.id.clone(),
                },
                blob,
            ));
        }

        let mut another_entries = Vec::new();
        for blob in self.another.split_into_blobs() {
            another_entries.push((
                UserBlobKeys::AnotherLargeUserFile {
                    owner: self.id.clone(),
                },
                blob,
            ));
        }

        vec![bio_entries, another_entries]
    }
}

impl StoreValueMarker<Definition> for User {}
impl StoreValueMarker<Definition> for UserID {}

impl StoreKeyMarker<Definition> for UserID {}
impl StoreKey<Definition, User> for UserID {}
impl StoreValue<Definition, UserID> for User {}

impl StoreKeyMarker<Definition> for UserSecondaryKeys {}
impl StoreKeyMarker<Definition> for UserRelationalKeys {}
impl StoreKeyMarker<Definition> for UserSubscriptions {}
impl StoreKeyMarker<Definition> for UserBlobKeys {}
impl StoreKeyMarker<Definition> for UserBlobItem {}

impl StoreKey<Definition, UserID> for UserSecondaryKeys {}
impl StoreKey<Definition, UserID> for UserRelationalKeys {}
impl StoreKey<Definition, UserID> for UserSubscriptions {}

impl StoreValue<Definition, UserSecondaryKeys> for UserID {}
impl StoreValue<Definition, UserRelationalKeys> for UserID {}
impl StoreValue<Definition, UserSubscriptions> for UserID {}

impl NetabaseModelMarker<Definition> for User {}

impl NetabaseModelKeys<Definition, User> for UserKeys {
    type Primary<'user> = UserID;
    type Secondary<'user> = UserSecondaryKeys;
    type Relational<'user> = UserRelationalKeys;
    type Subscription<'user> = UserSubscriptions;
    type Blob<'user> = UserBlobKeys;
}

impl<'a> NetabaseModelPrimaryKey<'a, Definition, User, UserKeys> for UserID {}
impl<'a> NetabaseModelSecondaryKey<'a, Definition, User, UserKeys> for UserSecondaryKeys {
    type PrimaryKey = UserID;
}
impl<'a> NetabaseModelRelationalKey<'a, Definition, User, UserKeys> for UserRelationalKeys {}

// Manual impl for User
impl Value for User {
    type SelfType<'a>
        = User
    where
        Self: 'a;
    type AsBytes<'a>
        = Cow<'a, [u8]>
    where
        Self: 'a;

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

impl Key for User {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl Value for UserID {
    type SelfType<'a> = Self;
    type AsBytes<'a> = Cow<'a, [u8]>;

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        UserID(String::from_utf8(data.to_vec()).unwrap())
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

impl Key for UserID {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl_redb_value_key_for_owned!(UserSecondaryKeys);
impl_redb_value_key_for_owned!(UserRelationalKeys);
impl_redb_value_key_for_owned!(UserSubscriptions);
impl_redb_value_key_for_owned!(UserCategory);
impl_redb_value_key_for_owned!(UserBlobKeys);
impl_redb_value_key_for_owned!(UserBlobItem);

impl NetabaseBlobItem for LargeUserFile {
    type Blobs = UserBlobItem;

    fn wrap_blob(index: u8, data: Vec<u8>) -> Self::Blobs {
        UserBlobItem::LargeUserFile { index, value: data }
    }

    fn unwrap_blob(blob: &Self::Blobs) -> Option<(u8, Vec<u8>)> {
        if let UserBlobItem::LargeUserFile { index, value } = blob {
            Some((*index, value.clone()))
        } else {
            None
        }
    }
}

impl NetabaseBlobItem for AnotherLargeUserFile {
    type Blobs = UserBlobItem;

    fn wrap_blob(index: u8, data: Vec<u8>) -> Self::Blobs {
        UserBlobItem::AnotherLargeUserFile { index, value: data }
    }

    fn unwrap_blob(blob: &Self::Blobs) -> Option<(u8, Vec<u8>)> {
        if let UserBlobItem::AnotherLargeUserFile { index, value } = blob {
            Some((*index, value.clone()))
        } else {
            None
        }
    }
}

impl NetabaseBlobItem for UserBlobItem {
    type Blobs = Self;

    fn wrap_blob(_index: u8, _data: Vec<u8>) -> Self::Blobs {
        unimplemented!("UserBlobItem is the blob container, not the content")
    }

    fn unwrap_blob(_blob: &Self::Blobs) -> Option<(u8, Vec<u8>)> {
        unimplemented!("UserBlobItem is the blob container, not the content")
    }

    fn split_into_blobs(&self) -> Vec<Self::Blobs> {
        vec![self.clone()]
    }

    fn reconstruct_from_blobs(blobs: Vec<Self::Blobs>) -> Self {
        blobs.into_iter().next().unwrap()
    }
}

// RedbNetbaseModel impls - only needs type def as TREE_NAMES is in NetabaseModel
impl<'db> RedbNetbaseModel<'db, Definition> for User {
    type RedbTables = ModelOpenTables<'db, 'db, Definition, Self>;
    type TableV = User;
}