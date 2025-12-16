use crate::boilerplate_lib::models::category::{Category, CategoryID};
use crate::boilerplate_lib::{
    Definition, DefinitionDiscriminants, DefinitionSubscriptions, DefinitionTwo,
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

use bincode::{BorrowDecode, Decode, Encode};
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

#[derive(Debug, Clone, Encode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct User<'a> {
    pub id: UserID,
    pub name: String,
    pub age: u8,
    pub partner: RelationalLink<'a, Definition<'a>, Definition<'a>, User<'a>>,
    pub category: RelationalLink<'a, Definition<'a>, DefinitionTwo, Category>,
    pub subscriptions: Vec<DefinitionSubscriptions>,
}

impl<'de, C> BorrowDecode<'de, C> for User<'de> {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let id: UserID = UserID::decode(decoder)?;
        let name: String = String::decode(decoder)?;
        let age: u8 = u8::decode(decoder)?;
        let partner: RelationalLink<'de, Definition, Definition, User<'de>> =
            RelationalLink::<'de, Definition, Definition, User<'de>>::borrow_decode(decoder)?;
        let category: RelationalLink<'de, Definition, DefinitionTwo, Category> =
            RelationalLink::<'de, Definition, DefinitionTwo, Category>::decode(decoder)?;
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

impl<'a> NetabaseModelSubscriptionKey<Definition<'a>, User<'a>, UserKeys> for UserSubscriptions {}

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

impl<'a> NetabaseModel<Definition<'a>> for User<'a> {
    type Keys = UserKeys;

    const TREE_NAMES: ModelTreeNames<'static, Definition<'a>, Self> = ModelTreeNames {
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

impl<'a> StoreValueMarker<Definition<'a>> for User<'a> {}
impl<'a> StoreValueMarker<Definition<'a>> for UserID {}

impl<'a> StoreKeyMarker<Definition<'a>> for UserID {}
impl<'a> StoreKey<Definition<'a>, User<'a>> for UserID {}
impl<'a> StoreValue<Definition<'a>, UserID> for User<'a> {}

impl<'a> StoreKeyMarker<Definition<'a>> for UserSecondaryKeys {}
impl<'a> StoreKeyMarker<Definition<'a>> for UserRelationalKeys {}
impl<'a> StoreKeyMarker<Definition<'a>> for UserSubscriptions {}

impl<'a> StoreKey<Definition<'a>, UserID> for UserSecondaryKeys {}
impl<'a> StoreKey<Definition<'a>, UserID> for UserRelationalKeys {}
impl<'a> StoreKey<Definition<'a>, UserID> for UserSubscriptions {}

impl<'a> StoreValue<Definition<'a>, UserSecondaryKeys> for UserID {}
impl<'a> StoreValue<Definition<'a>, UserRelationalKeys> for UserID {}
impl<'a> StoreValue<Definition<'a>, UserSubscriptions> for UserID {}

impl<'a> NetabaseModelMarker<Definition<'a>> for User<'a> {}

impl<'a> NetabaseModelKeys<Definition<'a>, User<'a>> for UserKeys {
    type Primary<'b> = UserID;
    type Secondary<'b> = UserSecondaryKeys;
    type Relational<'b> = UserRelationalKeys;
    type Subscription<'b> = UserSubscriptions;
}

impl<'a, 'b> NetabaseModelPrimaryKey<'b, Definition<'a>, User<'a>, UserKeys> for UserID {}
impl<'a, 'b> NetabaseModelSecondaryKey<'b, Definition<'a>, User<'a>, UserKeys>
    for UserSecondaryKeys
{
    type PrimaryKey = UserID;
}
impl<'a, 'b> NetabaseModelRelationalKey<'b, Definition<'a>, User<'a>, UserKeys>
    for UserRelationalKeys
{
}

// Manual impl for User
impl Value for User<'static> {
    type SelfType<'a> = User<'a>;
    type AsBytes<'a> = Cow<'a, [u8]>;

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        bincode::borrow_decode_from_slice(data, bincode::config::standard())
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

impl Key for User<'static> {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl_redb_value_key_for_owned!(UserSecondaryKeys);
impl_redb_value_key_for_owned!(UserRelationalKeys);
impl_redb_value_key_for_owned!(UserSubscriptions);
impl_redb_value_key_for_owned!(UserCategory);

// RedbNetbaseModel impls - only needs type def as TREE_NAMES is in NetabaseModel
impl<'db> RedbNetbaseModel<'db, Definition<'db>> for User<'static> {
    type RedbTables = ModelOpenTables<'db, 'db, Definition<'db>, Self>;
}
