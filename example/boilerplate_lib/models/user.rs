use crate::{
    Category, CategoryID, Definition, DefinitionDiscriminants, DefinitionSubscriptions,
    DefinitionTwo, GlobalKeys,
};
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
}

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
                            let link: RelationalLink<'_, Definition, Definition, User> =
                                map.next_value()?;
                            partner = Some(unsafe { std::mem::transmute(link) });
                        }
                        Field::Category => {
                            if category.is_some() {
                                return Err(serde::de::Error::duplicate_field("category"));
                            }
                            // Deserialize ignoring lifetime
                            let link: RelationalLink<'_, Definition, DefinitionTwo, Category> =
                                map.next_value()?;
                            category = Some(unsafe { std::mem::transmute(link) });
                        }
                        Field::Subscriptions => {
                            if subscriptions.is_some() {
                                return Err(serde::de::Error::duplicate_field("subscriptions"));
                            }
                            subscriptions = Some(map.next_value()?);
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

                Ok(User {
                    id,
                    name,
                    age,
                    partner,
                    category,
                    subscriptions,
                })
            }
        }

        const FIELDS: &[&str] = &["id", "name", "age", "partner", "category", "subscriptions"];
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

        Ok(Self {
            id,
            name,
            age,
            partner,
            category,
            subscriptions,
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

        Ok(Self {
            id,
            name,
            age,
            partner,
            category,
            subscriptions,
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

#[derive(Clone, Debug)]
pub enum UserKeys {
    Primary(UserID),
    Secondary(UserSecondaryKeys),
    Relational(UserRelationalKeys),
    Subscription(UserSubscriptions),
}

// --- User Implementation ---

use netabase_store::traits::permissions::{AccessLevel, CrossAccessLevel, ModelPermissions};

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
    };

    const PERMISSIONS: ModelPermissions<'static, Definition> = ModelPermissions {
        // Outbound: Which models User can access
        outbound: &[
            // User can read/hydrate partner (another User)
            (
                DefinitionDiscriminants::User,
                AccessLevel::new(true, false, false, false, true), // read + hydrate only
            ),
            // User can read posts
            (DefinitionDiscriminants::Post, AccessLevel::READ_ONLY),
        ],

        // Inbound: Which models can access User
        inbound: &[
            (DefinitionDiscriminants::Post, AccessLevel::READ_ONLY), // Post->author
            (DefinitionDiscriminants::User, AccessLevel::READ_ONLY), // User->partner
        ],

        // Cross-definition access
        cross_definition: &[
            (GlobalKeys::Def2Category, CrossAccessLevel::READ), // User->category
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
}

impl StoreValueMarker<Definition> for User {}
impl StoreValueMarker<Definition> for UserID {}

impl StoreKeyMarker<Definition> for UserID {}
impl StoreKey<Definition, User> for UserID {}
impl StoreValue<Definition, UserID> for User {}

impl StoreKeyMarker<Definition> for UserSecondaryKeys {}
impl StoreKeyMarker<Definition> for UserRelationalKeys {}
impl StoreKeyMarker<Definition> for UserSubscriptions {}

impl StoreKey<Definition, UserID> for UserSecondaryKeys {}
impl StoreKey<Definition, UserID> for UserRelationalKeys {}
impl StoreKey<Definition, UserID> for UserSubscriptions {}

impl StoreValue<Definition, UserSecondaryKeys> for UserID {}
impl StoreValue<Definition, UserRelationalKeys> for UserID {}
impl StoreValue<Definition, UserSubscriptions> for UserID {}

impl NetabaseModelMarker<Definition> for User {}

impl NetabaseModelKeys<Definition, User> for UserKeys {
    type Primary<'a> = UserID;
    type Secondary<'a> = UserSecondaryKeys;
    type Relational<'a> = UserRelationalKeys;
    type Subscription<'a> = UserSubscriptions;
}

impl<'a> NetabaseModelPrimaryKey<'a, Definition, User, UserKeys> for UserID {}
impl<'a> NetabaseModelSecondaryKey<'a, Definition, User, UserKeys> for UserSecondaryKeys {
    type PrimaryKey = UserID;
}
impl<'a> NetabaseModelRelationalKey<'a, Definition, User, UserKeys> for UserRelationalKeys {}

// Manual impl for User
impl Value for User {
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

// RedbNetbaseModel impls - only needs type def as TREE_NAMES is in NetabaseModel
impl<'db> RedbNetbaseModel<'db, Definition> for User {
    type RedbTables = ModelOpenTables<'db, 'db, Definition, Self>;
}
