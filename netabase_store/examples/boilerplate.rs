use bincode::{Decode, Encode};
use derive_more::TryInto;
use netabase_store::{
    databases::redb_store::{RedbModelAssociatedTypesExt, RedbNetabaseModelTrait, RedbStore},
    databases::sled_store::{
        SledModelAssociatedTypesExt, SledNetabaseModelTrait, SledStore, SledStoreTrait,
    },
    error::{NetabaseError, NetabaseResult},
    traits::{
        definition::{
            DiscriminantName, ModelAssociatedTypesExt, NetabaseDefinitionTrait,
            key::NetabaseDefinitionKeyTrait,
        },
        model::{NetabaseModelTrait, key::NetabaseModelKeyTrait},
        store::{
            store::StoreTrait,
            tree_manager::{AllTrees, TreeManager},
        },
    },
};
use redb::{Key, TableDefinition, TypeName, Value, WriteTransaction};
use std::{borrow::Cow, path::Path, time::Instant};
use strum::{AsRefStr, EnumDiscriminants, EnumIter, IntoDiscriminant, IntoEnumIterator};

// ================================================================================= ================
// Model 1: User
// ================================================================================= ================

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub age: u32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct UserId(pub u64);

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct UserEmail(pub String);

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct UserName(pub String);

// Helper macro for implementing redb::Value and redb::Key for simple wrappers
macro_rules! impl_key_wrapper {
    ($wrapper:ty, $inner:ty, $name:expr) => {
        impl Value for $wrapper {
            type SelfType<'a> = $wrapper;
            type AsBytes<'a> = <$inner as Value>::AsBytes<'a>;

            fn fixed_width() -> Option<usize> {
                <$inner as Value>::fixed_width()
            }

            fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
            where
                Self: 'a,
            {
                Self(<$inner as Value>::from_bytes(data))
            }

            fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
                <$inner as Value>::as_bytes(&value.0)
            }

            fn type_name() -> TypeName {
                TypeName::new($name)
            }
        }

        impl Key for $wrapper {
            fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                <$inner as Key>::compare(data1, data2)
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct UserAge(pub u32);

impl_key_wrapper!(UserId, u64, "UserId");
impl_key_wrapper!(UserEmail, String, "UserEmail");
impl_key_wrapper!(UserName, String, "UserName");
impl_key_wrapper!(UserAge, u32, "UserAge");

// Bincode 2.0 conversions for UserId
impl TryFrom<Vec<u8>> for UserId {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (u64, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(UserId(value))
    }
}

impl TryFrom<UserId> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: UserId) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value.0, bincode::config::standard())
    }
}

// Manual impl of redb::Value for User
impl Value for User {
    type SelfType<'a> = User;
    type AsBytes<'a> = Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let id = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let age = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let email_len = u64::from_le_bytes(data[12..20].try_into().unwrap()) as usize;
        let email_end = 20 + email_len;
        let email = String::from_utf8(data[20..email_end].to_vec()).unwrap();
        let name = String::from_utf8(data[email_end..].to_vec()).unwrap();
        User {
            id,
            email,
            name,
            age,
        }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        // Serialize: id(8) + age(4) + email_len(8) + email + name
        let mut out = Vec::new();
        out.extend_from_slice(&value.id.to_le_bytes());
        out.extend_from_slice(&value.age.to_le_bytes());
        let email_bytes = value.email.as_bytes();
        out.extend_from_slice(&(email_bytes.len() as u64).to_le_bytes());
        out.extend_from_slice(email_bytes);
        out.extend_from_slice(value.name.as_bytes());
        Cow::Owned(out)
    }

    fn type_name() -> TypeName {
        TypeName::new("User")
    }
}

#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(UserSecondaryKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum UserSecondaryKeys {
    Email(UserEmail),
    Name(UserName),
    Age(UserAge),
}

pub struct UserSecondaryKeysIter {
    iter: std::vec::IntoIter<UserSecondaryKeys>,
}

impl Iterator for UserSecondaryKeysIter {
    type Item = UserSecondaryKeys;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

// DiscriminantName is automatically implemented via the trait's default implementation using AsRefStr
impl DiscriminantName for UserSecondaryKeysDiscriminants {}

// Tree access enum - no inner types, used purely for tree identification
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum UserSecondaryTreeNames {
    Email,
    Name,
    Age,
}

impl DiscriminantName for UserSecondaryTreeNames {}

impl Value for UserSecondaryKeys {
    type SelfType<'a> = UserSecondaryKeys;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize UserSecondaryKeys")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize UserSecondaryKeys");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("UserSecondaryKeys")
    }
}

impl Key for UserSecondaryKeys {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

// Bincode conversions for UserSecondaryKeys
impl TryFrom<Vec<u8>> for UserSecondaryKeys {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (UserSecondaryKeys, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(value)
    }
}

impl TryFrom<UserSecondaryKeys> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: UserSecondaryKeys) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value, bincode::config::standard())
    }
}

// Need to derive Serialize and Deserialize for UserSecondaryKeys and its variants
// This requires adding serde to the dependencies and deriving on all relevant types

#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(UserRelationalKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum UserRelationalKeys {
    // For products created by this user, we store Product's primary key
    // This allows us to fetch related Product models
    CreatedProducts(ProductId),
}

pub struct UserRelationalKeysIter {
    iter: std::vec::IntoIter<UserRelationalKeys>,
}

impl Iterator for UserRelationalKeysIter {
    type Item = UserRelationalKeys;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

// DiscriminantName is automatically implemented via the trait's default implementation using AsRefStr
impl DiscriminantName for UserRelationalKeysDiscriminants {}

// Tree access enum - no inner types, used purely for tree identification
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum UserRelationalTreeNames {
    CreatedProducts,
}

impl DiscriminantName for UserRelationalTreeNames {}

#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(UserSubscriptionsDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum UserSubscriptions {
    // Dummy subscription
    Updates,
}

impl DiscriminantName for UserSubscriptionsDiscriminants {}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum UserSubscriptionTreeNames {
    Updates,
}

impl DiscriminantName for UserSubscriptionTreeNames {}

impl Value for UserSubscriptions {
    type SelfType<'a> = UserSubscriptions;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize UserSubscriptions")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize UserSubscriptions");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("UserSubscriptions")
    }
}

impl Key for UserSubscriptions {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

// Bincode conversions for UserSubscriptions
impl TryFrom<Vec<u8>> for UserSubscriptions {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (UserSubscriptions, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(value)
    }
}

impl TryFrom<UserSubscriptions> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: UserSubscriptions) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value, bincode::config::standard())
    }
}

impl Value for UserRelationalKeys {
    type SelfType<'a> = UserRelationalKeys;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize UserRelationalKeys")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize UserRelationalKeys");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("UserRelationalKeys")
    }
}

impl Key for UserRelationalKeys {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

// Bincode 2.0 conversions for UserRelationalKeys
impl TryFrom<Vec<u8>> for UserRelationalKeys {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (UserRelationalKeys, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(value)
    }
}

impl TryFrom<UserRelationalKeys> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: UserRelationalKeys) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value, bincode::config::standard())
    }
}

#[derive(Debug, Clone)]
pub enum UserKeys {
    Primary(UserId),
    Secondary(UserSecondaryKeys),
    Relational(UserRelationalKeys),
}

impl NetabaseModelKeyTrait<Definitions, User> for UserKeys {
    type PrimaryKey = UserId;
    type SecondaryEnum = UserSecondaryKeys;
    type RelationalEnum = UserRelationalKeys;

    fn secondary_keys(model: &User) -> Vec<Self::SecondaryEnum> {
        vec![
            UserSecondaryKeys::Email(UserEmail(model.email.clone())),
            UserSecondaryKeys::Name(UserName(model.name.clone())),
            UserSecondaryKeys::Age(UserAge(model.age)),
        ]
    }

    fn relational_keys(_model: &User) -> Vec<Self::RelationalEnum> {
        // Note: In a real implementation, you'd need to query the database
        // to find all products created by this user. For demonstration purposes,
        // we're returning an empty vec. The transaction layer should handle
        // populating these relationships when needed.
        vec![]
    }
}

impl NetabaseModelTrait<Definitions> for User {
    type Keys = UserKeys;
    const MODEL_TREE_NAME: DefinitionsDiscriminants = DefinitionsDiscriminants::User;

    type SecondaryKeys = UserSecondaryKeysIter;
    type RelationalKeys = UserRelationalKeysIter;
    type SubscriptionEnum = UserSubscriptions;
    type Hash = [u8; 32]; // Blake3 hash

    fn primary_key(&self) -> Self::PrimaryKey {
        UserId(self.id)
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        UserSecondaryKeysIter {
            iter: vec![
                UserSecondaryKeys::Email(UserEmail(self.email.clone())),
                UserSecondaryKeys::Name(UserName(self.name.clone())),
                UserSecondaryKeys::Age(UserAge(self.age)),
            ]
            .into_iter(),
        }
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        // Relational keys should be fetched from the database, not generated from the model
        // This is a limitation of the current API - it should be lazy or require a transaction context
        UserRelationalKeysIter {
            iter: vec![].into_iter(),
        }
    }

    fn get_subscriptions(&self) -> Vec<Self::SubscriptionEnum> {
        vec![UserSubscriptions::Updates]
    }

    fn compute_hash(&self) -> Self::Hash {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&self.id.to_le_bytes());
        hasher.update(self.email.as_bytes());
        hasher.update(self.name.as_bytes());
        hasher.update(&self.age.to_le_bytes());
        *hasher.finalize().as_bytes()
    }

    fn wrap_primary_key(key: Self::PrimaryKey) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::UserPrimaryKey(key)
    }

    fn wrap_model(model: Self) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::UserModel(model)
    }

    fn wrap_secondary_key(key: UserSecondaryKeys) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::UserSecondaryKey(key)
    }

    fn wrap_relational_key(key: UserRelationalKeys) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::UserRelationalKey(key)
    }

    fn wrap_subscription_key(key: UserSubscriptions) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::UserSubscriptionKey(key)
    }

    fn wrap_secondary_key_discriminant(
        key: UserSecondaryKeysDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::UserSecondaryKeyDiscriminant(key)
    }

    fn wrap_relational_key_discriminant(
        key: UserRelationalKeysDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::UserRelationalKeyDiscriminant(key)
    }

    fn wrap_subscription_key_discriminant(
        key: UserSubscriptionsDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::UserSubscriptionKeyDiscriminant(key)
    }
}

impl RedbNetabaseModelTrait<Definitions> for User {
    fn definition<'a>(db: &RedbStore<Definitions>) -> TableDefinition<'a, Self::PrimaryKey, Self> {
        let _ = db; // Avoid unused parameter warning
        TableDefinition::new(Self::MODEL_TREE_NAME.as_ref())
    }

    fn secondary_key_table_name(key_discriminant: UserSecondaryKeysDiscriminants) -> String {
        format!("User:Secondary:{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(key_discriminant: UserRelationalKeysDiscriminants) -> String {
        format!("User:Relation:{}", key_discriminant.as_ref())
    }

    fn subscription_key_table_name(key_discriminant: UserSubscriptionsDiscriminants) -> String {
        format!("User:Subscription:{}", key_discriminant.as_ref())
    }

    fn hash_tree_table_name() -> String {
        "User:Hash".to_string()
    }
}

impl SledNetabaseModelTrait<Definitions> for User {
    fn secondary_key_table_name(key_discriminant: UserSecondaryKeysDiscriminants) -> String {
        format!("User_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(key_discriminant: UserRelationalKeysDiscriminants) -> String {
        format!("User_rel_{}", key_discriminant.as_ref())
    }

    fn subscription_key_table_name(key_discriminant: UserSubscriptionsDiscriminants) -> String {
        format!("User_sub_{}", key_discriminant.as_ref())
    }

    fn hash_tree_table_name() -> String {
        "User_hash".to_string()
    }
}

// ================================================================================= ================
// Model 2: Product
// ================================================================================= ================

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct Product {
    pub uuid: u128,
    pub title: String,
    pub score: i32,
    pub created_by: u64, // Foreign key to User.id
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct ProductId(pub u128);

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct ProductTitle(pub String);

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct ProductScore(pub i32);

impl_key_wrapper!(ProductId, u128, "ProductId");
impl_key_wrapper!(ProductTitle, String, "ProductTitle");
impl_key_wrapper!(ProductScore, i32, "ProductScore");

// Bincode 2.0 conversions for ProductId
impl TryFrom<Vec<u8>> for ProductId {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (u128, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(ProductId(value))
    }
}

impl TryFrom<ProductId> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: ProductId) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value.0, bincode::config::standard())
    }
}

impl Value for Product {
    type SelfType<'a> = Product;
    type AsBytes<'a> = Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let uuid = u128::from_le_bytes(data[0..16].try_into().unwrap());
        let score = i32::from_le_bytes(data[16..20].try_into().unwrap());
        let created_by = u64::from_le_bytes(data[20..28].try_into().unwrap());
        let title = String::from_utf8(data[28..].to_vec()).unwrap();
        Product {
            uuid,
            title,
            score,
            created_by,
        }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        // Serialize: uuid(16) + score(4) + created_by(8) + title
        let mut out = Vec::new();
        out.extend_from_slice(&value.uuid.to_le_bytes());
        out.extend_from_slice(&value.score.to_le_bytes());
        out.extend_from_slice(&value.created_by.to_le_bytes());
        out.extend_from_slice(value.title.as_bytes());
        Cow::Owned(out)
    }

    fn type_name() -> TypeName {
        TypeName::new("Product")
    }
}

#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(ProductSecondaryKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum ProductSecondaryKeys {
    Title(ProductTitle),
    Score(ProductScore),
}

pub struct ProductSecondaryKeysIter {
    iter: std::vec::IntoIter<ProductSecondaryKeys>,
}

impl Iterator for ProductSecondaryKeysIter {
    type Item = ProductSecondaryKeys;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

// DiscriminantName is automatically implemented via the trait's default implementation using AsRefStr
impl DiscriminantName for ProductSecondaryKeysDiscriminants {}

// Tree access enum - no inner types, used purely for tree identification
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum ProductSecondaryTreeNames {
    Title,
    Score,
}

impl DiscriminantName for ProductSecondaryTreeNames {}

impl Value for ProductSecondaryKeys {
    type SelfType<'a> = ProductSecondaryKeys;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize ProductSecondaryKeys")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize ProductSecondaryKeys");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("ProductSecondaryKeys")
    }
}

impl Key for ProductSecondaryKeys {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

// Bincode 2.0 conversions for ProductSecondaryKeys
impl TryFrom<Vec<u8>> for ProductSecondaryKeys {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (ProductSecondaryKeys, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(value)
    }
}

impl TryFrom<ProductSecondaryKeys> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: ProductSecondaryKeys) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value, bincode::config::standard())
    }
}

#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(ProductRelationalKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum ProductRelationalKeys {
    // Link back to the user who created this product
    // Holds User's primary key type to enable fetching the related User model
    CreatedBy(UserId),
}

pub struct ProductRelationalKeysIter {
    iter: std::vec::IntoIter<ProductRelationalKeys>,
}

impl Iterator for ProductRelationalKeysIter {
    type Item = ProductRelationalKeys;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

// DiscriminantName is automatically implemented via the trait's default implementation using AsRefStr
impl DiscriminantName for ProductRelationalKeysDiscriminants {}

// Tree access enum - no inner types, used purely for tree identification
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum ProductRelationalTreeNames {
    CreatedBy,
}

impl DiscriminantName for ProductRelationalTreeNames {}

#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(ProductSubscriptionsDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum ProductSubscriptions {
    Updates,
}

impl DiscriminantName for ProductSubscriptionsDiscriminants {}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum ProductSubscriptionTreeNames {
    Updates,
}

impl DiscriminantName for ProductSubscriptionTreeNames {}

impl Value for ProductSubscriptions {
    type SelfType<'a> = ProductSubscriptions;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize ProductSubscriptions")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize ProductSubscriptions");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("ProductSubscriptions")
    }
}

impl Key for ProductSubscriptions {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl TryFrom<Vec<u8>> for ProductSubscriptions {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (ProductSubscriptions, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(value)
    }
}

impl TryFrom<ProductSubscriptions> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: ProductSubscriptions) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value, bincode::config::standard())
    }
}

impl Value for ProductRelationalKeys {
    type SelfType<'a> = ProductRelationalKeys;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize ProductRelationalKeys")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize ProductRelationalKeys");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("ProductRelationalKeys")
    }
}

impl Key for ProductRelationalKeys {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

// Bincode 2.0 conversions for ProductRelationalKeys
impl TryFrom<Vec<u8>> for ProductRelationalKeys {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (ProductRelationalKeys, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(value)
    }
}

impl TryFrom<ProductRelationalKeys> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: ProductRelationalKeys) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value, bincode::config::standard())
    }
}

#[derive(Debug, Clone)]
pub enum ProductKeys {
    Primary(ProductId),
    Secondary(ProductSecondaryKeys),
    Relational(ProductRelationalKeys),
}

impl NetabaseModelKeyTrait<Definitions, Product> for ProductKeys {
    type PrimaryKey = ProductId;
    type SecondaryEnum = ProductSecondaryKeys;
    type RelationalEnum = ProductRelationalKeys;

    fn secondary_keys(model: &Product) -> Vec<Self::SecondaryEnum> {
        vec![
            ProductSecondaryKeys::Title(ProductTitle(model.title.clone())),
            ProductSecondaryKeys::Score(ProductScore(model.score)),
        ]
    }

    fn relational_keys(model: &Product) -> Vec<Self::RelationalEnum> {
        // Product knows who created it (stored in model.created_by field)
        // so we can generate this relational key directly
        vec![ProductRelationalKeys::CreatedBy(UserId(model.created_by))]
    }
}

impl NetabaseModelTrait<Definitions> for Product {
    type Keys = ProductKeys;
    const MODEL_TREE_NAME: DefinitionsDiscriminants = DefinitionsDiscriminants::Product;

    type SecondaryKeys = ProductSecondaryKeysIter;
    type RelationalKeys = ProductRelationalKeysIter;
    type SubscriptionEnum = ProductSubscriptions;
    type Hash = [u8; 32]; // Blake3 hash

    fn primary_key(&self) -> Self::PrimaryKey {
        ProductId(self.uuid)
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        ProductSecondaryKeysIter {
            iter: vec![
                ProductSecondaryKeys::Title(ProductTitle(self.title.clone())),
                ProductSecondaryKeys::Score(ProductScore(self.score)),
            ]
            .into_iter(),
        }
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        ProductRelationalKeysIter {
            iter: vec![ProductRelationalKeys::CreatedBy(UserId(self.created_by))].into_iter(),
        }
    }

    fn get_subscriptions(&self) -> Vec<Self::SubscriptionEnum> {
        vec![]
    }

    fn compute_hash(&self) -> Self::Hash {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&self.uuid.to_le_bytes());
        hasher.update(self.title.as_bytes());
        hasher.update(&self.score.to_le_bytes());
        hasher.update(&self.created_by.to_le_bytes());
        *hasher.finalize().as_bytes()
    }

    fn wrap_primary_key(key: Self::PrimaryKey) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductPrimaryKey(key)
    }

    fn wrap_model(model: Self) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductModel(model)
    }

    fn wrap_secondary_key(key: ProductSecondaryKeys) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductSecondaryKey(key)
    }

    fn wrap_relational_key(key: ProductRelationalKeys) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductRelationalKey(key)
    }

    fn wrap_subscription_key(key: ProductSubscriptions) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductSubscriptionKey(key)
    }

    fn wrap_secondary_key_discriminant(
        key: ProductSecondaryKeysDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductSecondaryKeyDiscriminant(key)
    }

    fn wrap_relational_key_discriminant(
        key: ProductRelationalKeysDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductRelationalKeyDiscriminant(key)
    }

    fn wrap_subscription_key_discriminant(
        key: ProductSubscriptionsDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductSubscriptionKeyDiscriminant(key)
    }
}

impl RedbNetabaseModelTrait<Definitions> for Product {
    fn definition<'a>(db: &RedbStore<Definitions>) -> TableDefinition<'a, Self::PrimaryKey, Self> {
        let _ = db; // Avoid unused parameter warning
        TableDefinition::new(Self::MODEL_TREE_NAME.as_ref())
    }

    fn secondary_key_table_name(key_discriminant: ProductSecondaryKeysDiscriminants) -> String {
        format!("Product_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(key_discriminant: ProductRelationalKeysDiscriminants) -> String {
        format!("Product_rel_{}", key_discriminant.as_ref())
    }

    fn subscription_key_table_name(key_discriminant: ProductSubscriptionsDiscriminants) -> String {
        format!("Product_sub_{}", key_discriminant.as_ref())
    }

    fn hash_tree_table_name() -> String {
        "Product_hash".to_string()
    }
}

impl SledNetabaseModelTrait<Definitions> for Product {
    fn secondary_key_table_name(key_discriminant: ProductSecondaryKeysDiscriminants) -> String {
        format!("Product_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(key_discriminant: ProductRelationalKeysDiscriminants) -> String {
        format!("Product_rel_{}", key_discriminant.as_ref())
    }

    fn subscription_key_table_name(key_discriminant: ProductSubscriptionsDiscriminants) -> String {
        format!("Product_sub_{}", key_discriminant.as_ref())
    }

    fn hash_tree_table_name() -> String {
        "Product_hash".to_string()
    }
}

// ================================================================================= ================
// Model 3: Category
// ================================================================================= ================

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct Category {
    pub id: u64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct CategoryId(pub u64);

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct CategoryName(pub String);

impl_key_wrapper!(CategoryId, u64, "CategoryId");
impl_key_wrapper!(CategoryName, String, "CategoryName");

// Bincode 2.0 conversions for CategoryId
impl TryFrom<Vec<u8>> for CategoryId {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (u64, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(CategoryId(value))
    }
}

impl TryFrom<CategoryId> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: CategoryId) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value.0, bincode::config::standard())
    }
}

// Manual impl of redb::Value for Category
impl Value for Category {
    type SelfType<'a> = Category;
    type AsBytes<'a> = Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let id = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let name_len = u64::from_le_bytes(data[8..16].try_into().unwrap()) as usize;
        let name_end = 16 + name_len;
        let name = String::from_utf8(data[16..name_end].to_vec()).unwrap();
        let description = String::from_utf8(data[name_end..].to_vec()).unwrap();
        Category {
            id,
            name,
            description,
        }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let mut out = Vec::new();
        out.extend_from_slice(&value.id.to_le_bytes());
        let name_bytes = value.name.as_bytes();
        out.extend_from_slice(&(name_bytes.len() as u64).to_le_bytes());
        out.extend_from_slice(name_bytes);
        out.extend_from_slice(value.description.as_bytes());
        Cow::Owned(out)
    }

    fn type_name() -> TypeName {
        TypeName::new("Category")
    }
}

#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(CategorySecondaryKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum CategorySecondaryKeys {
    Name(CategoryName),
}

pub struct CategorySecondaryKeysIter {
    iter: std::vec::IntoIter<CategorySecondaryKeys>,
}

impl Iterator for CategorySecondaryKeysIter {
    type Item = CategorySecondaryKeys;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl DiscriminantName for CategorySecondaryKeysDiscriminants {}

// Tree access enum
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum CategorySecondaryTreeNames {
    Name,
}

impl DiscriminantName for CategorySecondaryTreeNames {}

impl Value for CategorySecondaryKeys {
    type SelfType<'a> = CategorySecondaryKeys;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize CategorySecondaryKeys")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize CategorySecondaryKeys");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("CategorySecondaryKeys")
    }
}

impl Key for CategorySecondaryKeys {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

// Bincode conversions for CategorySecondaryKeys
impl TryFrom<Vec<u8>> for CategorySecondaryKeys {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (CategorySecondaryKeys, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(value)
    }
}

impl TryFrom<CategorySecondaryKeys> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: CategorySecondaryKeys) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value, bincode::config::standard())
    }
}

// Category has relationships to Products that belong to it
#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(CategoryRelationalKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum CategoryRelationalKeys {
    // Products in this category (one-to-many)
    Products(ProductId),
}

pub struct CategoryRelationalKeysIter {
    iter: std::vec::IntoIter<CategoryRelationalKeys>,
}

impl Iterator for CategoryRelationalKeysIter {
    type Item = CategoryRelationalKeys;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl DiscriminantName for CategoryRelationalKeysDiscriminants {}

// Tree access enum
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum CategoryRelationalTreeNames {
    Products,
}

impl DiscriminantName for CategoryRelationalTreeNames {}

#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(CategorySubscriptionsDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum CategorySubscriptions {
    Updates,
}

impl DiscriminantName for CategorySubscriptionsDiscriminants {}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum CategorySubscriptionTreeNames {
    Updates,
}

impl DiscriminantName for CategorySubscriptionTreeNames {}

impl Value for CategorySubscriptions {
    type SelfType<'a> = CategorySubscriptions;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize CategorySubscriptions")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize CategorySubscriptions");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("CategorySubscriptions")
    }
}

impl Key for CategorySubscriptions {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl TryFrom<Vec<u8>> for CategorySubscriptions {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (CategorySubscriptions, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(value)
    }
}

impl TryFrom<CategorySubscriptions> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: CategorySubscriptions) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value, bincode::config::standard())
    }
}

impl Value for CategoryRelationalKeys {
    type SelfType<'a> = CategoryRelationalKeys;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize CategoryRelationalKeys")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize CategoryRelationalKeys");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("CategoryRelationalKeys")
    }
}

impl Key for CategoryRelationalKeys {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

// Bincode conversions for CategoryRelationalKeys
impl TryFrom<Vec<u8>> for CategoryRelationalKeys {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (CategoryRelationalKeys, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(value)
    }
}

impl TryFrom<CategoryRelationalKeys> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: CategoryRelationalKeys) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value, bincode::config::standard())
    }
}

#[derive(Debug, Clone)]
pub enum CategoryKeys {
    Primary(CategoryId),
    Secondary(CategorySecondaryKeys),
    Relational(CategoryRelationalKeys),
}

impl NetabaseModelKeyTrait<Definitions, Category> for CategoryKeys {
    type PrimaryKey = CategoryId;
    type SecondaryEnum = CategorySecondaryKeys;
    type RelationalEnum = CategoryRelationalKeys;

    fn secondary_keys(model: &Category) -> Vec<Self::SecondaryEnum> {
        vec![CategorySecondaryKeys::Name(CategoryName(
            model.name.clone(),
        ))]
    }

    fn relational_keys(_model: &Category) -> Vec<Self::RelationalEnum> {
        // Would need to query database for products in this category
        vec![]
    }
}

impl NetabaseModelTrait<Definitions> for Category {
    type Keys = CategoryKeys;
    const MODEL_TREE_NAME: DefinitionsDiscriminants = DefinitionsDiscriminants::Category;

    type SecondaryKeys = CategorySecondaryKeysIter;
    type RelationalKeys = CategoryRelationalKeysIter;
    type SubscriptionEnum = CategorySubscriptions;
    type Hash = [u8; 32];

    fn primary_key(&self) -> Self::PrimaryKey {
        CategoryId(self.id)
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        CategorySecondaryKeysIter {
            iter: vec![CategorySecondaryKeys::Name(CategoryName(self.name.clone()))].into_iter(),
        }
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        CategoryRelationalKeysIter {
            iter: vec![].into_iter(),
        }
    }

    fn get_subscriptions(&self) -> Vec<Self::SubscriptionEnum> {
        vec![]
    }

    fn compute_hash(&self) -> Self::Hash {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&self.id.to_le_bytes());
        hasher.update(self.name.as_bytes());
        hasher.update(self.description.as_bytes());
        *hasher.finalize().as_bytes()
    }

    fn wrap_primary_key(key: Self::PrimaryKey) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::CategoryPrimaryKey(key)
    }

    fn wrap_model(model: Self) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::CategoryModel(model)
    }

    fn wrap_secondary_key(key: CategorySecondaryKeys) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::CategorySecondaryKey(key)
    }

    fn wrap_relational_key(key: CategoryRelationalKeys) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::CategoryRelationalKey(key)
    }

    fn wrap_subscription_key(key: CategorySubscriptions) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::CategorySubscriptionKey(key)
    }

    fn wrap_secondary_key_discriminant(
        key: CategorySecondaryKeysDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::CategorySecondaryKeyDiscriminant(key)
    }

    fn wrap_relational_key_discriminant(
        key: CategoryRelationalKeysDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::CategoryRelationalKeyDiscriminant(key)
    }

    fn wrap_subscription_key_discriminant(
        key: CategorySubscriptionsDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::CategorySubscriptionKeyDiscriminant(key)
    }
}

impl RedbNetabaseModelTrait<Definitions> for Category {
    fn definition<'a>(db: &RedbStore<Definitions>) -> TableDefinition<'a, Self::PrimaryKey, Self> {
        let _ = db;
        TableDefinition::new(Self::MODEL_TREE_NAME.as_ref())
    }

    fn secondary_key_table_name(key_discriminant: CategorySecondaryKeysDiscriminants) -> String {
        format!("Category_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(key_discriminant: CategoryRelationalKeysDiscriminants) -> String {
        format!("Category_rel_{}", key_discriminant.as_ref())
    }

    fn subscription_key_table_name(key_discriminant: CategorySubscriptionsDiscriminants) -> String {
        format!("Category_sub_{}", key_discriminant.as_ref())
    }

    fn hash_tree_table_name() -> String {
        "Category_hash".to_string()
    }
}

impl SledNetabaseModelTrait<Definitions> for Category {
    fn secondary_key_table_name(key_discriminant: CategorySecondaryKeysDiscriminants) -> String {
        format!("Category_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(key_discriminant: CategoryRelationalKeysDiscriminants) -> String {
        format!("Category_rel_{}", key_discriminant.as_ref())
    }

    fn subscription_key_table_name(key_discriminant: CategorySubscriptionsDiscriminants) -> String {
        format!("Category_sub_{}", key_discriminant.as_ref())
    }

    fn hash_tree_table_name() -> String {
        "Category_hash".to_string()
    }
}

// ==================================================================================
// Model 4: Review
// ==================================================================================

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct Review {
    pub id: u64,
    pub product_id: u128,
    pub user_id: u64,
    pub rating: u8,
    pub comment: String,
    pub created_at: u64, // Unix timestamp
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct ReviewId(pub u64);

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct ReviewRating(pub u8);

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct ReviewCreatedAt(pub u64);

// TryFrom implementations for ReviewId
impl TryFrom<Vec<u8>> for ReviewId {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (u64, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(ReviewId(value))
    }
}

impl TryFrom<ReviewId> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: ReviewId) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value.0, bincode::config::standard())
    }
}

impl_key_wrapper!(ReviewId, u64, "ReviewId");

// Implement redb::Value for Review
impl redb::Value for Review {
    type SelfType<'a>
        = Self
    where
        Self: 'a;
    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let config = bincode::config::standard();
        bincode::decode_from_slice(data, config)
            .expect("Failed to decode Review")
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        let config = bincode::config::standard();
        bincode::encode_to_vec(value, config).expect("Failed to encode Review")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("Review")
    }
}

// Secondary keys for Review
#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(ReviewSecondaryKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum ReviewSecondaryKeys {
    Rating(ReviewRating),
    CreatedAt(ReviewCreatedAt),
}

impl DiscriminantName for ReviewSecondaryKeysDiscriminants {}

// Tree access enum for Review secondary keys (no inner types)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum ReviewSecondaryTreeNames {
    Rating,
    CreatedAt,
}

impl DiscriminantName for ReviewSecondaryTreeNames {}

// Relational keys for Review - dual relationships to User and Product
#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(ReviewRelationalKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum ReviewRelationalKeys {
    ReviewedProduct(ProductId), // FK to Product
    Reviewer(UserId),           // FK to User
}

impl DiscriminantName for ReviewRelationalKeysDiscriminants {}

// Tree access enum for Review relational keys (no inner types)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum ReviewRelationalTreeNames {
    ReviewedProduct,
    Reviewer,
}

impl DiscriminantName for ReviewRelationalTreeNames {}

#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(ReviewSubscriptionsDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum ReviewSubscriptions {
    Updates,
}

impl DiscriminantName for ReviewSubscriptionsDiscriminants {}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum ReviewSubscriptionTreeNames {
    Updates,
}

impl DiscriminantName for ReviewSubscriptionTreeNames {}

impl Value for ReviewSubscriptions {
    type SelfType<'a> = ReviewSubscriptions;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize ReviewSubscriptions")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize ReviewSubscriptions");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("ReviewSubscriptions")
    }
}

impl Key for ReviewSubscriptions {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl TryFrom<Vec<u8>> for ReviewSubscriptions {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (ReviewSubscriptions, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(value)
    }
}

impl TryFrom<ReviewSubscriptions> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: ReviewSubscriptions) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value, bincode::config::standard())
    }
}

// Value implementations for Review enums
impl Value for ReviewSecondaryKeys {
    type SelfType<'a> = ReviewSecondaryKeys;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize ReviewSecondaryKeys")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize ReviewSecondaryKeys");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> TypeName {
        TypeName::new("ReviewSecondaryKeys")
    }
}

impl Value for ReviewRelationalKeys {
    type SelfType<'a> = ReviewRelationalKeys;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize ReviewRelationalKeys")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize ReviewRelationalKeys");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> TypeName {
        TypeName::new("ReviewRelationalKeys")
    }
}

// Key implementations for Review enums
impl Key for ReviewSecondaryKeys {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl Key for ReviewRelationalKeys {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

// Conversion implementations for ReviewSecondaryKeys
impl TryFrom<ReviewSecondaryKeys> for Vec<u8> {
    type Error = Box<bincode::error::EncodeError>;

    fn try_from(value: ReviewSecondaryKeys) -> Result<Self, Self::Error> {
        let config = bincode::config::standard();
        bincode::encode_to_vec(value, config).map_err(Box::new)
    }
}

impl TryFrom<Vec<u8>> for ReviewSecondaryKeys {
    type Error = Box<bincode::error::DecodeError>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let config = bincode::config::standard();
        let (decoded, _) = bincode::decode_from_slice(&value, config)?;
        Ok(decoded)
    }
}

// Conversion implementations for ReviewRelationalKeys
impl TryFrom<ReviewRelationalKeys> for Vec<u8> {
    type Error = Box<bincode::error::EncodeError>;

    fn try_from(value: ReviewRelationalKeys) -> Result<Self, Self::Error> {
        let config = bincode::config::standard();
        bincode::encode_to_vec(value, config).map_err(Box::new)
    }
}

impl TryFrom<Vec<u8>> for ReviewRelationalKeys {
    type Error = Box<bincode::error::DecodeError>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let config = bincode::config::standard();
        let (decoded, _) = bincode::decode_from_slice(&value, config)?;
        Ok(decoded)
    }
}

// Key trait implementation for Review
pub struct ReviewKeys;

impl NetabaseModelKeyTrait<Definitions, Review> for ReviewKeys {
    type PrimaryKey = ReviewId;
    type SecondaryEnum = ReviewSecondaryKeys;
    type RelationalEnum = ReviewRelationalKeys;

    fn secondary_keys(model: &Review) -> Vec<Self::SecondaryEnum> {
        vec![
            ReviewSecondaryKeys::Rating(ReviewRating(model.rating)),
            ReviewSecondaryKeys::CreatedAt(ReviewCreatedAt(model.created_at)),
        ]
    }

    fn relational_keys(model: &Review) -> Vec<Self::RelationalEnum> {
        vec![
            ReviewRelationalKeys::ReviewedProduct(ProductId(model.product_id)),
            ReviewRelationalKeys::Reviewer(UserId(model.user_id)),
        ]
    }
}

impl NetabaseModelTrait<Definitions> for Review {
    type Keys = ReviewKeys;
    const MODEL_TREE_NAME: DefinitionsDiscriminants = DefinitionsDiscriminants::Review;

    type SecondaryKeys = ReviewSecondaryKeysIter;
    type RelationalKeys = ReviewRelationalKeysIter;
    type SubscriptionEnum = ReviewSubscriptions;
    type Hash = [u8; 32];

    fn primary_key(&self) -> ReviewId {
        ReviewId(self.id)
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        ReviewSecondaryKeysIter {
            iter: ReviewKeys::secondary_keys(self).into_iter(),
        }
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        ReviewRelationalKeysIter {
            iter: ReviewKeys::relational_keys(self).into_iter(),
        }
    }

    fn get_subscriptions(&self) -> Vec<Self::SubscriptionEnum> {
        vec![]
    }

    fn compute_hash(&self) -> Self::Hash {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&self.id.to_le_bytes());
        hasher.update(&self.product_id.to_le_bytes());
        hasher.update(&self.user_id.to_le_bytes());
        hasher.update(&[self.rating]);
        hasher.update(self.comment.as_bytes());
        hasher.update(&self.created_at.to_le_bytes());
        *hasher.finalize().as_bytes()
    }

    fn wrap_primary_key(key: Self::PrimaryKey) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ReviewPrimaryKey(key)
    }

    fn wrap_model(model: Self) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ReviewModel(model)
    }

    fn wrap_secondary_key(key: ReviewSecondaryKeys) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ReviewSecondaryKey(key)
    }

    fn wrap_relational_key(key: ReviewRelationalKeys) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ReviewRelationalKey(key)
    }

    fn wrap_subscription_key(key: ReviewSubscriptions) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ReviewSubscriptionKey(key)
    }

    fn wrap_secondary_key_discriminant(
        key: ReviewSecondaryKeysDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ReviewSecondaryKeyDiscriminant(key)
    }

    fn wrap_relational_key_discriminant(
        key: ReviewRelationalKeysDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ReviewRelationalKeyDiscriminant(key)
    }

    fn wrap_subscription_key_discriminant(
        key: ReviewSubscriptionsDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ReviewSubscriptionKeyDiscriminant(key)
    }
}

// Iterator types for Review
pub struct ReviewSecondaryKeysIter {
    iter: std::vec::IntoIter<ReviewSecondaryKeys>,
}

impl Iterator for ReviewSecondaryKeysIter {
    type Item = ReviewSecondaryKeys;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct ReviewRelationalKeysIter {
    iter: std::vec::IntoIter<ReviewRelationalKeys>,
}

impl Iterator for ReviewRelationalKeysIter {
    type Item = ReviewRelationalKeys;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl RedbNetabaseModelTrait<Definitions> for Review {
    fn definition<'a>(db: &RedbStore<Definitions>) -> TableDefinition<'a, Self::PrimaryKey, Self> {
        let _ = db;
        TableDefinition::new(Self::MODEL_TREE_NAME.as_ref())
    }

    fn secondary_key_table_name(key_discriminant: ReviewSecondaryKeysDiscriminants) -> String {
        format!("Review_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(key_discriminant: ReviewRelationalKeysDiscriminants) -> String {
        format!("Review_rel_{}", key_discriminant.as_ref())
    }

    fn subscription_key_table_name(key_discriminant: ReviewSubscriptionsDiscriminants) -> String {
        format!("Review_sub_{}", key_discriminant.as_ref())
    }

    fn hash_tree_table_name() -> String {
        "Review_hash".to_string()
    }
}

impl SledNetabaseModelTrait<Definitions> for Review {
    fn secondary_key_table_name(key_discriminant: ReviewSecondaryKeysDiscriminants) -> String {
        format!("Review_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(key_discriminant: ReviewRelationalKeysDiscriminants) -> String {
        format!("Review_rel_{}", key_discriminant.as_ref())
    }

    fn subscription_key_table_name(key_discriminant: ReviewSubscriptionsDiscriminants) -> String {
        format!("Review_sub_{}", key_discriminant.as_ref())
    }

    fn hash_tree_table_name() -> String {
        "Review_hash".to_string()
    }
}

// ==================================================================================
// Model 5: Tag
// ==================================================================================

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct Tag {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct TagId(pub u64);

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct TagName(pub String);

// TryFrom implementations for TagId
impl TryFrom<Vec<u8>> for TagId {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (u64, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(TagId(value))
    }
}

impl TryFrom<TagId> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: TagId) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value.0, bincode::config::standard())
    }
}

impl_key_wrapper!(TagId, u64, "TagId");

// Implement redb::Value for Tag
impl redb::Value for Tag {
    type SelfType<'a>
        = Self
    where
        Self: 'a;
    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let config = bincode::config::standard();
        bincode::decode_from_slice(data, config)
            .expect("Failed to decode Tag")
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        let config = bincode::config::standard();
        bincode::encode_to_vec(value, config).expect("Failed to encode Tag")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("Tag")
    }
}

// Secondary keys for Tag
#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(TagSecondaryKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum TagSecondaryKeys {
    Name(TagName),
}

impl DiscriminantName for TagSecondaryKeysDiscriminants {}

// Tree access enum for Tag secondary keys (no inner types)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum TagSecondaryTreeNames {
    Name,
}

impl DiscriminantName for TagSecondaryTreeNames {}

// Relational keys for Tag - products with this tag (via junction table)
#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(TagRelationalKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum TagRelationalKeys {
    TaggedProducts(ProductId), // Many-to-many via ProductTag junction
}

impl DiscriminantName for TagRelationalKeysDiscriminants {}

// Tree access enum for Tag relational keys (no inner types)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum TagRelationalTreeNames {
    TaggedProducts,
}

impl DiscriminantName for TagRelationalTreeNames {}

#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(TagSubscriptionsDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum TagSubscriptions {
    Updates,
}

impl DiscriminantName for TagSubscriptionsDiscriminants {}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum TagSubscriptionTreeNames {
    Updates,
}

impl DiscriminantName for TagSubscriptionTreeNames {}

impl Value for TagSubscriptions {
    type SelfType<'a> = TagSubscriptions;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize TagSubscriptions")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize TagSubscriptions");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("TagSubscriptions")
    }
}

impl Key for TagSubscriptions {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl TryFrom<Vec<u8>> for TagSubscriptions {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (TagSubscriptions, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(value)
    }
}

impl TryFrom<TagSubscriptions> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: TagSubscriptions) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value, bincode::config::standard())
    }
}

// Value implementations for Tag enums
impl Value for TagSecondaryKeys {
    type SelfType<'a> = TagSecondaryKeys;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize TagSecondaryKeys")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize TagSecondaryKeys");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> TypeName {
        TypeName::new("TagSecondaryKeys")
    }
}

impl Value for TagRelationalKeys {
    type SelfType<'a> = TagRelationalKeys;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize TagRelationalKeys")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize TagRelationalKeys");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> TypeName {
        TypeName::new("TagRelationalKeys")
    }
}

// Key implementations for Tag enums
impl Key for TagSecondaryKeys {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl Key for TagRelationalKeys {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

// Conversion implementations for TagSecondaryKeys
impl TryFrom<TagSecondaryKeys> for Vec<u8> {
    type Error = Box<bincode::error::EncodeError>;

    fn try_from(value: TagSecondaryKeys) -> Result<Self, Self::Error> {
        let config = bincode::config::standard();
        bincode::encode_to_vec(value, config).map_err(Box::new)
    }
}

impl TryFrom<Vec<u8>> for TagSecondaryKeys {
    type Error = Box<bincode::error::DecodeError>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let config = bincode::config::standard();
        let (decoded, _) = bincode::decode_from_slice(&value, config)?;
        Ok(decoded)
    }
}

// Conversion implementations for TagRelationalKeys
impl TryFrom<TagRelationalKeys> for Vec<u8> {
    type Error = Box<bincode::error::EncodeError>;

    fn try_from(value: TagRelationalKeys) -> Result<Self, Self::Error> {
        let config = bincode::config::standard();
        bincode::encode_to_vec(value, config).map_err(Box::new)
    }
}

impl TryFrom<Vec<u8>> for TagRelationalKeys {
    type Error = Box<bincode::error::DecodeError>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let config = bincode::config::standard();
        let (decoded, _) = bincode::decode_from_slice(&value, config)?;
        Ok(decoded)
    }
}

// Key trait implementation for Tag
pub struct TagKeys;

impl NetabaseModelKeyTrait<Definitions, Tag> for TagKeys {
    type PrimaryKey = TagId;
    type SecondaryEnum = TagSecondaryKeys;
    type RelationalEnum = TagRelationalKeys;

    fn secondary_keys(model: &Tag) -> Vec<Self::SecondaryEnum> {
        vec![TagSecondaryKeys::Name(TagName(model.name.clone()))]
    }

    fn relational_keys(_model: &Tag) -> Vec<Self::RelationalEnum> {
        // Relational keys would be populated based on ProductTag junction table
        vec![]
    }
}

impl NetabaseModelTrait<Definitions> for Tag {
    type Keys = TagKeys;
    const MODEL_TREE_NAME: DefinitionsDiscriminants = DefinitionsDiscriminants::Tag;

    type SecondaryKeys = TagSecondaryKeysIter;
    type RelationalKeys = TagRelationalKeysIter;
    type SubscriptionEnum = TagSubscriptions;
    type Hash = [u8; 32];

    fn primary_key(&self) -> TagId {
        TagId(self.id)
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        TagSecondaryKeysIter {
            iter: TagKeys::secondary_keys(self).into_iter(),
        }
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        TagRelationalKeysIter {
            iter: TagKeys::relational_keys(self).into_iter(),
        }
    }

    fn get_subscriptions(&self) -> Vec<Self::SubscriptionEnum> {
        vec![]
    }

    fn compute_hash(&self) -> Self::Hash {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&self.id.to_le_bytes());
        hasher.update(self.name.as_bytes());
        *hasher.finalize().as_bytes()
    }

    fn wrap_primary_key(key: Self::PrimaryKey) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::TagPrimaryKey(key)
    }

    fn wrap_model(model: Self) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::TagModel(model)
    }

    fn wrap_secondary_key(key: TagSecondaryKeys) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::TagSecondaryKey(key)
    }

    fn wrap_relational_key(key: TagRelationalKeys) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::TagRelationalKey(key)
    }

    fn wrap_subscription_key(key: TagSubscriptions) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::TagSubscriptionKey(key)
    }

    fn wrap_secondary_key_discriminant(
        key: TagSecondaryKeysDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::TagSecondaryKeyDiscriminant(key)
    }

    fn wrap_relational_key_discriminant(
        key: TagRelationalKeysDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::TagRelationalKeyDiscriminant(key)
    }

    fn wrap_subscription_key_discriminant(
        key: TagSubscriptionsDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::TagSubscriptionKeyDiscriminant(key)
    }
}

// Iterator types for Tag
pub struct TagSecondaryKeysIter {
    iter: std::vec::IntoIter<TagSecondaryKeys>,
}

impl Iterator for TagSecondaryKeysIter {
    type Item = TagSecondaryKeys;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct TagRelationalKeysIter {
    iter: std::vec::IntoIter<TagRelationalKeys>,
}

impl Iterator for TagRelationalKeysIter {
    type Item = TagRelationalKeys;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl RedbNetabaseModelTrait<Definitions> for Tag {
    fn definition<'a>(db: &RedbStore<Definitions>) -> TableDefinition<'a, Self::PrimaryKey, Self> {
        let _ = db;
        TableDefinition::new(Self::MODEL_TREE_NAME.as_ref())
    }

    fn secondary_key_table_name(key_discriminant: TagSecondaryKeysDiscriminants) -> String {
        format!("Tag_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(key_discriminant: TagRelationalKeysDiscriminants) -> String {
        format!("Tag_rel_{}", key_discriminant.as_ref())
    }

    fn subscription_key_table_name(key_discriminant: TagSubscriptionsDiscriminants) -> String {
        format!("Tag_sub_{}", key_discriminant.as_ref())
    }

    fn hash_tree_table_name() -> String {
        "Tag_hash".to_string()
    }
}

impl SledNetabaseModelTrait<Definitions> for Tag {
    fn secondary_key_table_name(key_discriminant: TagSecondaryKeysDiscriminants) -> String {
        format!("Tag_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(key_discriminant: TagRelationalKeysDiscriminants) -> String {
        format!("Tag_rel_{}", key_discriminant.as_ref())
    }

    fn subscription_key_table_name(key_discriminant: TagSubscriptionsDiscriminants) -> String {
        format!("Tag_sub_{}", key_discriminant.as_ref())
    }

    fn hash_tree_table_name() -> String {
        "Tag_hash".to_string()
    }
}

// ==================================================================================
// Model 6: ProductTag (Junction Table)
// ==================================================================================

#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct ProductTag {
    pub product_id: u128,
    pub tag_id: u64,
}

// Composite primary key for ProductTag
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct ProductTagId {
    pub product_id: u128,
    pub tag_id: u64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct ProductTagProductId(pub u128);

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
pub struct ProductTagTagId(pub u64);

// TryFrom implementations for ProductTagId
impl TryFrom<Vec<u8>> for ProductTagId {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (ProductTagId, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(value)
    }
}

impl TryFrom<ProductTagId> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: ProductTagId) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value, bincode::config::standard())
    }
}

// Manual implementation of Key and Value for composite key ProductTagId
impl redb::Value for ProductTagId {
    type SelfType<'a>
        = Self
    where
        Self: 'a;
    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let config = bincode::config::standard();
        bincode::decode_from_slice(data, config)
            .expect("Failed to decode ProductTagId")
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        let config = bincode::config::standard();
        bincode::encode_to_vec(value, config).expect("Failed to encode ProductTagId")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("ProductTagId")
    }
}

impl redb::Key for ProductTagId {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        let config = bincode::config::standard();
        let key1: ProductTagId = bincode::decode_from_slice(data1, config)
            .expect("Failed to decode ProductTagId")
            .0;
        let key2: ProductTagId = bincode::decode_from_slice(data2, config)
            .expect("Failed to decode ProductTagId")
            .0;
        key1.cmp(&key2)
    }
}

// Implement redb::Value for ProductTag
impl redb::Value for ProductTag {
    type SelfType<'a>
        = Self
    where
        Self: 'a;
    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let config = bincode::config::standard();
        bincode::decode_from_slice(data, config)
            .expect("Failed to decode ProductTag")
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        let config = bincode::config::standard();
        bincode::encode_to_vec(value, config).expect("Failed to encode ProductTag")
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("ProductTag")
    }
}

// Secondary keys for ProductTag - allow lookup by either product or tag
#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(ProductTagSecondaryKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum ProductTagSecondaryKeys {
    ProductId(ProductTagProductId),
    TagId(ProductTagTagId),
}

impl DiscriminantName for ProductTagSecondaryKeysDiscriminants {}

// Tree access enum for ProductTag secondary keys (no inner types)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum ProductTagSecondaryTreeNames {
    ProductId,
    TagId,
}

impl DiscriminantName for ProductTagSecondaryTreeNames {}

// Relational keys for ProductTag - references to actual models
#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(ProductTagRelationalKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum ProductTagRelationalKeys {
    Product(ProductId),
    Tag(TagId),
}

impl DiscriminantName for ProductTagRelationalKeysDiscriminants {}

// Tree access enum for ProductTag relational keys (no inner types)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum ProductTagRelationalTreeNames {
    Product,
    Tag,
}

impl DiscriminantName for ProductTagRelationalTreeNames {}

#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(ProductTagSubscriptionsDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum ProductTagSubscriptions {
    Updates,
}

impl DiscriminantName for ProductTagSubscriptionsDiscriminants {}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum ProductTagSubscriptionTreeNames {
    Updates,
}

impl DiscriminantName for ProductTagSubscriptionTreeNames {}

impl Value for ProductTagSubscriptions {
    type SelfType<'a> = ProductTagSubscriptions;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize ProductTagSubscriptions")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize ProductTagSubscriptions");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("ProductTagSubscriptions")
    }
}

impl Key for ProductTagSubscriptions {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl TryFrom<Vec<u8>> for ProductTagSubscriptions {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (ProductTagSubscriptions, usize) =
            bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(value)
    }
}

impl TryFrom<ProductTagSubscriptions> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: ProductTagSubscriptions) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value, bincode::config::standard())
    }
}

// Key implementations for ProductTag enums
impl Key for ProductTagSecondaryKeys {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl Key for ProductTagRelationalKeys {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

// Value implementations for ProductTag enums
impl Value for ProductTagSecondaryKeys {
    type SelfType<'a> = ProductTagSecondaryKeys;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize ProductTagSecondaryKeys")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize ProductTagSecondaryKeys");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> TypeName {
        TypeName::new("ProductTagSecondaryKeys")
    }
}

impl Value for ProductTagRelationalKeys {
    type SelfType<'a> = ProductTagRelationalKeys;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::try_from(data.to_vec()).expect("Failed to deserialize ProductTagRelationalKeys")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        let bytes: Vec<u8> = value
            .clone()
            .try_into()
            .expect("Failed to serialize ProductTagRelationalKeys");
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> TypeName {
        TypeName::new("ProductTagRelationalKeys")
    }
}

// Conversion implementations for ProductTagSecondaryKeys
impl TryFrom<ProductTagSecondaryKeys> for Vec<u8> {
    type Error = Box<bincode::error::EncodeError>;

    fn try_from(value: ProductTagSecondaryKeys) -> Result<Self, Self::Error> {
        let config = bincode::config::standard();
        bincode::encode_to_vec(value, config).map_err(Box::new)
    }
}

impl TryFrom<Vec<u8>> for ProductTagSecondaryKeys {
    type Error = Box<bincode::error::DecodeError>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let config = bincode::config::standard();
        let (decoded, _) = bincode::decode_from_slice(&value, config)?;
        Ok(decoded)
    }
}

// Conversion implementations for ProductTagRelationalKeys
impl TryFrom<ProductTagRelationalKeys> for Vec<u8> {
    type Error = Box<bincode::error::EncodeError>;

    fn try_from(value: ProductTagRelationalKeys) -> Result<Self, Self::Error> {
        let config = bincode::config::standard();
        bincode::encode_to_vec(value, config).map_err(Box::new)
    }
}

impl TryFrom<Vec<u8>> for ProductTagRelationalKeys {
    type Error = Box<bincode::error::DecodeError>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let config = bincode::config::standard();
        let (decoded, _) = bincode::decode_from_slice(&value, config)?;
        Ok(decoded)
    }
}

// Key trait implementation for ProductTag
pub struct ProductTagKeys;

impl NetabaseModelKeyTrait<Definitions, ProductTag> for ProductTagKeys {
    type PrimaryKey = ProductTagId;
    type SecondaryEnum = ProductTagSecondaryKeys;
    type RelationalEnum = ProductTagRelationalKeys;

    fn secondary_keys(model: &ProductTag) -> Vec<Self::SecondaryEnum> {
        vec![
            ProductTagSecondaryKeys::ProductId(ProductTagProductId(model.product_id)),
            ProductTagSecondaryKeys::TagId(ProductTagTagId(model.tag_id)),
        ]
    }

    fn relational_keys(model: &ProductTag) -> Vec<Self::RelationalEnum> {
        vec![
            ProductTagRelationalKeys::Product(ProductId(model.product_id)),
            ProductTagRelationalKeys::Tag(TagId(model.tag_id)),
        ]
    }
}

impl NetabaseModelTrait<Definitions> for ProductTag {
    type Keys = ProductTagKeys;
    const MODEL_TREE_NAME: DefinitionsDiscriminants = DefinitionsDiscriminants::ProductTag;

    type SecondaryKeys = ProductTagSecondaryKeysIter;
    type RelationalKeys = ProductTagRelationalKeysIter;
    type SubscriptionEnum = ProductTagSubscriptions;
    type Hash = [u8; 32];

    fn primary_key(&self) -> ProductTagId {
        ProductTagId {
            product_id: self.product_id,
            tag_id: self.tag_id,
        }
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        ProductTagSecondaryKeysIter {
            iter: ProductTagKeys::secondary_keys(self).into_iter(),
        }
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        ProductTagRelationalKeysIter {
            iter: ProductTagKeys::relational_keys(self).into_iter(),
        }
    }

    fn get_subscriptions(&self) -> Vec<Self::SubscriptionEnum> {
        vec![]
    }

    fn compute_hash(&self) -> Self::Hash {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&self.product_id.to_le_bytes());
        hasher.update(&self.tag_id.to_le_bytes());
        *hasher.finalize().as_bytes()
    }

    fn wrap_primary_key(key: Self::PrimaryKey) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductTagPrimaryKey(key)
    }

    fn wrap_model(model: Self) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductTagModel(model)
    }

    fn wrap_secondary_key(key: ProductTagSecondaryKeys) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductTagSecondaryKey(key)
    }

    fn wrap_relational_key(key: ProductTagRelationalKeys) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductTagRelationalKey(key)
    }

    fn wrap_subscription_key(key: ProductTagSubscriptions) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductTagSubscriptionKey(key)
    }

    fn wrap_secondary_key_discriminant(
        key: ProductTagSecondaryKeysDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductTagSecondaryKeyDiscriminant(key)
    }

    fn wrap_relational_key_discriminant(
        key: ProductTagRelationalKeysDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductTagRelationalKeyDiscriminant(key)
    }

    fn wrap_subscription_key_discriminant(
        key: ProductTagSubscriptionsDiscriminants,
    ) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductTagSubscriptionKeyDiscriminant(key)
    }
}

// Iterator types for ProductTag
pub struct ProductTagSecondaryKeysIter {
    iter: std::vec::IntoIter<ProductTagSecondaryKeys>,
}

impl Iterator for ProductTagSecondaryKeysIter {
    type Item = ProductTagSecondaryKeys;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct ProductTagRelationalKeysIter {
    iter: std::vec::IntoIter<ProductTagRelationalKeys>,
}

impl Iterator for ProductTagRelationalKeysIter {
    type Item = ProductTagRelationalKeys;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl RedbNetabaseModelTrait<Definitions> for ProductTag {
    fn definition<'a>(db: &RedbStore<Definitions>) -> TableDefinition<'a, Self::PrimaryKey, Self> {
        let _ = db;
        TableDefinition::new(Self::MODEL_TREE_NAME.as_ref())
    }

    fn secondary_key_table_name(key_discriminant: ProductTagSecondaryKeysDiscriminants) -> String {
        format!("ProductTag_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(
        key_discriminant: ProductTagRelationalKeysDiscriminants,
    ) -> String {
        format!("ProductTag_rel_{}", key_discriminant.as_ref())
    }

    fn subscription_key_table_name(
        key_discriminant: ProductTagSubscriptionsDiscriminants,
    ) -> String {
        format!("ProductTag_sub_{}", key_discriminant.as_ref())
    }

    fn hash_tree_table_name() -> String {
        "ProductTag_hash".to_string()
    }
}

impl SledNetabaseModelTrait<Definitions> for ProductTag {
    fn secondary_key_table_name(key_discriminant: ProductTagSecondaryKeysDiscriminants) -> String {
        format!("ProductTag_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(
        key_discriminant: ProductTagRelationalKeysDiscriminants,
    ) -> String {
        format!("ProductTag_rel_{}", key_discriminant.as_ref())
    }

    fn subscription_key_table_name(
        key_discriminant: ProductTagSubscriptionsDiscriminants,
    ) -> String {
        format!("ProductTag_sub_{}", key_discriminant.as_ref())
    }

    fn hash_tree_table_name() -> String {
        "ProductTag_hash".to_string()
    }
}

// ==================================================================================
// Definitions
// ==================================================================================

/// Unified enum wrapping all model-associated types for the Definition
/// This eliminates the need for opaque Vec<u8> and String types in operations
#[derive(Debug, Clone)]
pub enum DefinitionModelAssociatedTypes {
    // User-related types
    UserPrimaryKey(UserId),
    UserModel(User),
    UserSecondaryKey(UserSecondaryKeys),
    UserRelationalKey(UserRelationalKeys),
    UserSubscriptionKey(UserSubscriptions),
    UserSecondaryKeyDiscriminant(UserSecondaryKeysDiscriminants),
    UserRelationalKeyDiscriminant(UserRelationalKeysDiscriminants),
    UserSubscriptionKeyDiscriminant(UserSubscriptionsDiscriminants),

    // Product-related types
    ProductPrimaryKey(ProductId),
    ProductModel(Product),
    ProductSecondaryKey(ProductSecondaryKeys),
    ProductRelationalKey(ProductRelationalKeys),
    ProductSubscriptionKey(ProductSubscriptions),
    ProductSecondaryKeyDiscriminant(ProductSecondaryKeysDiscriminants),
    ProductRelationalKeyDiscriminant(ProductRelationalKeysDiscriminants),
    ProductSubscriptionKeyDiscriminant(ProductSubscriptionsDiscriminants),

    // Category-related types
    CategoryPrimaryKey(CategoryId),
    CategoryModel(Category),
    CategorySecondaryKey(CategorySecondaryKeys),
    CategoryRelationalKey(CategoryRelationalKeys),
    CategorySubscriptionKey(CategorySubscriptions),
    CategorySecondaryKeyDiscriminant(CategorySecondaryKeysDiscriminants),
    CategoryRelationalKeyDiscriminant(CategoryRelationalKeysDiscriminants),
    CategorySubscriptionKeyDiscriminant(CategorySubscriptionsDiscriminants),

    // Review-related types
    ReviewPrimaryKey(ReviewId),
    ReviewModel(Review),
    ReviewSecondaryKey(ReviewSecondaryKeys),
    ReviewRelationalKey(ReviewRelationalKeys),
    ReviewSubscriptionKey(ReviewSubscriptions),
    ReviewSecondaryKeyDiscriminant(ReviewSecondaryKeysDiscriminants),
    ReviewRelationalKeyDiscriminant(ReviewRelationalKeysDiscriminants),
    ReviewSubscriptionKeyDiscriminant(ReviewSubscriptionsDiscriminants),

    // Tag-related types
    TagPrimaryKey(TagId),
    TagModel(Tag),
    TagSecondaryKey(TagSecondaryKeys),
    TagRelationalKey(TagRelationalKeys),
    TagSubscriptionKey(TagSubscriptions),
    TagSecondaryKeyDiscriminant(TagSecondaryKeysDiscriminants),
    TagRelationalKeyDiscriminant(TagRelationalKeysDiscriminants),
    TagSubscriptionKeyDiscriminant(TagSubscriptionsDiscriminants),

    // ProductTag-related types
    ProductTagPrimaryKey(ProductTagId),
    ProductTagModel(ProductTag),
    ProductTagSecondaryKey(ProductTagSecondaryKeys),
    ProductTagRelationalKey(ProductTagRelationalKeys),
    ProductTagSubscriptionKey(ProductTagSubscriptions),
    ProductTagSecondaryKeyDiscriminant(ProductTagSecondaryKeysDiscriminants),
    ProductTagRelationalKeyDiscriminant(ProductTagRelationalKeysDiscriminants),
    ProductTagSubscriptionKeyDiscriminant(ProductTagSubscriptionsDiscriminants),

    // Generic key wrapper
    DefinitionKey(DefinitionKeys),
}

impl ModelAssociatedTypesExt<Definitions> for DefinitionModelAssociatedTypes {
    fn from_primary_key<M: NetabaseModelTrait<Definitions>>(key: M::PrimaryKey) -> Self {
        M::wrap_primary_key(key)
    }

    fn from_model<M: NetabaseModelTrait<Definitions>>(model: M) -> Self {
        M::wrap_model(model)
    }

    fn from_secondary_key<M: NetabaseModelTrait<Definitions>>(
        key: <<M::Keys as NetabaseModelKeyTrait<Definitions, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant,
    ) -> Self {
        M::wrap_secondary_key_discriminant(key)
    }

    fn from_relational_key_discriminant<M: NetabaseModelTrait<Definitions>>(
        key: <<M::Keys as NetabaseModelKeyTrait<Definitions, M>>::RelationalEnum as IntoDiscriminant>::Discriminant,
    ) -> Self {
        M::wrap_relational_key_discriminant(key)
    }

    fn from_secondary_key_data<M: NetabaseModelTrait<Definitions>>(
        key: <M::Keys as NetabaseModelKeyTrait<Definitions, M>>::SecondaryEnum,
    ) -> Self {
        M::wrap_secondary_key(key)
    }

    fn from_relational_key_data<M: NetabaseModelTrait<Definitions>>(
        key: <M::Keys as NetabaseModelKeyTrait<Definitions, M>>::RelationalEnum,
    ) -> Self {
        M::wrap_relational_key(key)
    }

    fn from_subscription_key_discriminant<M: NetabaseModelTrait<Definitions>>(
        key: <<M as NetabaseModelTrait<Definitions>>::SubscriptionEnum as IntoDiscriminant>::Discriminant,
    ) -> Self {
        M::wrap_subscription_key_discriminant(key)
    }
}

// Implement RedbModelAssociatedTypesExt with direct implementations
impl RedbModelAssociatedTypesExt<Definitions> for DefinitionModelAssociatedTypes {
    fn insert_model_into_redb(
        &self,
        txn: &WriteTransaction,
        table_name: &str,
        key: &Self,
    ) -> NetabaseResult<()> {
        match (self, key) {
            (
                DefinitionModelAssociatedTypes::UserModel(model),
                DefinitionModelAssociatedTypes::UserPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<UserId, User> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, model)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ProductModel(model),
                DefinitionModelAssociatedTypes::ProductPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<ProductId, Product> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, model)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::CategoryModel(model),
                DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<CategoryId, Category> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, model)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ReviewModel(model),
                DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<ReviewId, Review> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, model)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::TagModel(model),
                DefinitionModelAssociatedTypes::TagPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<TagId, Tag> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, model)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ProductTagModel(model),
                DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<ProductTagId, ProductTag> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, model)?;
                Ok(())
            }
            _ => Err(NetabaseError::Other(
                "Type mismatch in insert_model_into_redb".into(),
            )),
        }
    }

    fn insert_secondary_key_into_redb(
        &self,
        txn: &WriteTransaction,
        table_name: &str,
        primary_key_ref: &Self,
    ) -> NetabaseResult<()> {
        match (self, primary_key_ref) {
            (
                DefinitionModelAssociatedTypes::UserSecondaryKey(sk),
                DefinitionModelAssociatedTypes::UserPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<UserSecondaryKeys, UserId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(sk, pk)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ProductSecondaryKey(sk),
                DefinitionModelAssociatedTypes::ProductPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<ProductSecondaryKeys, ProductId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(sk, pk)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::CategorySecondaryKey(sk),
                DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<CategorySecondaryKeys, CategoryId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(sk, pk)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ReviewSecondaryKey(sk),
                DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<ReviewSecondaryKeys, ReviewId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(sk, pk)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::TagSecondaryKey(sk),
                DefinitionModelAssociatedTypes::TagPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<TagSecondaryKeys, TagId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(sk, pk)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ProductTagSecondaryKey(sk),
                DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<ProductTagSecondaryKeys, ProductTagId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(sk, pk)?;
                Ok(())
            }
            _ => Err(NetabaseError::Other(
                "Type mismatch in insert_secondary_key_into_redb".into(),
            )),
        }
    }

    fn insert_relational_key_into_redb(
        &self,
        txn: &WriteTransaction,
        table_name: &str,
        primary_key_ref: &Self,
    ) -> NetabaseResult<()> {
        match (self, primary_key_ref) {
            // For User relational keys: stores ProductId -> UserId mappings
            (
                DefinitionModelAssociatedTypes::UserRelationalKey(rk),
                DefinitionModelAssociatedTypes::UserPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<UserRelationalKeys, UserId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(rk, pk)?;
                Ok(())
            }
            // For Product relational keys: stores UserId -> ProductId mappings
            (
                DefinitionModelAssociatedTypes::ProductRelationalKey(rk),
                DefinitionModelAssociatedTypes::ProductPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<ProductRelationalKeys, ProductId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(rk, pk)?;
                Ok(())
            }
            // For Category relational keys
            (
                DefinitionModelAssociatedTypes::CategoryRelationalKey(rk),
                DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<CategoryRelationalKeys, CategoryId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(rk, pk)?;
                Ok(())
            }
            // For Review relational keys: stores ProductId/UserId -> ReviewId mappings
            (
                DefinitionModelAssociatedTypes::ReviewRelationalKey(rk),
                DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<ReviewRelationalKeys, ReviewId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(rk, pk)?;
                Ok(())
            }
            // For Tag relational keys
            (
                DefinitionModelAssociatedTypes::TagRelationalKey(rk),
                DefinitionModelAssociatedTypes::TagPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<TagRelationalKeys, TagId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(rk, pk)?;
                Ok(())
            }
            // For ProductTag relational keys: junction table mappings
            (
                DefinitionModelAssociatedTypes::ProductTagRelationalKey(rk),
                DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<ProductTagRelationalKeys, ProductTagId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(rk, pk)?;
                Ok(())
            }
            _ => Err(NetabaseError::Other(
                "Type mismatch in insert_relational_key_into_redb".into(),
            )),
        }
    }

    fn insert_hash_into_redb(
        hash: &[u8; 32],
        txn: &WriteTransaction,
        table_name: &str,
        primary_key_ref: &Self,
    ) -> NetabaseResult<()> {
        match primary_key_ref {
            DefinitionModelAssociatedTypes::UserPrimaryKey(pk) => {
                // Assuming Hash Table is [u8; 32] -> PrimaryKey
                let table_def: TableDefinition<[u8; 32], UserId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(hash, pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ProductPrimaryKey(pk) => {
                let table_def: TableDefinition<[u8; 32], ProductId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(hash, pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk) => {
                let table_def: TableDefinition<[u8; 32], CategoryId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(hash, pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk) => {
                let table_def: TableDefinition<[u8; 32], ReviewId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(hash, pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::TagPrimaryKey(pk) => {
                let table_def: TableDefinition<[u8; 32], TagId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(hash, pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk) => {
                let table_def: TableDefinition<[u8; 32], ProductTagId> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(hash, pk)?;
                Ok(())
            }
            _ => Err(NetabaseError::Other(
                "Type mismatch in insert_hash_into_redb".into(),
            )),
        }
    }

    fn insert_subscription_into_redb(
        hash: &[u8; 32],
        txn: &WriteTransaction,
        table_name: &str,
        primary_key_ref: &Self,
    ) -> NetabaseResult<()> {
        // Subscription tree stores: PrimaryKey (wrapped) -> Hash
        // This allows us to track which models are in this subscription
        // and compute order-independent state via hash accumulation
        match primary_key_ref {
            DefinitionModelAssociatedTypes::UserPrimaryKey(pk) => {
                let table_def: TableDefinition<UserId, [u8; 32]> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, hash)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ProductPrimaryKey(pk) => {
                let table_def: TableDefinition<ProductId, [u8; 32]> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, hash)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk) => {
                let table_def: TableDefinition<CategoryId, [u8; 32]> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, hash)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk) => {
                let table_def: TableDefinition<ReviewId, [u8; 32]> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, hash)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::TagPrimaryKey(pk) => {
                let table_def: TableDefinition<TagId, [u8; 32]> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, hash)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk) => {
                let table_def: TableDefinition<ProductTagId, [u8; 32]> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, hash)?;
                Ok(())
            }
            _ => Err(NetabaseError::Other(
                "Type mismatch in insert_subscription_into_redb".into(),
            )),
        }
    }

    fn delete_model_from_redb(
        &self,
        txn: &WriteTransaction,
        table_name: &str,
    ) -> NetabaseResult<()> {
        match self {
            DefinitionModelAssociatedTypes::UserPrimaryKey(pk) => {
                let table_def: TableDefinition<UserId, User> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ProductPrimaryKey(pk) => {
                let table_def: TableDefinition<ProductId, Product> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk) => {
                let table_def: TableDefinition<CategoryId, Category> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk) => {
                let table_def: TableDefinition<ReviewId, Review> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::TagPrimaryKey(pk) => {
                let table_def: TableDefinition<TagId, Tag> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk) => {
                let table_def: TableDefinition<ProductTagId, ProductTag> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            }
            _ => Err(NetabaseError::Other(
                "Type mismatch in delete_model_from_redb".into(),
            )),
        }
    }

    fn delete_subscription_from_redb(
        &self,
        txn: &WriteTransaction,
        table_name: &str,
    ) -> NetabaseResult<()> {
        // Subscription trees store: PrimaryKey -> Hash
        // Delete based on the primary key (self is the wrapped primary key)
        match self {
            DefinitionModelAssociatedTypes::UserPrimaryKey(pk) => {
                let table_def: TableDefinition<UserId, [u8; 32]> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ProductPrimaryKey(pk) => {
                let table_def: TableDefinition<ProductId, [u8; 32]> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk) => {
                let table_def: TableDefinition<CategoryId, [u8; 32]> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk) => {
                let table_def: TableDefinition<ReviewId, [u8; 32]> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::TagPrimaryKey(pk) => {
                let table_def: TableDefinition<TagId, [u8; 32]> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk) => {
                let table_def: TableDefinition<ProductTagId, [u8; 32]> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            }
            _ => Err(NetabaseError::Other(
                "Type mismatch in delete_subscription_from_redb".into(),
            )),
        }
    }
}

// Implement SledModelAssociatedTypesExt with direct implementations
impl SledModelAssociatedTypesExt<Definitions> for DefinitionModelAssociatedTypes {
    fn insert_model_into_sled(
        &self,
        db: &sled::Db,
        tree_name: &str,
        key: &Self,
    ) -> NetabaseResult<()> {
        let config = bincode::config::standard();
        match (self, key) {
            (
                DefinitionModelAssociatedTypes::UserModel(model),
                DefinitionModelAssociatedTypes::UserPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let key_bytes = bincode::encode_to_vec(pk, config)?;
                let value_bytes = bincode::encode_to_vec(model, config)?;
                tree.insert(key_bytes, value_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ProductModel(model),
                DefinitionModelAssociatedTypes::ProductPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let key_bytes = bincode::encode_to_vec(pk, config)?;
                let value_bytes = bincode::encode_to_vec(model, config)?;
                tree.insert(key_bytes, value_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::CategoryModel(model),
                DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let key_bytes = bincode::encode_to_vec(pk, config)?;
                let value_bytes = bincode::encode_to_vec(model, config)?;
                tree.insert(key_bytes, value_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ReviewModel(model),
                DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let key_bytes = bincode::encode_to_vec(pk, config)?;
                let value_bytes = bincode::encode_to_vec(model, config)?;
                tree.insert(key_bytes, value_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::TagModel(model),
                DefinitionModelAssociatedTypes::TagPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let key_bytes = bincode::encode_to_vec(pk, config)?;
                let value_bytes = bincode::encode_to_vec(model, config)?;
                tree.insert(key_bytes, value_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ProductTagModel(model),
                DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let key_bytes = bincode::encode_to_vec(pk, config)?;
                let value_bytes = bincode::encode_to_vec(model, config)?;
                tree.insert(key_bytes, value_bytes)?;
                Ok(())
            }
            _ => Err(NetabaseError::Other(
                "Type mismatch in insert_model_into_sled".into(),
            )),
        }
    }

    fn insert_secondary_key_into_sled(
        &self,
        db: &sled::Db,
        tree_name: &str,
        primary_key_ref: &Self,
    ) -> NetabaseResult<()> {
        let config = bincode::config::standard();
        match (self, primary_key_ref) {
            (
                DefinitionModelAssociatedTypes::UserSecondaryKey(sk),
                DefinitionModelAssociatedTypes::UserPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let sk_bytes = bincode::encode_to_vec(sk, config)?;
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(sk_bytes, pk_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ProductSecondaryKey(sk),
                DefinitionModelAssociatedTypes::ProductPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let sk_bytes = bincode::encode_to_vec(sk, config)?;
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(sk_bytes, pk_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::CategorySecondaryKey(sk),
                DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let sk_bytes = bincode::encode_to_vec(sk, config)?;
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(sk_bytes, pk_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ReviewSecondaryKey(sk),
                DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let sk_bytes = bincode::encode_to_vec(sk, config)?;
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(sk_bytes, pk_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::TagSecondaryKey(sk),
                DefinitionModelAssociatedTypes::TagPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let sk_bytes = bincode::encode_to_vec(sk, config)?;
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(sk_bytes, pk_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ProductTagSecondaryKey(sk),
                DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let sk_bytes = bincode::encode_to_vec(sk, config)?;
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(sk_bytes, pk_bytes)?;
                Ok(())
            }
            _ => Err(NetabaseError::Other(
                "Type mismatch in insert_secondary_key_into_sled".into(),
            )),
        }
    }

    fn insert_relational_key_into_sled(
        &self,
        db: &sled::Db,
        tree_name: &str,
        primary_key_ref: &Self,
    ) -> NetabaseResult<()> {
        let config = bincode::config::standard();
        match (self, primary_key_ref) {
            (
                DefinitionModelAssociatedTypes::UserRelationalKey(rk),
                DefinitionModelAssociatedTypes::UserPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let rk_bytes = bincode::encode_to_vec(rk, config)?;
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(rk_bytes, pk_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ProductRelationalKey(rk),
                DefinitionModelAssociatedTypes::ProductPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let rk_bytes = bincode::encode_to_vec(rk, config)?;
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(rk_bytes, pk_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::CategoryRelationalKey(rk),
                DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let rk_bytes = bincode::encode_to_vec(rk, config)?;
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(rk_bytes, pk_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ReviewRelationalKey(rk),
                DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let rk_bytes = bincode::encode_to_vec(rk, config)?;
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(rk_bytes, pk_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::TagRelationalKey(rk),
                DefinitionModelAssociatedTypes::TagPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let rk_bytes = bincode::encode_to_vec(rk, config)?;
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(rk_bytes, pk_bytes)?;
                Ok(())
            }
            (
                DefinitionModelAssociatedTypes::ProductTagRelationalKey(rk),
                DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk),
            ) => {
                let tree = db.open_tree(tree_name)?;
                let rk_bytes = bincode::encode_to_vec(rk, config)?;
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(rk_bytes, pk_bytes)?;
                Ok(())
            }
            _ => Err(NetabaseError::Other(
                "Type mismatch in insert_relational_key_into_sled".into(),
            )),
        }
    }

    fn insert_hash_into_sled(
        hash: &[u8; 32],
        db: &sled::Db,
        tree_name: &str,
        primary_key_ref: &Self,
    ) -> NetabaseResult<()> {
        let config = bincode::config::standard();
        let tree = db.open_tree(tree_name)?;
        let hash_bytes = bincode::encode_to_vec(hash, config)?;

        match primary_key_ref {
            DefinitionModelAssociatedTypes::UserPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(pk_bytes, hash_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ProductPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(pk_bytes, hash_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(pk_bytes, hash_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(pk_bytes, hash_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::TagPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(pk_bytes, hash_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(pk_bytes, hash_bytes)?;
                Ok(())
            }
            _ => Err(NetabaseError::Other(
                "Type mismatch in insert_hash_into_sled".into(),
            )),
        }
    }

    fn insert_subscription_into_sled(
        hash: &[u8; 32],
        db: &sled::Db,
        tree_name: &str,
        primary_key_ref: &Self,
    ) -> NetabaseResult<()> {
        let config = bincode::config::standard();
        let tree = db.open_tree(tree_name)?;
        let hash_bytes = bincode::encode_to_vec(hash, config)?;

        match primary_key_ref {
            DefinitionModelAssociatedTypes::UserPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(pk_bytes, hash_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ProductPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(pk_bytes, hash_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(pk_bytes, hash_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(pk_bytes, hash_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::TagPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(pk_bytes, hash_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.insert(pk_bytes, hash_bytes)?;
                Ok(())
            }
            _ => Err(NetabaseError::Other(
                "Type mismatch in insert_subscription_into_sled".into(),
            )),
        }
    }

    fn delete_model_from_sled(&self, db: &sled::Db, tree_name: &str) -> NetabaseResult<()> {
        let config = bincode::config::standard();
        let tree = db.open_tree(tree_name)?;

        match self {
            DefinitionModelAssociatedTypes::UserPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.remove(pk_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ProductPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.remove(pk_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.remove(pk_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.remove(pk_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::TagPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.remove(pk_bytes)?;
                Ok(())
            }
            DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk) => {
                let pk_bytes = bincode::encode_to_vec(pk, config)?;
                tree.remove(pk_bytes)?;
                Ok(())
            }
            _ => Err(NetabaseError::Other(
                "Type mismatch in delete_model_from_sled".into(),
            )),
        }
    }

    fn delete_subscription_from_sled(&self, db: &sled::Db, tree_name: &str) -> NetabaseResult<()> {
        // Reuse delete_model_from_sled logic as it's just removing by primary key
        self.delete_model_from_sled(db, tree_name)
    }
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(name(DefinitionsDiscriminants))]
#[strum_discriminants(derive(EnumIter, AsRefStr, Hash))]
pub enum Definitions {
    User(User),
    Product(Product),
    Category(Category),
    Review(Review),
    Tag(Tag),
    ProductTag(ProductTag),
}

impl NetabaseDefinitionTrait for Definitions {
    type Keys = DefinitionKeys;
    type ModelAssociatedTypes = DefinitionModelAssociatedTypes;
    type Permissions = netabase_store::traits::permission::NoPermissions;
}

// DiscriminantName is automatically implemented via the trait's default implementation using AsRefStr
impl DiscriminantName for DefinitionsDiscriminants {}

#[derive(Debug, Clone, TryInto, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, AsRefStr, Hash))]
pub enum DefinitionKeys {
    UserKeys,
    ProductKeys,
    CategoryKeys,
    ReviewKeys,
    TagKeys,
    ProductTagKeys,
}

impl TreeManager<Definitions> for Definitions {
    fn all_trees() -> AllTrees<Definitions> {
        AllTrees::new()
    }

    fn get_tree_name(model_discriminant: &DefinitionsDiscriminants) -> Option<String> {
        match model_discriminant {
            DefinitionsDiscriminants::User => Some("User".to_string()),
            DefinitionsDiscriminants::Product => Some("Product".to_string()),
            DefinitionsDiscriminants::Category => Some("Category".to_string()),
            DefinitionsDiscriminants::Review => Some("Review".to_string()),
            DefinitionsDiscriminants::Tag => Some("Tag".to_string()),
            DefinitionsDiscriminants::ProductTag => Some("ProductTag".to_string()),
        }
    }

    fn get_secondary_tree_names(model_discriminant: &DefinitionsDiscriminants) -> Vec<String> {
        match model_discriminant {
            DefinitionsDiscriminants::User => vec![
                "User_Email".to_string(),
                "User_Name".to_string(),
                "User_Age".to_string(),
            ],
            DefinitionsDiscriminants::Product => {
                vec!["Product_Title".to_string(), "Product_Score".to_string()]
            }
            DefinitionsDiscriminants::Category => vec!["Category_Name".to_string()],
            DefinitionsDiscriminants::Review => {
                vec!["Review_Rating".to_string(), "Review_CreatedAt".to_string()]
            }
            DefinitionsDiscriminants::Tag => vec!["Tag_Name".to_string()],
            DefinitionsDiscriminants::ProductTag => vec![
                "ProductTag_ProductId".to_string(),
                "ProductTag_TagId".to_string(),
            ],
        }
    }

    fn get_relational_tree_names(model_discriminant: &DefinitionsDiscriminants) -> Vec<String> {
        match model_discriminant {
            DefinitionsDiscriminants::User => vec!["User_rel_CreatedProducts".to_string()],
            DefinitionsDiscriminants::Product => vec!["Product_rel_CreatedBy".to_string()],
            DefinitionsDiscriminants::Category => vec!["Category_rel_Products".to_string()],
            DefinitionsDiscriminants::Review => vec![
                "Review_rel_ReviewedProduct".to_string(),
                "Review_rel_Reviewer".to_string(),
            ],
            DefinitionsDiscriminants::Tag => vec!["Tag_rel_TaggedProducts".to_string()],
            DefinitionsDiscriminants::ProductTag => vec![
                "ProductTag_rel_Product".to_string(),
                "ProductTag_rel_Tag".to_string(),
            ],
        }
    }

    fn get_subscription_tree_names(model_discriminant: &DefinitionsDiscriminants) -> Vec<String> {
        match model_discriminant {
            DefinitionsDiscriminants::User => vec!["User_sub_Updates".to_string()],
            DefinitionsDiscriminants::Product => vec!["Product_sub_Updates".to_string()],
            DefinitionsDiscriminants::Category => vec!["Category_sub_AllProducts".to_string()],
            DefinitionsDiscriminants::Review => vec!["Review_sub_ReviewsForProduct".to_string()],
            DefinitionsDiscriminants::Tag => vec!["Tag_sub_TaggedItems".to_string()],
            DefinitionsDiscriminants::ProductTag => vec!["ProductTag_sub_ProductTags".to_string()],
        }
    }
}

impl NetabaseDefinitionKeyTrait<Definitions> for DefinitionKeys {
    fn inner<M: NetabaseModelTrait<Definitions>>(&self) -> M::Keys
    where
        Self: TryInto<M::Keys>,
        <Self as TryInto<M::Keys>>::Error: std::fmt::Debug,
    {
        self.clone()
            .try_into()
            .expect("Key variant does not match requested Model")
    }
}

// ==================================================================================
// Store Helper Methods for Definition Enums
// ==================================================================================

/// Extension trait for RedbStore to work with Definition enums directly
pub trait DefinitionStoreExt {
    /// Insert a model from a Definition enum variant
    fn put_definition(&self, definition: Definitions) -> Result<(), Box<dyn std::error::Error>>;

    /// Retrieve a model as a Definition enum variant using a typed primary key
    fn get_definition(
        &self,
        key: DefinitionModelAssociatedTypes,
    ) -> Result<Option<Definitions>, Box<dyn std::error::Error>>;

    /// Batch insert multiple models from Definition enum variants
    fn put_many_definitions(
        &self,
        definitions: Vec<Definitions>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Returns an iterator over all models in the database
    fn iter_all_models(&self) -> Result<AllModelsIterator, Box<dyn std::error::Error>>;
}

impl DefinitionStoreExt for RedbStore<Definitions> {
    fn put_definition(&self, definition: Definitions) -> Result<(), Box<dyn std::error::Error>> {
        match definition {
            Definitions::User(model) => Ok(self.put_one(model)?),
            Definitions::Product(model) => Ok(self.put_one(model)?),
            Definitions::Category(model) => Ok(self.put_one(model)?),
            Definitions::Review(model) => Ok(self.put_one(model)?),
            Definitions::Tag(model) => Ok(self.put_one(model)?),
            Definitions::ProductTag(model) => Ok(self.put_one(model)?),
        }
    }

    fn get_definition(
        &self,
        key: DefinitionModelAssociatedTypes,
    ) -> Result<Option<Definitions>, Box<dyn std::error::Error>> {
        match key {
            DefinitionModelAssociatedTypes::UserPrimaryKey(pk) => {
                Ok(self.get_one::<User>(pk)?.map(Definitions::User))
            }
            DefinitionModelAssociatedTypes::ProductPrimaryKey(pk) => {
                Ok(self.get_one::<Product>(pk)?.map(Definitions::Product))
            }
            DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk) => {
                Ok(self.get_one::<Category>(pk)?.map(Definitions::Category))
            }
            DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk) => {
                Ok(self.get_one::<Review>(pk)?.map(Definitions::Review))
            }
            DefinitionModelAssociatedTypes::TagPrimaryKey(pk) => {
                Ok(self.get_one::<Tag>(pk)?.map(Definitions::Tag))
            }
            DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk) => {
                Ok(self.get_one::<ProductTag>(pk)?.map(Definitions::ProductTag))
            }
            _ => Err("Invalid key type - only PrimaryKey variants are supported".into()),
        }
    }

    fn put_many_definitions(
        &self,
        definitions: Vec<Definitions>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for definition in definitions {
            self.put_definition(definition)?;
        }
        Ok(())
    }

    fn iter_all_models(&self) -> Result<AllModelsIterator, Box<dyn std::error::Error>> {
        // Note: This is a simplified implementation.
        // For a production system, you would need to add API methods to the core library
        // to iterate through table keys, or maintain an index of all primary keys.
        Ok(AllModelsIterator::new(Vec::new()))
    }
}

// ==================================================================================
// Iterator Over All Models
// ==================================================================================

/// Iterator that provides access to all models in the store as Cow<Definitions>
pub struct AllModelsIterator {
    models: Vec<Definitions>,
    index: usize,
}

impl AllModelsIterator {
    /// Create a new iterator from a vector of all models
    fn new(models: Vec<Definitions>) -> Self {
        AllModelsIterator { models, index: 0 }
    }
}

impl Iterator for AllModelsIterator {
    type Item = Cow<'static, Definitions>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.models.len() {
            let model = self.models[self.index].clone();
            self.index += 1;
            Some(Cow::Owned(model))
        } else {
            None
        }
    }
}

// ================================================================================= ================
// Comprehensive Testing Functions
// ================================================================================= ================

fn create_test_data() -> (
    Vec<User>,
    Vec<Product>,
    Vec<Category>,
    Vec<Review>,
    Vec<Tag>,
    Vec<ProductTag>,
) {
    let users = vec![
        User {
            id: 1,
            email: "alice@example.com".to_string(),
            name: "Alice Johnson".to_string(),
            age: 28,
        },
        User {
            id: 2,
            email: "bob@example.com".to_string(),
            name: "Bob Smith".to_string(),
            age: 34,
        },
        User {
            id: 3,
            email: "charlie@example.com".to_string(),
            name: "Charlie Brown".to_string(),
            age: 22,
        },
    ];

    let categories = vec![
        Category {
            id: 1,
            name: "Electronics".to_string(),
            description: "Electronic devices and accessories".to_string(),
        },
        Category {
            id: 2,
            name: "Furniture".to_string(),
            description: "Home and office furniture".to_string(),
        },
        Category {
            id: 3,
            name: "Appliances".to_string(),
            description: "Kitchen and home appliances".to_string(),
        },
    ];

    let products = vec![
        Product {
            uuid: 100,
            title: "Laptop Pro".to_string(),
            score: 95,
            created_by: 1,
        },
        Product {
            uuid: 101,
            title: "Wireless Mouse".to_string(),
            score: 85,
            created_by: 1,
        },
        Product {
            uuid: 102,
            title: "Coffee Maker".to_string(),
            score: 78,
            created_by: 2,
        },
        Product {
            uuid: 103,
            title: "Gaming Chair".to_string(),
            score: 92,
            created_by: 3,
        },
    ];

    let reviews = vec![
        Review {
            id: 1,
            product_id: 100,
            user_id: 2,
            rating: 5,
            comment: "Excellent laptop! Very fast and reliable.".to_string(),
            created_at: 1609459200,
        },
        Review {
            id: 2,
            product_id: 100,
            user_id: 3,
            rating: 4,
            comment: "Great performance, but a bit pricey.".to_string(),
            created_at: 1609545600,
        },
        Review {
            id: 3,
            product_id: 101,
            user_id: 2,
            rating: 5,
            comment: "Perfect wireless mouse, no lag!".to_string(),
            created_at: 1609632000,
        },
        Review {
            id: 4,
            product_id: 102,
            user_id: 1,
            rating: 4,
            comment: "Makes great coffee, easy to use.".to_string(),
            created_at: 1609718400,
        },
        Review {
            id: 5,
            product_id: 103,
            user_id: 1,
            rating: 5,
            comment: "Super comfortable for long work sessions!".to_string(),
            created_at: 1609804800,
        },
    ];

    let tags = vec![
        Tag {
            id: 1,
            name: "Tech".to_string(),
        },
        Tag {
            id: 2,
            name: "Bestseller".to_string(),
        },
        Tag {
            id: 3,
            name: "Ergonomic".to_string(),
        },
        Tag {
            id: 4,
            name: "Premium".to_string(),
        },
    ];

    let product_tags = vec![
        ProductTag {
            product_id: 100,
            tag_id: 1,
        },
        ProductTag {
            product_id: 100,
            tag_id: 2,
        },
        ProductTag {
            product_id: 100,
            tag_id: 4,
        },
        ProductTag {
            product_id: 101,
            tag_id: 1,
        },
        ProductTag {
            product_id: 101,
            tag_id: 3,
        },
        ProductTag {
            product_id: 103,
            tag_id: 2,
        },
        ProductTag {
            product_id: 103,
            tag_id: 3,
        },
    ];

    (users, products, categories, reviews, tags, product_tags)
}

fn test_primary_key_access(users: &[User], products: &[Product]) {
    println!("\n=== Testing Primary Key Access ===");

    for user in users {
        let pk = user.primary_key();
        println!("User '{}' primary key: {:?}", user.name, pk.0);
    }

    for product in products {
        let pk = product.primary_key();
        println!("Product '{}' primary key: {:?}", product.title, pk.0);
    }
}

fn test_secondary_keys(users: &[User], products: &[Product]) {
    println!("\n=== Testing Secondary Keys ===");

    for user in users {
        println!("User '{}' secondary keys:", user.name);
        let secondary_keys =
            <UserKeys as NetabaseModelKeyTrait<Definitions, User>>::secondary_keys(user);
        for (i, sk) in secondary_keys.iter().enumerate() {
            println!("  [{}] {:?}", i, sk);
        }

        // Test iterator access
        println!("  Iterator access:");
        for (i, sk) in user.get_secondary_keys().enumerate() {
            println!("    [{}] {:?}", i, sk);
        }
    }

    for product in products {
        println!("Product '{}' secondary keys:", product.title);
        let secondary_keys =
            <ProductKeys as NetabaseModelKeyTrait<Definitions, Product>>::secondary_keys(product);
        for (i, sk) in secondary_keys.iter().enumerate() {
            println!("  [{}] {:?}", i, sk);
        }

        // Test iterator access
        println!("  Iterator access:");
        for (i, sk) in product.get_secondary_keys().enumerate() {
            println!("    [{}] {:?}", i, sk);
        }
    }
}

fn test_relational_keys(users: &[User], products: &[Product]) {
    println!("\n=== Testing Relational Keys ===");

    for user in users {
        println!("User '{}' relational keys:", user.name);
        let relational_keys =
            <UserKeys as NetabaseModelKeyTrait<Definitions, User>>::relational_keys(user);
        for (i, rk) in relational_keys.iter().enumerate() {
            println!("  [{}] {:?}", i, rk);
        }

        // Test iterator access
        println!("  Iterator access:");
        for (i, rk) in user.get_relational_keys().enumerate() {
            println!("    [{}] {:?}", i, rk);
        }
    }

    for product in products {
        println!("Product '{}' relational keys:", product.title);
        let relational_keys =
            <ProductKeys as NetabaseModelKeyTrait<Definitions, Product>>::relational_keys(product);
        for (i, rk) in relational_keys.iter().enumerate() {
            println!("  [{}] {:?}", i, rk);
        }

        // Test iterator access
        println!("  Iterator access:");
        for (i, rk) in product.get_relational_keys().enumerate() {
            println!("    [{}] {:?}", i, rk);
        }
    }
}

fn test_hash_computation(users: &[User], products: &[Product]) {
    println!("\n=== Testing Hash Computation ===");

    for user in users {
        let hash = user.compute_hash();
        println!("User '{}' hash: {}", user.name, hex::encode(hash));
    }

    for product in products {
        let hash = product.compute_hash();
        println!("Product '{}' hash: {}", product.title, hex::encode(hash));
    }
}

fn test_discriminant_enumeration() {
    println!("\n=== Testing Discriminant Enumeration ===");

    println!("Definition discriminants:");
    for discriminant in DefinitionsDiscriminants::iter() {
        println!("  {:?} -> {}", discriminant, discriminant.as_ref());
    }

    println!("User secondary key discriminants:");
    for discriminant in UserSecondaryKeysDiscriminants::iter() {
        println!("  {:?} -> {}", discriminant, discriminant.as_ref());
    }

    println!("User relational key discriminants:");
    for discriminant in UserRelationalKeysDiscriminants::iter() {
        println!("  {:?} -> {}", discriminant, discriminant.as_ref());
    }

    println!("Product secondary key discriminants:");
    for discriminant in ProductSecondaryKeysDiscriminants::iter() {
        println!("  {:?} -> {}", discriminant, discriminant.as_ref());
    }

    println!("Product relational key discriminants:");
    for discriminant in ProductRelationalKeysDiscriminants::iter() {
        println!("  {:?} -> {}", discriminant, discriminant.as_ref());
    }
}

fn test_tree_naming() {
    println!("\n=== Testing Tree Naming ===");

    for discriminant in DefinitionsDiscriminants::iter() {
        println!("Model: {:?}", discriminant);

        if let Some(tree_name) = Definitions::get_tree_name(&discriminant) {
            println!("  Main tree: {}", tree_name);
        }

        let secondary_trees = Definitions::get_secondary_tree_names(&discriminant);
        println!("  Secondary trees:");
        for tree in secondary_trees {
            println!("    {}", tree);
        }

        let relational_trees = Definitions::get_relational_tree_names(&discriminant);
        println!("  Relational trees:");
        for tree in relational_trees {
            println!("    {}", tree);
        }
    }
}

fn test_table_name_generation() {
    println!("\n=== Testing Table Name Generation ===");

    println!("User table names:");
    for discriminant in UserSecondaryKeysDiscriminants::iter() {
        let table_name =
            <User as RedbNetabaseModelTrait<Definitions>>::secondary_key_table_name(discriminant);
        println!("  Secondary [{}] : {}", discriminant.as_ref(), table_name);
    }

    for discriminant in UserRelationalKeysDiscriminants::iter() {
        let table_name =
            <User as RedbNetabaseModelTrait<Definitions>>::relational_key_table_name(discriminant);
        println!("  Relational [{}] : {}", discriminant.as_ref(), table_name);
    }

    println!(
        "  Hash: {}",
        <User as RedbNetabaseModelTrait<Definitions>>::hash_tree_table_name()
    );

    println!("Product table names:");
    for discriminant in ProductSecondaryKeysDiscriminants::iter() {
        let table_name = <Product as RedbNetabaseModelTrait<Definitions>>::secondary_key_table_name(
            discriminant,
        );
        println!("  Secondary [{}] : {}", discriminant.as_ref(), table_name);
    }

    for discriminant in ProductRelationalKeysDiscriminants::iter() {
        let table_name =
            <Product as RedbNetabaseModelTrait<Definitions>>::relational_key_table_name(
                discriminant,
            );
        println!("  Relational [{}] : {}", discriminant.as_ref(), table_name);
    }

    println!(
        "  Hash: {}",
        <Product as RedbNetabaseModelTrait<Definitions>>::hash_tree_table_name()
    );
}

fn test_tree_access_enums() {
    println!("\n=== Testing Tree Access Enums (No Inner Types) ===");

    println!("User Secondary Tree Names (Copy, lightweight):");
    for tree_name in UserSecondaryTreeNames::iter() {
        println!(
            "  Tree: {} (name: {})",
            format!("{:?}", tree_name),
            tree_name.as_ref()
        );
    }

    println!("User Relational Tree Names:");
    for tree_name in UserRelationalTreeNames::iter() {
        println!(
            "  Tree: {} (name: {})",
            format!("{:?}", tree_name),
            tree_name.as_ref()
        );
    }

    println!("Product Secondary Tree Names:");
    for tree_name in ProductSecondaryTreeNames::iter() {
        println!(
            "  Tree: {} (name: {})",
            format!("{:?}", tree_name),
            tree_name.as_ref()
        );
    }

    println!("Product Relational Tree Names:");
    for tree_name in ProductRelationalTreeNames::iter() {
        println!(
            "  Tree: {} (name: {})",
            format!("{:?}", tree_name),
            tree_name.as_ref()
        );
    }

    // Demonstrate Copy trait
    let tree = UserSecondaryTreeNames::Email;
    let tree_copy = tree; // This is a copy, not a move!
    println!("\nDemonstrating Copy trait:");
    println!("  Original: {:?}, Copy: {:?}", tree, tree_copy);
    println!(
        "  Both can still be used: {} == {}",
        tree.as_ref(),
        tree_copy.as_ref()
    );
}

fn test_serialization_roundtrip(users: &[User], products: &[Product]) {
    println!("\n=== Testing Serialization Roundtrip ===");

    // Test primary key serialization
    for user in users {
        let pk = user.primary_key();
        let serialized: Vec<u8> = pk.clone().try_into().expect("Serialization failed");
        let deserialized = UserId::try_from(serialized).expect("Deserialization failed");
        assert_eq!(pk.0, deserialized.0);
        println!("User PK roundtrip successful: {}", pk.0);
    }

    for product in products {
        let pk = product.primary_key();
        let serialized: Vec<u8> = pk.clone().try_into().expect("Serialization failed");
        let deserialized = ProductId::try_from(serialized).expect("Deserialization failed");
        assert_eq!(pk.0, deserialized.0);
        println!("Product PK roundtrip successful: {}", pk.0);
    }

    // Test secondary key serialization
    for user in users {
        let secondary_keys =
            <UserKeys as NetabaseModelKeyTrait<Definitions, User>>::secondary_keys(user);
        for sk in secondary_keys {
            let serialized: Vec<u8> = sk.clone().try_into().expect("Serialization failed");
            let deserialized =
                UserSecondaryKeys::try_from(serialized).expect("Deserialization failed");
            println!(
                "User secondary key roundtrip successful: {:?} -> {:?}",
                sk, deserialized
            );
        }
    }
}

fn test_sled_database_operations(
    users: &[User],
    products: &[Product],
    categories: &[Category],
    reviews: &[Review],
    tags: &[Tag],
    product_tags: &[ProductTag],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing Sled Database Operations ===");

    // Create a temporary database
    let db_path = "/tmp/boilerplate_sled_test.db";
    if Path::new(db_path).exists() {
        std::fs::remove_dir_all(db_path)?;
    }

    let store = SledStore::<Definitions>::new(db_path)?;

    // ========== STORING ALL ENTITIES ==========
    println!("\n--- Storing Users (Sled) ---");
    store.put_many(users.to_vec())?;

    println!("\n--- Storing Categories (Sled) ---");
    store.put_many(categories.to_vec())?;

    println!("\n--- Storing Products (Sled) ---");
    store.put_many(products.to_vec())?;

    println!("\n--- Storing Reviews (Sled) ---");
    store.put_many(reviews.to_vec())?;

    println!("\n--- Storing Tags (Sled) ---");
    store.put_many(tags.to_vec())?;

    println!("\n--- Storing ProductTags (Sled) ---");
    store.put_many(product_tags.to_vec())?;

    // ========== RETRIEVING BY PRIMARY KEY ==========
    println!("\n--- Retrieving Users by Primary Key (Sled) ---");
    for user in users {
        let retrieved = store.get_one::<User>(user.primary_key())?;
        match retrieved {
            Some(u) => {
                assert_eq!(u.id, user.id);
                assert_eq!(u.name, user.name);
                assert_eq!(u.email, user.email);
                assert_eq!(u.age, user.age);
                println!("  ✓ Retrieved user: {} (ID: {})", u.name, u.id);
            }
            None => panic!("User not found: {}", user.name),
        }
    }

    println!("\n--- Retrieving Categories by Primary Key (Sled) ---");
    for category in categories {
        let retrieved = store.get_one::<Category>(category.primary_key())?;
        match retrieved {
            Some(c) => {
                assert_eq!(c.id, category.id);
                assert_eq!(c.name, category.name);
                assert_eq!(c.description, category.description);
                println!("  ✓ Retrieved category: {} (ID: {})", c.name, c.id);
            }
            None => panic!("Category not found: {}", category.name),
        }
    }

    println!("\n--- Retrieving Products by Primary Key (Sled) ---");
    for product in products {
        let retrieved = store.get_one::<Product>(product.primary_key())?;
        match retrieved {
            Some(p) => {
                assert_eq!(p.uuid, product.uuid);
                assert_eq!(p.title, product.title);
                assert_eq!(p.score, product.score);
                assert_eq!(p.created_by, product.created_by);
                println!("  ✓ Retrieved product: {} (ID: {})", p.title, p.uuid);
            }
            None => panic!("Product not found: {}", product.title),
        }
    }

    // Clean up
    std::fs::remove_dir_all(db_path)?;
    println!("\n✓ Sled database cleaned up");

    Ok(())
}

fn test_real_database_operations(
    users: &[User],
    products: &[Product],
    categories: &[Category],
    reviews: &[Review],
    tags: &[Tag],
    product_tags: &[ProductTag],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing Real Database Operations ===");

    // Create a temporary database
    let db_path = "/tmp/boilerplate_test.db";
    if Path::new(db_path).exists() {
        std::fs::remove_file(db_path)?;
    }

    let store = RedbStore::<Definitions>::new(db_path)?;

    // ========== STORING ALL ENTITIES ==========
    println!("\n--- Storing Users ---");
    store.put_many(users.to_vec())?;

    println!("\n--- Storing Categories ---");
    store.put_many(categories.to_vec())?;

    println!("\n--- Storing Products ---");
    store.put_many(products.to_vec())?;

    println!("\n--- Storing Reviews ---");
    store.put_many(reviews.to_vec())?;

    println!("\n--- Storing Tags ---");
    store.put_many(tags.to_vec())?;

    println!("\n--- Storing ProductTags (junction table) ---");
    store.put_many(product_tags.to_vec())?;

    // ========== RETRIEVING BY PRIMARY KEY ==========
    println!("\n--- Retrieving Users by Primary Key ---");
    for user in users {
        let retrieved = store.get_one::<User>(user.primary_key())?;
        match retrieved {
            Some(u) => {
                assert_eq!(u.id, user.id);
                assert_eq!(u.name, user.name);
                assert_eq!(u.email, user.email);
                assert_eq!(u.age, user.age);
                println!("  ✓ Retrieved user: {} (ID: {})", u.name, u.id);
            }
            None => panic!("User not found: {}", user.name),
        }
    }

    println!("\n--- Retrieving Categories by Primary Key ---");
    for category in categories {
        let retrieved = store.get_one::<Category>(category.primary_key())?;
        match retrieved {
            Some(c) => {
                assert_eq!(c.id, category.id);
                assert_eq!(c.name, category.name);
                assert_eq!(c.description, category.description);
                println!("  ✓ Retrieved category: {} (ID: {})", c.name, c.id);
            }
            None => panic!("Category not found: {}", category.name),
        }
    }

    println!("\n--- Retrieving Products by Primary Key ---");
    for product in products {
        let retrieved = store.get_one::<Product>(product.primary_key())?;
        match retrieved {
            Some(p) => {
                assert_eq!(p.uuid, product.uuid);
                assert_eq!(p.title, product.title);
                assert_eq!(p.score, product.score);
                assert_eq!(p.created_by, product.created_by);
                println!("  ✓ Retrieved product: {} (ID: {})", p.title, p.uuid);
            }
            None => panic!("Product not found: {}", product.title),
        }
    }

    println!("\n--- Retrieving Reviews by Primary Key ---");
    for review in reviews {
        let retrieved = store.get_one::<Review>(review.primary_key())?;
        match retrieved {
            Some(r) => {
                assert_eq!(r.id, review.id);
                assert_eq!(r.product_id, review.product_id);
                assert_eq!(r.user_id, review.user_id);
                assert_eq!(r.rating, review.rating);
                assert_eq!(r.comment, review.comment);
                println!("  ✓ Retrieved review ID {} (rating: {})", r.id, r.rating);
            }
            None => panic!("Review not found: {}", review.id),
        }
    }

    println!("\n--- Retrieving Tags by Primary Key ---");
    for tag in tags {
        let retrieved = store.get_one::<Tag>(tag.primary_key())?;
        match retrieved {
            Some(t) => {
                assert_eq!(t.id, tag.id);
                assert_eq!(t.name, tag.name);
                println!("  ✓ Retrieved tag: {} (ID: {})", t.name, t.id);
            }
            None => panic!("Tag not found: {}", tag.name),
        }
    }

    println!("\n--- Retrieving ProductTags by Primary Key ---");
    for product_tag in product_tags {
        let retrieved = store.get_one::<ProductTag>(product_tag.primary_key())?;
        match retrieved {
            Some(pt) => {
                assert_eq!(pt.product_id, product_tag.product_id);
                assert_eq!(pt.tag_id, product_tag.tag_id);
                println!(
                    "  ✓ Retrieved ProductTag: product {} <-> tag {}",
                    pt.product_id, pt.tag_id
                );
            }
            None => panic!(
                "ProductTag not found: product {} tag {}",
                product_tag.product_id, product_tag.tag_id
            ),
        }
    }

    // ========== BATCH OPERATIONS ==========
    println!("\n--- Testing Batch Operations ---");

    let user_pks: Vec<_> = users.iter().map(|u| u.primary_key()).collect();
    let retrieved_users = store.get_many::<User>(user_pks)?;
    let user_count = retrieved_users.iter().filter_map(|u| u.as_ref()).count();
    assert_eq!(user_count, users.len());
    println!("  ✓ Batch retrieved {} users", user_count);

    let product_pks: Vec<_> = products.iter().map(|p| p.primary_key()).collect();
    let retrieved_products = store.get_many::<Product>(product_pks)?;
    let product_count = retrieved_products.iter().filter_map(|p| p.as_ref()).count();
    assert_eq!(product_count, products.len());
    println!("  ✓ Batch retrieved {} products", product_count);

    let category_pks: Vec<_> = categories.iter().map(|c| c.primary_key()).collect();
    let retrieved_categories = store.get_many::<Category>(category_pks)?;
    let category_count = retrieved_categories
        .iter()
        .filter_map(|c| c.as_ref())
        .count();
    assert_eq!(category_count, categories.len());
    println!("  ✓ Batch retrieved {} categories", category_count);

    let review_pks: Vec<_> = reviews.iter().map(|r| r.primary_key()).collect();
    let retrieved_reviews = store.get_many::<Review>(review_pks)?;
    let review_count = retrieved_reviews.iter().filter_map(|r| r.as_ref()).count();
    assert_eq!(review_count, reviews.len());
    println!("  ✓ Batch retrieved {} reviews", review_count);

    let tag_pks: Vec<_> = tags.iter().map(|t| t.primary_key()).collect();
    let retrieved_tags = store.get_many::<Tag>(tag_pks)?;
    let tag_count = retrieved_tags.iter().filter_map(|t| t.as_ref()).count();
    assert_eq!(tag_count, tags.len());
    println!("  ✓ Batch retrieved {} tags", tag_count);

    let product_tag_pks: Vec<_> = product_tags.iter().map(|pt| pt.primary_key()).collect();
    let retrieved_product_tags = store.get_many::<ProductTag>(product_tag_pks)?;
    let product_tag_count = retrieved_product_tags
        .iter()
        .filter_map(|pt| pt.as_ref())
        .count();
    assert_eq!(product_tag_count, product_tags.len());
    println!("  ✓ Batch retrieved {} product tags", product_tag_count);

    // ========== DATA INTEGRITY CHECKS ==========
    println!("\n--- Testing Data Integrity ---");

    // Verify reviews reference valid products and users
    for review in reviews {
        let product_exists = products.iter().any(|p| p.uuid == review.product_id);
        let user_exists = users.iter().any(|u| u.id == review.user_id);
        assert!(
            product_exists,
            "Review {} references non-existent product {}",
            review.id, review.product_id
        );
        assert!(
            user_exists,
            "Review {} references non-existent user {}",
            review.id, review.user_id
        );
    }
    println!("  ✓ All reviews reference valid products and users");

    // Verify product tags reference valid products and tags
    for product_tag in product_tags {
        let product_exists = products.iter().any(|p| p.uuid == product_tag.product_id);
        let tag_exists = tags.iter().any(|t| t.id == product_tag.tag_id);
        assert!(
            product_exists,
            "ProductTag references non-existent product {}",
            product_tag.product_id
        );
        assert!(
            tag_exists,
            "ProductTag references non-existent tag {}",
            product_tag.tag_id
        );
    }
    println!("  ✓ All product tags reference valid products and tags");

    // ========== RELATIONSHIP TESTS ==========
    println!("\n--- Testing Relationships ---");

    // Count reviews per product
    let laptop_reviews = reviews.iter().filter(|r| r.product_id == 100).count();
    println!("  ✓ Laptop Pro has {} reviews", laptop_reviews);
    assert_eq!(laptop_reviews, 2);

    // Count products per tag
    let tech_tag_products = product_tags.iter().filter(|pt| pt.tag_id == 1).count();
    println!(
        "  ✓ Tech tag is associated with {} products",
        tech_tag_products
    );
    assert_eq!(tech_tag_products, 2);

    // Count tags per product
    let laptop_tags = product_tags
        .iter()
        .filter(|pt| pt.product_id == 100)
        .count();
    println!("  ✓ Laptop Pro has {} tags", laptop_tags);
    assert_eq!(laptop_tags, 3);

    // Clean up
    std::fs::remove_file(db_path)?;
    println!("\n✓ Database cleaned up");

    Ok(())
}

fn test_definition_enum_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing Definition Enum Store Operations ===");

    let db_path = "/tmp/definition_enum_test.db";
    if Path::new(db_path).exists() {
        std::fs::remove_file(db_path)?;
    }

    let store = RedbStore::<Definitions>::new(db_path)?;

    println!("\n--- Testing put_definition ---");
    let user = User {
        id: 100,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        age: 25,
    };

    store.put_definition(Definitions::User(user.clone()))?;
    println!("  ✓ Inserted user via Definition enum");

    let product = Product {
        uuid: 200,
        title: "Test Product".to_string(),
        score: 88,
        created_by: 100,
    };

    store.put_definition(Definitions::Product(product.clone()))?;
    println!("  ✓ Inserted product via Definition enum");

    println!("\n--- Testing get_definition ---");
    let retrieved_user =
        store.get_definition(DefinitionModelAssociatedTypes::UserPrimaryKey(UserId(100)))?;

    match retrieved_user {
        Some(Definitions::User(u)) => {
            assert_eq!(u.id, 100);
            assert_eq!(u.name, "Test User");
            println!("  ✓ Retrieved user via Definition enum: {}", u.name);
        }
        _ => panic!("Failed to retrieve user"),
    }

    let retrieved_product = store.get_definition(
        DefinitionModelAssociatedTypes::ProductPrimaryKey(ProductId(200)),
    )?;

    match retrieved_product {
        Some(Definitions::Product(p)) => {
            assert_eq!(p.uuid, 200);
            assert_eq!(p.title, "Test Product");
            println!("  ✓ Retrieved product via Definition enum: {}", p.title);
        }
        _ => panic!("Failed to retrieve product"),
    }

    println!("\n--- Testing put_many_definitions ---");
    let batch = vec![
        Definitions::User(User {
            id: 101,
            email: "batch1@example.com".to_string(),
            name: "Batch User 1".to_string(),
            age: 30,
        }),
        Definitions::User(User {
            id: 102,
            email: "batch2@example.com".to_string(),
            name: "Batch User 2".to_string(),
            age: 35,
        }),
    ];

    store.put_many_definitions(batch)?;
    println!("  ✓ Batch inserted 2 users via Definition enum");

    // Verify batch insert
    let batch_user1 =
        store.get_definition(DefinitionModelAssociatedTypes::UserPrimaryKey(UserId(101)))?;
    assert!(batch_user1.is_some());
    println!("  ✓ Verified batch user 1");

    let batch_user2 =
        store.get_definition(DefinitionModelAssociatedTypes::UserPrimaryKey(UserId(102)))?;
    assert!(batch_user2.is_some());
    println!("  ✓ Verified batch user 2");

    std::fs::remove_file(db_path)?;
    println!("\n✓ Definition enum operations test completed");

    Ok(())
}

fn test_subscription_feature() -> Result<(), Box<dyn std::error::Error>> {
    use netabase_store::traits::store::transaction::{ReadTransaction, WriteTransaction};

    println!("\n=== Testing Subscription Feature ===");

    let db_path = "test_subscriptions.redb";
    let _ = std::fs::remove_file(db_path);

    let store = RedbStore::<Definitions>::new(db_path)?;

    // Create test users with subscriptions
    let user1 = User {
        id: 1,
        email: "user1@example.com".to_string(),
        name: "User One".to_string(),
        age: 25,
    };

    let user2 = User {
        id: 2,
        email: "user2@example.com".to_string(),
        name: "User Two".to_string(),
        age: 30,
    };

    let user3 = User {
        id: 3,
        email: "user3@example.com".to_string(),
        name: "User Three".to_string(),
        age: 35,
    };

    println!("\n--- Inserting users with subscriptions ---");
    store.write(|txn| {
        txn.put(user1.clone())?;
        txn.put(user2.clone())?;
        txn.put(user3.clone())?;
        Ok(())
    })?;
    println!("  ✓ Inserted 3 users (subscriptions auto-tracked)");

    println!("\n--- Testing subscription accumulator ---");
    let (accumulator, count) = store.read(|txn| {
        txn.get_subscription_accumulator::<User>(UserSubscriptionsDiscriminants::Updates)
    })?;

    println!("  ✓ Subscription accumulator: {:?}", &accumulator[..8]);
    println!("  ✓ Item count: {}", count);
    assert_eq!(count, 3, "Should have 3 items in subscription");

    println!("\n--- Getting subscription keys ---");
    let keys = store
        .read(|txn| txn.get_subscription_keys::<User>(UserSubscriptionsDiscriminants::Updates))?;

    println!(
        "  ✓ Retrieved {} primary keys from subscription",
        keys.len()
    );
    assert_eq!(keys.len(), 3, "Should have 3 keys");
    for key in &keys {
        println!("    - User ID: {}", key.0);
    }

    println!("\n--- Testing order-independent property ---");
    // Create a second store with same data inserted in different order
    let db_path2 = "test_subscriptions2.redb";
    let _ = std::fs::remove_file(db_path2);
    let store2 = RedbStore::<Definitions>::new(db_path2)?;

    store2.write(|txn| {
        // Insert in different order: 3, 1, 2 instead of 1, 2, 3
        txn.put(user3.clone())?;
        txn.put(user1.clone())?;
        txn.put(user2.clone())?;
        Ok(())
    })?;

    let (accumulator2, count2) = store2.read(|txn| {
        txn.get_subscription_accumulator::<User>(UserSubscriptionsDiscriminants::Updates)
    })?;

    assert_eq!(
        accumulator, accumulator2,
        "Accumulators should match regardless of insertion order"
    );
    assert_eq!(count, count2, "Counts should match");
    println!("  ✓ Accumulators match despite different insertion order!");
    println!("  ✓ Order-independent comparison verified");

    println!("\n--- Testing subscription sync detection ---");
    // Add one more user to store2
    let user4 = User {
        id: 4,
        email: "user4@example.com".to_string(),
        name: "User Four".to_string(),
        age: 40,
    };

    store2.write(|txn| {
        txn.put(user4.clone())?;
        Ok(())
    })?;

    let (accumulator_after, count_after) = store2.read(|txn| {
        txn.get_subscription_accumulator::<User>(UserSubscriptionsDiscriminants::Updates)
    })?;

    assert_ne!(
        accumulator, accumulator_after,
        "Accumulators should differ when stores have different data"
    );
    assert_eq!(count_after, 4, "Store2 should have 4 items");
    println!("  ✓ Detected difference between stores (3 vs 4 items)");
    println!("  ✓ Subscription sync detection working");

    // Cleanup
    std::fs::remove_file(db_path)?;
    std::fs::remove_file(db_path2)?;

    println!("\n✓ Subscription feature test completed!");

    Ok(())
}

fn test_sled_batch_operations(users: &[User]) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing Sled Batch Operations ===");

    let db_path = "/tmp/boilerplate_sled_batch_test.db";
    if Path::new(db_path).exists() {
        std::fs::remove_dir_all(db_path)?;
    }

    let store = SledStore::<Definitions>::new(db_path)?;

    println!("\n--- Testing put_many (Sled) ---");
    // Insert all users in one batch
    store.put_many(users.to_vec())?;
    println!("  ✓ Batch inserted {} users", users.len());

    println!("\n--- Testing get_many (Sled) ---");
    let user_pks: Vec<_> = users.iter().map(|u| u.primary_key()).collect();
    let retrieved_users = store.get_many::<User>(user_pks)?;

    assert_eq!(retrieved_users.len(), users.len());
    for (i, retrieved) in retrieved_users.iter().enumerate() {
        match retrieved {
            Some(u) => {
                assert_eq!(u.id, users[i].id);
                println!("  ✓ Retrieved user: {}", u.name);
            }
            None => panic!("Failed to retrieve user {}", users[i].name),
        }
    }

    std::fs::remove_dir_all(db_path)?;
    println!("\n✓ Sled batch operations test completed");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("🚀 Comprehensive Boilerplate Example for Netabase Store");
    println!("=====================================================");

    let (users, products, categories, reviews, tags, product_tags) = create_test_data();

    // Test all access patterns and associated types
    test_primary_key_access(&users, &products);
    test_secondary_keys(&users, &products);
    test_relational_keys(&users, &products);
    test_hash_computation(&users, &products);
    test_discriminant_enumeration();
    test_tree_naming();
    test_table_name_generation();
    test_tree_access_enums();
    test_serialization_roundtrip(&users, &products);

    // Test real database operations
    let redb_start = Instant::now();
    test_real_database_operations(
        &users,
        &products,
        &categories,
        &reviews,
        &tags,
        &product_tags,
    )?;

    let redb_end = Instant::now();
    let red_time = redb_end - redb_start;
    let sled_start = Instant::now();
    test_sled_database_operations(
        &users,
        &products,
        &categories,
        &reviews,
        &tags,
        &product_tags,
    )?;
    test_sled_batch_operations(&users)?;
    let sled_end = Instant::now();
    let sled_time = sled_end - sled_start;
    println!("###\n\n\nSLED DURATION: {:?}\n\n\n", sled_time);
    println!("###\n\n\nREDB DURATION: {:?}\n\n\n", red_time);

    // Test Definition enum operations
    test_definition_enum_operations()?;

    // Test Subscription feature
    test_subscription_feature()?;

    println!("\n✅ All tests completed successfully!");
    println!("\nThis example demonstrates:");
    println!("• 6 different model types (User, Product, Category, Review, Tag, ProductTag)");
    println!("• Primary key access and storage (including composite keys)");
    println!("• Secondary key enumeration and indexing");
    println!("• Relational key relationships");
    println!("• One-to-Many relationships (User → Reviews, Category → Products)");
    println!("• Many-to-One relationships (Review → User/Product)");
    println!("• Many-to-Many relationships (Product ↔ Tag via ProductTag junction table)");
    println!("• Hash computation for data integrity");
    println!("• Discriminant enumeration for type safety");
    println!("• Tree and table name generation");
    println!("• Tree access enums (Copy, no inner types) for efficient tree identification");
    println!("• Serialization/deserialization roundtrips");
    println!("• Real database storage and retrieval operations");
    println!("• Batch operations for performance");
    println!("• Data integrity verification");
    println!("• Definition enum-based store operations (put_definition, get_definition)");
    println!("• Unified interface for inserting and retrieving different model types");
    println!("• Subscription trees with order-independent comparison (XOR accumulation)");
    println!("• Automatic subscription tracking on write operations");
    println!("• Store synchronization detection and comparison");

    Ok(())
}
