use derive_more::TryInto;
use bincode::{Decode, Encode};
use netabase_store::{
    databases::redb_store::RedbStore,
    error::{NetabaseResult, NetabaseError},
    traits::{
        definition::{NetabaseDefinitionTrait, DiscriminantName, key::NetabaseDefinitionKeyTrait, ModelAssociatedTypesExt},
        model::{NetabaseModelTrait, RedbNetabaseModelTrait, key::NetabaseModelKeyTrait, RelationalLink},
        store::{
            tree_manager::{TreeManager, AllTrees},
            store::StoreTrait,
        },
    },
};
use redb::{Key, Value, TableDefinition, TypeName, WriteTransaction, ReadableTable};
use std::{borrow::Cow, path::Path};
use strum::{EnumIter, EnumDiscriminants, AsRefStr, IntoEnumIterator, IntoDiscriminant};

// ================================================================================= ================
// Model 1: User
// ================================================================================= ================

#[derive(Debug, Clone, PartialEq, Default)]
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
        let (value, _): (u64, usize) = bincode::decode_from_slice(&data, bincode::config::standard())?;
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
        User { id, email, name, age }
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
        let bytes: Vec<u8> = value.clone().try_into().expect("Failed to serialize UserSecondaryKeys");
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
        let (value, _): (UserSecondaryKeys, usize) = bincode::decode_from_slice(&data, bincode::config::standard())?;
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
        let bytes: Vec<u8> = value.clone().try_into().expect("Failed to serialize UserRelationalKeys");
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
        let (value, _): (UserRelationalKeys, usize) = bincode::decode_from_slice(&data, bincode::config::standard())?;
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

    fn relational_keys(model: &User) -> Vec<Self::RelationalEnum> {
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
            ].into_iter()
        }
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        // Relational keys should be fetched from the database, not generated from the model
        // This is a limitation of the current API - it should be lazy or require a transaction context
        UserRelationalKeysIter {
            iter: vec![].into_iter()
        }
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

    fn wrap_secondary_key_discriminant(key: UserSecondaryKeysDiscriminants) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::UserSecondaryKeyDiscriminant(key)
    }

    fn wrap_relational_key_discriminant(key: UserRelationalKeysDiscriminants) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::UserRelationalKeyDiscriminant(key)
    }
}

impl RedbNetabaseModelTrait<Definitions> for User {
    fn definition<'a>(db: &RedbStore<Definitions>) -> TableDefinition<'a, Self::PrimaryKey, Self> {
        let _ = db; // Avoid unused parameter warning
        TableDefinition::new(Self::MODEL_TREE_NAME.as_ref())
    }
    
    fn secondary_key_table_name(
        key_discriminant: UserSecondaryKeysDiscriminants,
    ) -> String {
        format!("User_sec_{}", key_discriminant.as_ref())
    }
    
    fn relational_key_table_name(
        key_discriminant: UserRelationalKeysDiscriminants,
    ) -> String {
        format!("User_rel_{}", key_discriminant.as_ref())
    }
    
    fn hash_tree_table_name() -> String {
        "User_hash".to_string()
    }
}

// ================================================================================= ================
// Model 2: Product
// ================================================================================= ================

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Product {
    pub uuid: u128,
    pub title: String,
    pub score: i32,
    pub created_by: u64,  // Foreign key to User.id
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
        let (value, _): (u128, usize) = bincode::decode_from_slice(&data, bincode::config::standard())?;
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
        Product { uuid, title, score, created_by }
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
        let bytes: Vec<u8> = value.clone().try_into().expect("Failed to serialize ProductSecondaryKeys");
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
        let (value, _): (ProductSecondaryKeys, usize) = bincode::decode_from_slice(&data, bincode::config::standard())?;
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
        let bytes: Vec<u8> = value.clone().try_into().expect("Failed to serialize ProductRelationalKeys");
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
        let (value, _): (ProductRelationalKeys, usize) = bincode::decode_from_slice(&data, bincode::config::standard())?;
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
        vec![
            ProductRelationalKeys::CreatedBy(UserId(model.created_by)),
        ]
    }
}

impl NetabaseModelTrait<Definitions> for Product {
    type Keys = ProductKeys;
    const MODEL_TREE_NAME: DefinitionsDiscriminants = DefinitionsDiscriminants::Product;

    type SecondaryKeys = ProductSecondaryKeysIter;
    type RelationalKeys = ProductRelationalKeysIter;
    type Hash = [u8; 32]; // Blake3 hash

    fn primary_key(&self) -> Self::PrimaryKey {
        ProductId(self.uuid)
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        ProductSecondaryKeysIter {
            iter: vec![
                ProductSecondaryKeys::Title(ProductTitle(self.title.clone())),
                ProductSecondaryKeys::Score(ProductScore(self.score)),
            ].into_iter()
        }
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        ProductRelationalKeysIter {
            iter: vec![
                ProductRelationalKeys::CreatedBy(UserId(self.created_by)),
            ].into_iter()
        }
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

    fn wrap_secondary_key_discriminant(key: ProductSecondaryKeysDiscriminants) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductSecondaryKeyDiscriminant(key)
    }

    fn wrap_relational_key_discriminant(key: ProductRelationalKeysDiscriminants) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductRelationalKeyDiscriminant(key)
    }
}

impl RedbNetabaseModelTrait<Definitions> for Product {
    fn definition<'a>(db: &RedbStore<Definitions>) -> TableDefinition<'a, Self::PrimaryKey, Self> {
        let _ = db; // Avoid unused parameter warning
        TableDefinition::new(Self::MODEL_TREE_NAME.as_ref())
    }
    
    fn secondary_key_table_name(
        key_discriminant: ProductSecondaryKeysDiscriminants,
    ) -> String {
        format!("Product_sec_{}", key_discriminant.as_ref())
    }
    
    fn relational_key_table_name(
        key_discriminant: ProductRelationalKeysDiscriminants,
    ) -> String {
        format!("Product_rel_{}", key_discriminant.as_ref())
    }
    
    fn hash_tree_table_name() -> String {
        "Product_hash".to_string()
    }
}

// ================================================================================= ================
// Model 3: Category
// ================================================================================= ================

#[derive(Debug, Clone, PartialEq, Default)]
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
        let (value, _): (u64, usize) = bincode::decode_from_slice(&data, bincode::config::standard())?;
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
        Category { id, name, description }
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
        let bytes: Vec<u8> = value.clone().try_into().expect("Failed to serialize CategorySecondaryKeys");
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
        let (value, _): (CategorySecondaryKeys, usize) = bincode::decode_from_slice(&data, bincode::config::standard())?;
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
        let bytes: Vec<u8> = value.clone().try_into().expect("Failed to serialize CategoryRelationalKeys");
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
        let (value, _): (CategoryRelationalKeys, usize) = bincode::decode_from_slice(&data, bincode::config::standard())?;
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
        vec![
            CategorySecondaryKeys::Name(CategoryName(model.name.clone())),
        ]
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
    type Hash = [u8; 32];

    fn primary_key(&self) -> Self::PrimaryKey {
        CategoryId(self.id)
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        CategorySecondaryKeysIter {
            iter: vec![
                CategorySecondaryKeys::Name(CategoryName(self.name.clone())),
            ].into_iter()
        }
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        CategoryRelationalKeysIter {
            iter: vec![].into_iter()
        }
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

    fn wrap_secondary_key_discriminant(key: CategorySecondaryKeysDiscriminants) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::CategorySecondaryKeyDiscriminant(key)
    }

    fn wrap_relational_key_discriminant(key: CategoryRelationalKeysDiscriminants) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::CategoryRelationalKeyDiscriminant(key)
    }
}

impl RedbNetabaseModelTrait<Definitions> for Category {
    fn definition<'a>(db: &RedbStore<Definitions>) -> TableDefinition<'a, Self::PrimaryKey, Self> {
        let _ = db;
        TableDefinition::new(Self::MODEL_TREE_NAME.as_ref())
    }

    fn secondary_key_table_name(
        key_discriminant: CategorySecondaryKeysDiscriminants,
    ) -> String {
        format!("Category_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(
        key_discriminant: CategoryRelationalKeysDiscriminants,
    ) -> String {
        format!("Category_rel_{}", key_discriminant.as_ref())
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
        let (value, _): (u64, usize) = bincode::decode_from_slice(&data, bincode::config::standard())?;
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
    type SelfType<'a> = Self where Self: 'a;
    type AsBytes<'a> = Vec<u8> where Self: 'a;

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
        let bytes: Vec<u8> = value.clone().try_into().expect("Failed to serialize ReviewSecondaryKeys");
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
        let bytes: Vec<u8> = value.clone().try_into().expect("Failed to serialize ReviewRelationalKeys");
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
        bincode::encode_to_vec(value, config)
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
        bincode::encode_to_vec(value, config)
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
    type Hash = [u8; 32];

    fn primary_key(&self) -> ReviewId {
        ReviewId(self.id)
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        ReviewSecondaryKeysIter {
            iter: ReviewKeys::secondary_keys(self).into_iter()
        }
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        ReviewRelationalKeysIter {
            iter: ReviewKeys::relational_keys(self).into_iter()
        }
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

    fn wrap_secondary_key_discriminant(key: ReviewSecondaryKeysDiscriminants) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ReviewSecondaryKeyDiscriminant(key)
    }

    fn wrap_relational_key_discriminant(key: ReviewRelationalKeysDiscriminants) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ReviewRelationalKeyDiscriminant(key)
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

    fn secondary_key_table_name(
        key_discriminant: ReviewSecondaryKeysDiscriminants,
    ) -> String {
        format!("Review_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(
        key_discriminant: ReviewRelationalKeysDiscriminants,
    ) -> String {
        format!("Review_rel_{}", key_discriminant.as_ref())
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
        let (value, _): (u64, usize) = bincode::decode_from_slice(&data, bincode::config::standard())?;
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
    type SelfType<'a> = Self where Self: 'a;
    type AsBytes<'a> = Vec<u8> where Self: 'a;

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
        let bytes: Vec<u8> = value.clone().try_into().expect("Failed to serialize TagSecondaryKeys");
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
        let bytes: Vec<u8> = value.clone().try_into().expect("Failed to serialize TagRelationalKeys");
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
        bincode::encode_to_vec(value, config)
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
        bincode::encode_to_vec(value, config)
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
    type Hash = [u8; 32];

    fn primary_key(&self) -> TagId {
        TagId(self.id)
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        TagSecondaryKeysIter {
            iter: TagKeys::secondary_keys(self).into_iter()
        }
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        TagRelationalKeysIter {
            iter: TagKeys::relational_keys(self).into_iter()
        }
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

    fn wrap_secondary_key_discriminant(key: TagSecondaryKeysDiscriminants) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::TagSecondaryKeyDiscriminant(key)
    }

    fn wrap_relational_key_discriminant(key: TagRelationalKeysDiscriminants) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::TagRelationalKeyDiscriminant(key)
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

    fn secondary_key_table_name(
        key_discriminant: TagSecondaryKeysDiscriminants,
    ) -> String {
        format!("Tag_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(
        key_discriminant: TagRelationalKeysDiscriminants,
    ) -> String {
        format!("Tag_rel_{}", key_discriminant.as_ref())
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
        let (value, _): (ProductTagId, usize) = bincode::decode_from_slice(&data, bincode::config::standard())?;
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
    type SelfType<'a> = Self where Self: 'a;
    type AsBytes<'a> = Vec<u8> where Self: 'a;

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
    type SelfType<'a> = Self where Self: 'a;
    type AsBytes<'a> = Vec<u8> where Self: 'a;

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

// Conversion implementations for ProductTagSecondaryKeys
impl TryFrom<ProductTagSecondaryKeys> for Vec<u8> {
    type Error = Box<bincode::error::EncodeError>;

    fn try_from(value: ProductTagSecondaryKeys) -> Result<Self, Self::Error> {
        let config = bincode::config::standard();
        bincode::encode_to_vec(value, config)
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
        bincode::encode_to_vec(value, config)
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
    type Hash = [u8; 32];

    fn primary_key(&self) -> ProductTagId {
        ProductTagId {
            product_id: self.product_id,
            tag_id: self.tag_id,
        }
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        ProductTagSecondaryKeysIter {
            iter: ProductTagKeys::secondary_keys(self).into_iter()
        }
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        ProductTagRelationalKeysIter {
            iter: ProductTagKeys::relational_keys(self).into_iter()
        }
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

    fn wrap_secondary_key_discriminant(key: ProductTagSecondaryKeysDiscriminants) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductTagSecondaryKeyDiscriminant(key)
    }

    fn wrap_relational_key_discriminant(key: ProductTagRelationalKeysDiscriminants) -> DefinitionModelAssociatedTypes {
        DefinitionModelAssociatedTypes::ProductTagRelationalKeyDiscriminant(key)
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

    fn secondary_key_table_name(
        key_discriminant: ProductTagSecondaryKeysDiscriminants,
    ) -> String {
        format!("ProductTag_sec_{}", key_discriminant.as_ref())
    }

    fn relational_key_table_name(
        key_discriminant: ProductTagRelationalKeysDiscriminants,
    ) -> String {
        format!("ProductTag_rel_{}", key_discriminant.as_ref())
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
    UserSecondaryKeyDiscriminant(UserSecondaryKeysDiscriminants),
    UserRelationalKeyDiscriminant(UserRelationalKeysDiscriminants),

    // Product-related types
    ProductPrimaryKey(ProductId),
    ProductModel(Product),
    ProductSecondaryKey(ProductSecondaryKeys),
    ProductRelationalKey(ProductRelationalKeys),
    ProductSecondaryKeyDiscriminant(ProductSecondaryKeysDiscriminants),
    ProductRelationalKeyDiscriminant(ProductRelationalKeysDiscriminants),

    // Category-related types
    CategoryPrimaryKey(CategoryId),
    CategoryModel(Category),
    CategorySecondaryKey(CategorySecondaryKeys),
    CategoryRelationalKey(CategoryRelationalKeys),
    CategorySecondaryKeyDiscriminant(CategorySecondaryKeysDiscriminants),
    CategoryRelationalKeyDiscriminant(CategoryRelationalKeysDiscriminants),

    // Review-related types
    ReviewPrimaryKey(ReviewId),
    ReviewModel(Review),
    ReviewSecondaryKey(ReviewSecondaryKeys),
    ReviewRelationalKey(ReviewRelationalKeys),
    ReviewSecondaryKeyDiscriminant(ReviewSecondaryKeysDiscriminants),
    ReviewRelationalKeyDiscriminant(ReviewRelationalKeysDiscriminants),

    // Tag-related types
    TagPrimaryKey(TagId),
    TagModel(Tag),
    TagSecondaryKey(TagSecondaryKeys),
    TagRelationalKey(TagRelationalKeys),
    TagSecondaryKeyDiscriminant(TagSecondaryKeysDiscriminants),
    TagRelationalKeyDiscriminant(TagRelationalKeysDiscriminants),

    // ProductTag-related types
    ProductTagPrimaryKey(ProductTagId),
    ProductTagModel(ProductTag),
    ProductTagSecondaryKey(ProductTagSecondaryKeys),
    ProductTagRelationalKey(ProductTagRelationalKeys),
    ProductTagSecondaryKeyDiscriminant(ProductTagSecondaryKeysDiscriminants),
    ProductTagRelationalKeyDiscriminant(ProductTagRelationalKeysDiscriminants),

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
        key: <<M::Keys as NetabaseModelKeyTrait<Definitions, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant
    ) -> Self {
        M::wrap_secondary_key_discriminant(key)
    }
    
    fn from_relational_key_discriminant<M: NetabaseModelTrait<Definitions>>(
        key: <<M::Keys as NetabaseModelKeyTrait<Definitions, M>>::RelationalEnum as IntoDiscriminant>::Discriminant
    ) -> Self {
        M::wrap_relational_key_discriminant(key)
    }
    
    fn from_secondary_key_data<M: NetabaseModelTrait<Definitions>>(
        key: <M::Keys as NetabaseModelKeyTrait<Definitions, M>>::SecondaryEnum
    ) -> Self {
        M::wrap_secondary_key(key)
    }
    
    fn from_relational_key_data<M: NetabaseModelTrait<Definitions>>(
        key: <M::Keys as NetabaseModelKeyTrait<Definitions, M>>::RelationalEnum
    ) -> Self {
        M::wrap_relational_key(key)
    }

    fn insert_model_into_redb(
        &self,
        txn: &WriteTransaction,
        table_name: &str,
        key: &Self,
    ) -> NetabaseResult<()> {
        match (self, key) {
            (DefinitionModelAssociatedTypes::UserModel(model), DefinitionModelAssociatedTypes::UserPrimaryKey(pk)) => {
                let table_def: TableDefinition<UserId, User> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, model)?;
                Ok(())
            },
            (DefinitionModelAssociatedTypes::ProductModel(model), DefinitionModelAssociatedTypes::ProductPrimaryKey(pk)) => {
                let table_def: TableDefinition<ProductId, Product> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, model)?;
                Ok(())
            },
            (DefinitionModelAssociatedTypes::CategoryModel(model), DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk)) => {
                let table_def: TableDefinition<CategoryId, Category> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, model)?;
                Ok(())
            },
            (DefinitionModelAssociatedTypes::ReviewModel(model), DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk)) => {
                let table_def: TableDefinition<ReviewId, Review> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, model)?;
                Ok(())
            },
            (DefinitionModelAssociatedTypes::TagModel(model), DefinitionModelAssociatedTypes::TagPrimaryKey(pk)) => {
                let table_def: TableDefinition<TagId, Tag> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, model)?;
                Ok(())
            },
            (DefinitionModelAssociatedTypes::ProductTagModel(model), DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk)) => {
                let table_def: TableDefinition<ProductTagId, ProductTag> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, model)?;
                Ok(())
            },
            _ => Err(NetabaseError::Other("Type mismatch in insert_model_into_redb".into())),
        }
    }

    fn insert_secondary_key_into_redb(
        &self,
        txn: &WriteTransaction,
        table_name: &str,
        primary_key_ref: &Self,
    ) -> NetabaseResult<()> {
        match (self, primary_key_ref) {
            (DefinitionModelAssociatedTypes::UserSecondaryKey(sk), DefinitionModelAssociatedTypes::UserPrimaryKey(pk)) => {
                let table_def: TableDefinition<UserSecondaryKeys, UserId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(sk, pk)?;
                Ok(())
            },
            (DefinitionModelAssociatedTypes::ProductSecondaryKey(sk), DefinitionModelAssociatedTypes::ProductPrimaryKey(pk)) => {
                let table_def: TableDefinition<ProductSecondaryKeys, ProductId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(sk, pk)?;
                Ok(())
            },
            (DefinitionModelAssociatedTypes::CategorySecondaryKey(sk), DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk)) => {
                let table_def: TableDefinition<CategorySecondaryKeys, CategoryId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(sk, pk)?;
                Ok(())
            },
            (DefinitionModelAssociatedTypes::ReviewSecondaryKey(sk), DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk)) => {
                let table_def: TableDefinition<ReviewSecondaryKeys, ReviewId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(sk, pk)?;
                Ok(())
            },
            (DefinitionModelAssociatedTypes::TagSecondaryKey(sk), DefinitionModelAssociatedTypes::TagPrimaryKey(pk)) => {
                let table_def: TableDefinition<TagSecondaryKeys, TagId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(sk, pk)?;
                Ok(())
            },
            (DefinitionModelAssociatedTypes::ProductTagSecondaryKey(sk), DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk)) => {
                let table_def: TableDefinition<ProductTagSecondaryKeys, ProductTagId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(sk, pk)?;
                Ok(())
            },
            _ => Err(NetabaseError::Other("Type mismatch in insert_secondary_key_into_redb".into())),
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
            (DefinitionModelAssociatedTypes::UserRelationalKey(rk), DefinitionModelAssociatedTypes::UserPrimaryKey(pk)) => {
                let table_def: TableDefinition<UserRelationalKeys, UserId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(rk, pk)?;
                Ok(())
            },
            // For Product relational keys: stores UserId -> ProductId mappings
            (DefinitionModelAssociatedTypes::ProductRelationalKey(rk), DefinitionModelAssociatedTypes::ProductPrimaryKey(pk)) => {
                let table_def: TableDefinition<ProductRelationalKeys, ProductId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(rk, pk)?;
                Ok(())
            },
            // For Category relational keys
            (DefinitionModelAssociatedTypes::CategoryRelationalKey(rk), DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk)) => {
                let table_def: TableDefinition<CategoryRelationalKeys, CategoryId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(rk, pk)?;
                Ok(())
            },
            // For Review relational keys: stores ProductId/UserId -> ReviewId mappings
            (DefinitionModelAssociatedTypes::ReviewRelationalKey(rk), DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk)) => {
                let table_def: TableDefinition<ReviewRelationalKeys, ReviewId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(rk, pk)?;
                Ok(())
            },
            // For Tag relational keys
            (DefinitionModelAssociatedTypes::TagRelationalKey(rk), DefinitionModelAssociatedTypes::TagPrimaryKey(pk)) => {
                let table_def: TableDefinition<TagRelationalKeys, TagId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(rk, pk)?;
                Ok(())
            },
            // For ProductTag relational keys: junction table mappings
            (DefinitionModelAssociatedTypes::ProductTagRelationalKey(rk), DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk)) => {
                let table_def: TableDefinition<ProductTagRelationalKeys, ProductTagId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(rk, pk)?;
                Ok(())
            },
            _ => Err(NetabaseError::Other("Type mismatch in insert_relational_key_into_redb".into())),
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
            },
            DefinitionModelAssociatedTypes::ProductPrimaryKey(pk) => {
                let table_def: TableDefinition<[u8; 32], ProductId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(hash, pk)?;
                Ok(())
            },
            DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk) => {
                let table_def: TableDefinition<[u8; 32], CategoryId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(hash, pk)?;
                Ok(())
            },
            DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk) => {
                let table_def: TableDefinition<[u8; 32], ReviewId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(hash, pk)?;
                Ok(())
            },
            DefinitionModelAssociatedTypes::TagPrimaryKey(pk) => {
                let table_def: TableDefinition<[u8; 32], TagId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(hash, pk)?;
                Ok(())
            },
            DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk) => {
                let table_def: TableDefinition<[u8; 32], ProductTagId> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(hash, pk)?;
                Ok(())
            },
            _ => Err(NetabaseError::Other("Type mismatch in insert_hash_into_redb".into())),
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
            },
            DefinitionModelAssociatedTypes::ProductPrimaryKey(pk) => {
                let table_def: TableDefinition<ProductId, Product> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            },
            DefinitionModelAssociatedTypes::CategoryPrimaryKey(pk) => {
                let table_def: TableDefinition<CategoryId, Category> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            },
            DefinitionModelAssociatedTypes::ReviewPrimaryKey(pk) => {
                let table_def: TableDefinition<ReviewId, Review> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            },
            DefinitionModelAssociatedTypes::TagPrimaryKey(pk) => {
                let table_def: TableDefinition<TagId, Tag> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            },
            DefinitionModelAssociatedTypes::ProductTagPrimaryKey(pk) => {
                let table_def: TableDefinition<ProductTagId, ProductTag> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.remove(pk)?;
                Ok(())
            },
            _ => Err(NetabaseError::Other("Type mismatch in delete_model_from_redb".into())),
        }
    }
}

#[derive(Debug, EnumDiscriminants)]
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
            DefinitionsDiscriminants::Product => vec![
                "Product_Title".to_string(),
                "Product_Score".to_string(),
            ],
            DefinitionsDiscriminants::Category => vec![
                "Category_Name".to_string(),
            ],
            DefinitionsDiscriminants::Review => vec![
                "Review_Rating".to_string(),
                "Review_CreatedAt".to_string(),
            ],
            DefinitionsDiscriminants::Tag => vec![
                "Tag_Name".to_string(),
            ],
            DefinitionsDiscriminants::ProductTag => vec![
                "ProductTag_ProductId".to_string(),
                "ProductTag_TagId".to_string(),
            ],
        }
    }

    fn get_relational_tree_names(model_discriminant: &DefinitionsDiscriminants) -> Vec<String> {
        match model_discriminant {
            DefinitionsDiscriminants::User => vec![
                "User_rel_CreatedProducts".to_string(),
            ],
            DefinitionsDiscriminants::Product => vec![
                "Product_rel_CreatedBy".to_string(),
            ],
            DefinitionsDiscriminants::Category => vec![
                "Category_rel_Products".to_string(),
            ],
            DefinitionsDiscriminants::Review => vec![
                "Review_rel_ReviewedProduct".to_string(),
                "Review_rel_Reviewer".to_string(),
            ],
            DefinitionsDiscriminants::Tag => vec![
                "Tag_rel_TaggedProducts".to_string(),
            ],
            DefinitionsDiscriminants::ProductTag => vec![
                "ProductTag_rel_Product".to_string(),
                "ProductTag_rel_Tag".to_string(),
            ],
        }
    }
}

impl NetabaseDefinitionKeyTrait<Definitions> for DefinitionKeys {
    fn inner<M: NetabaseModelTrait<Definitions>>(&self) -> M::Keys
    where
        Self: TryInto<M::Keys>,
        <Self as TryInto<M::Keys>>::Error: std::fmt::Debug,
    {
        self.clone().try_into().expect("Key variant does not match requested Model")
    }
}

// ================================================================================= ================
// Comprehensive Testing Functions
// ================================================================================= ================

fn create_test_data() -> (Vec<User>, Vec<Product>) {
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

    (users, products)
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
        let secondary_keys = <UserKeys as NetabaseModelKeyTrait<Definitions, User>>::secondary_keys(user);
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
        let secondary_keys = <ProductKeys as NetabaseModelKeyTrait<Definitions, Product>>::secondary_keys(product);
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
        let relational_keys = <UserKeys as NetabaseModelKeyTrait<Definitions, User>>::relational_keys(user);
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
        let relational_keys = <ProductKeys as NetabaseModelKeyTrait<Definitions, Product>>::relational_keys(product);
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
        let table_name = User::secondary_key_table_name(discriminant);
        println!("  Secondary [{}] : {}", discriminant.as_ref(), table_name);
    }

    for discriminant in UserRelationalKeysDiscriminants::iter() {
        let table_name = User::relational_key_table_name(discriminant);
        println!("  Relational [{}] : {}", discriminant.as_ref(), table_name);
    }

    println!("  Hash: {}", User::hash_tree_table_name());

    println!("Product table names:");
    for discriminant in ProductSecondaryKeysDiscriminants::iter() {
        let table_name = Product::secondary_key_table_name(discriminant);
        println!("  Secondary [{}] : {}", discriminant.as_ref(), table_name);
    }

    for discriminant in ProductRelationalKeysDiscriminants::iter() {
        let table_name = Product::relational_key_table_name(discriminant);
        println!("  Relational [{}] : {}", discriminant.as_ref(), table_name);
    }

    println!("  Hash: {}", Product::hash_tree_table_name());
}

fn test_tree_access_enums() {
    println!("\n=== Testing Tree Access Enums (No Inner Types) ===");

    println!("User Secondary Tree Names (Copy, lightweight):");
    for tree_name in UserSecondaryTreeNames::iter() {
        println!("  Tree: {} (name: {})", format!("{:?}", tree_name), tree_name.as_ref());
    }

    println!("User Relational Tree Names:");
    for tree_name in UserRelationalTreeNames::iter() {
        println!("  Tree: {} (name: {})", format!("{:?}", tree_name), tree_name.as_ref());
    }

    println!("Product Secondary Tree Names:");
    for tree_name in ProductSecondaryTreeNames::iter() {
        println!("  Tree: {} (name: {})", format!("{:?}", tree_name), tree_name.as_ref());
    }

    println!("Product Relational Tree Names:");
    for tree_name in ProductRelationalTreeNames::iter() {
        println!("  Tree: {} (name: {})", format!("{:?}", tree_name), tree_name.as_ref());
    }

    // Demonstrate Copy trait
    let tree = UserSecondaryTreeNames::Email;
    let tree_copy = tree; // This is a copy, not a move!
    println!("\nDemonstrating Copy trait:");
    println!("  Original: {:?}, Copy: {:?}", tree, tree_copy);
    println!("  Both can still be used: {} == {}", tree.as_ref(), tree_copy.as_ref());
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
        let secondary_keys = <UserKeys as NetabaseModelKeyTrait<Definitions, User>>::secondary_keys(user);
        for sk in secondary_keys {
            let serialized: Vec<u8> = sk.clone().try_into().expect("Serialization failed");
            let deserialized = UserSecondaryKeys::try_from(serialized).expect("Deserialization failed");
            println!("User secondary key roundtrip successful: {:?} -> {:?}", sk, deserialized);
        }
    }
}

fn test_real_database_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing Real Database Operations ===");
    
    // Create a temporary database
    let db_path = "/tmp/boilerplate_test.db";
    if Path::new(db_path).exists() {
        std::fs::remove_file(db_path)?;
    }
    
    let store = RedbStore::<Definitions>::new(db_path)?;
    let (users, products) = create_test_data();
    
    println!("Storing users in database...");
    for user in &users {
        store.put_one(user.clone())?;
        println!("  Stored user: {}", user.name);
    }
    
    println!("Storing products in database...");
    for product in &products {
        store.put_one(product.clone())?;
        println!("  Stored product: {}", product.title);
    }
    
    println!("Retrieving users by primary key...");
    for user in &users {
        let retrieved = store.get_one::<User>(user.primary_key())?;
        match retrieved {
            Some(u) => {
                assert_eq!(u.id, user.id);
                assert_eq!(u.name, user.name);
                println!("  Retrieved user: {} (ID: {})", u.name, u.id);
            },
            None => println!("  User not found: {}", user.name),
        }
    }
    
    println!("Retrieving products by primary key...");
    for product in &products {
        let retrieved = store.get_one::<Product>(product.primary_key())?;
        match retrieved {
            Some(p) => {
                assert_eq!(p.uuid, product.uuid);
                assert_eq!(p.title, product.title);
                println!("  Retrieved product: {} (ID: {})", p.title, p.uuid);
            },
            None => println!("  Product not found: {}", product.title),
        }
    }
    
    // Test batch operations
    println!("Testing batch operations...");
    let user_pks: Vec<_> = users.iter().map(|u| u.primary_key()).collect();
    let retrieved_users = store.get_many::<User>(user_pks)?;
    println!("  Batch retrieved {} users", retrieved_users.iter().filter_map(|u: &Option<User>| u.as_ref()).count());
    
    // Clean up
    std::fs::remove_file(db_path)?;
    println!("Database cleaned up");
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Comprehensive Boilerplate Example for Netabase Store");
    println!("=====================================================");
    
    let (users, products) = create_test_data();
    
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
    test_real_database_operations()?;
    
    println!("\n✅ All tests completed successfully!");
    println!("\nThis example demonstrates:");
    println!("• Primary key access and storage");
    println!("• Secondary key enumeration and indexing");
    println!("• Relational key relationships");
    println!("• Hash computation for data integrity");
    println!("• Discriminant enumeration for type safety");
    println!("• Tree and table name generation");
    println!("• Tree access enums (Copy, no inner types) for efficient tree identification");
    println!("• Serialization/deserialization roundtrips");
    println!("• Real database storage and retrieval operations");
    println!("• Batch operations for performance");
    
    Ok(())
}
