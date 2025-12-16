use crate::boilerplate_lib::{
    DefinitionTwo, DefinitionTwoDiscriminants, DefinitionTwoSubscriptions,
};
use netabase_store::databases::redb::transaction::ModelOpenTables;
use netabase_store::traits::registery::{
    models::{
        StoreKey, StoreKeyMarker, StoreValue, StoreValueMarker,
        keys::{
            NetabaseModelKeys, NetabaseModelPrimaryKey, NetabaseModelRelationalKey,
            NetabaseModelSecondaryKey, NetabaseModelSubscriptionKey,
        },
        model::{NetabaseModel, NetabaseModelMarker, RedbNetbaseModel},
        treenames::{DiscriminantTableName, ModelTreeNames},
    },
};

use bincode::{Decode, Encode};
use redb::{Key, Value};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use strum::{AsRefStr, EnumDiscriminants};

// --- Category Model ---

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Category {
    pub id: CategoryID,
    pub name: String,
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, EnumDiscriminants, Hash)]
#[strum_discriminants(name(CategorySubscriptionsDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum CategorySubscriptions {
    General(DefinitionTwoSubscriptions),
}

impl NetabaseModelSubscriptionKey<DefinitionTwo, Category, CategoryKeys> for CategorySubscriptions {}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash)]
pub struct CategoryID(pub String);
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash)]
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

#[derive(Clone, Debug)]
pub enum CategoryKeys {
    Primary(CategoryID),
    Secondary(CategorySecondaryKeys),
    Relational(CategoryRelationalKeys),
    Subscription(CategorySubscriptions),
}

// --- Category Implementation ---

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
        subscription: Some(&[
            DiscriminantTableName::new(
                CategorySubscriptionsDiscriminants::General,
                "DefinitionTwo:Subscription:General",
            ),
        ]),
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
        vec![CategorySubscriptions::General(DefinitionTwoSubscriptions::General)]
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
}

impl<'a> NetabaseModelPrimaryKey<'a, DefinitionTwo, Category, CategoryKeys> for CategoryID {}
impl<'a> NetabaseModelSecondaryKey<'a, DefinitionTwo, Category, CategoryKeys> for CategorySecondaryKeys {
    type PrimaryKey = CategoryID;
}
impl<'a> NetabaseModelRelationalKey<'a, DefinitionTwo, Category, CategoryKeys> for CategoryRelationalKeys {}

// --- Helpers ---

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

impl_redb_value_key_for_owned!(Category);
impl_redb_value_key_for_owned!(CategorySecondaryKeys);
impl_redb_value_key_for_owned!(CategoryRelationalKeys);
impl_redb_value_key_for_owned!(CategorySubscriptions);

impl<'db> RedbNetbaseModel<'db, DefinitionTwo> for Category {
    type RedbTables = ModelOpenTables<'db, 'db, DefinitionTwo, Self>;
}
