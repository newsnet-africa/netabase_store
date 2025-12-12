use bincode::{Decode, Encode};
use derive_more::TryInto;
use netabase_store::{
    TreeType,
    databases::{
        redb_store::{RedbNetabaseModelTrait, RedbStore},
        sled_store::{SledNetabaseModelTrait, SledStore, SledStoreTrait},
    },
    error::{NetabaseError, NetabaseResult},
    traits::{
        definition::{DiscriminantName, NetabaseDefinition, key::NetabaseDefinitionKeyTrait},
        model::{ModelTypeContainer, NetabaseModelTrait, key::NetabaseModelKeyTrait},
        store::{store::StoreTrait, tree_manager::{TreeManager, ModelTreeManager, StandardModelTreeName}},
    },
};
use redb::{Key, TableDefinition, TypeName, Value, WriteTransaction};
use strum::IntoDiscriminant;
use strum::{AsRefStr, EnumDiscriminants, EnumIter};

// =================================================================================
// CORE TYPE SYSTEM - Strict Nested Associated Types
// =================================================================================

/// Strict permission levels with compile-time safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, AsRefStr, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum PermissionLevel {
    None,
    Read,
    Write,
    ReadWrite,
    Admin,
}

impl Default for PermissionLevel {
    fn default() -> Self {
        Self::None
    }
}

impl PermissionLevel {
    pub const fn can_read(&self) -> bool {
        matches!(self, Self::Read | Self::ReadWrite | Self::Admin)
    }

    pub const fn can_write(&self) -> bool {
        matches!(self, Self::Write | Self::ReadWrite | Self::Admin)
    }

    pub const fn can_manage_permissions(&self) -> bool {
        matches!(self, Self::Admin)
    }
}

// =================================================================================
// USER MODEL - Complete Strict Redesign
// =================================================================================

/// Primary User model with strict typing
#[derive(Debug, Clone, Encode, Decode, Default)]
pub struct User {
    pub id: UserId,
    pub email: UserEmail,
    pub name: UserName,
    pub age: UserAge,
    pub created_products: Vec<ProductId>,
}

impl ModelTypeContainer for User {
    type PrimaryKey = UserId;
    type SecondaryKeys = UserSecondaryKeys;
    type RelationalKeys = UserRelationalKeys;
    type Subscriptions = UserSubscriptions;
    type TreeName = StandardModelTreeName<StrictDefinitions, User>;

    fn primary_tree_name() -> Self::TreeName {
        StandardModelTreeName::Main
    }
}

// Strongly typed identifiers with strum derive macros
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct UserId(pub u64);

impl Default for UserId {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct UserEmail(pub String);

impl Default for UserEmail {
    fn default() -> Self {
        Self(String::new())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct UserName(pub String);

impl Default for UserName {
    fn default() -> Self {
        Self(String::new())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct UserAge(pub u32);

impl Default for UserAge {
    fn default() -> Self {
        Self(0)
    }
}



/// User secondary keys - nested and type-safe with strum derive macros
#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(UserSecondaryTreeNames))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum UserSecondaryKeys {
    Email(UserEmail),
    Name(UserName),
    Age(UserAge),
}

impl Default for UserSecondaryTreeNames {
    fn default() -> Self {
        Self::Email
    }
}

impl DiscriminantName for UserSecondaryTreeNames {}

/// User relational keys - type-safe references to other models
#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(UserRelationalTreeNames))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum UserRelationalKeys {
    CreatedProducts(ProductId),
}

impl Default for UserRelationalTreeNames {
    fn default() -> Self {
        Self::CreatedProducts
    }
}

impl DiscriminantName for UserRelationalTreeNames {}

/// User subscriptions - type-safe subscription management
#[derive(Debug, Clone, Copy, AsRefStr, EnumIter, Encode, Decode)]
#[strum(serialize_all = "snake_case")]
#[derive(PartialEq)]
pub enum UserSubscriptions {
    Updates,
}

impl Default for UserSubscriptions {
    fn default() -> Self {
        Self::Updates
    }
}

impl DiscriminantName for UserSubscriptions {}

// =================================================================================
// PRODUCT MODEL - Complete Strict Redesign
// =================================================================================

#[derive(Debug, Clone, Encode, Decode, Default)]
pub struct Product {
    pub id: ProductId,
    pub title: ProductTitle,
    pub score: ProductScore,
    pub created_by: UserId,
    pub category_id: CategoryId,
}

impl NetabaseModelTrait<StrictDefinitions> for Product {
    type Keys = ProductKeys;

    const MODEL_TREE_NAME: <StrictDefinitions as strum::IntoDiscriminant>::Discriminant =
        StrictModelTreeNames::Product;

    type Hash = [u8; 32];

    fn primary_key(&self) -> Self::PrimaryKey {
        todo!()
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        todo!()
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        todo!()
    }

    fn get_subscriptions(&self) -> Vec<Self::Subscriptions> {
        todo!()
    }

    fn compute_hash(&self) -> Self::Hash {
        todo!()
    }
}

impl ModelTypeContainer for Product {
    type PrimaryKey = ProductId;
    type SecondaryKeys = ProductSecondaryKeys;
    type RelationalKeys = ProductRelationalKeys;
    type Subscriptions = ProductSubscriptions;
    type TreeName = StandardModelTreeName<StrictDefinitions, Product>;

    fn primary_tree_name() -> Self::TreeName {
        StandardModelTreeName::Main
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ProductId(pub u64);

impl Default for ProductId {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, Default)]
pub struct ProductTitle(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, Default)]
pub struct ProductScore(pub u32);



#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(ProductSecondaryTreeNames))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum ProductSecondaryKeys {
    Title(ProductTitle),
    Score(ProductScore),
}

impl DiscriminantName for ProductSecondaryTreeNames {}

#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(ProductRelationalKeyType))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum ProductRelationalKeys {
    CreatedBy(UserId),
    Category(CategoryId),
}

impl DiscriminantName for ProductRelationalKeyType {}

#[derive(Debug, Clone, Copy, AsRefStr, EnumIter, Encode, Decode)]
#[strum(serialize_all = "snake_case")]
#[derive(PartialEq)]
pub enum ProductSubscriptions {
    Updates,
}

impl Default for ProductSubscriptions {
    fn default() -> Self {
        Self::Updates
    }
}

impl DiscriminantName for ProductSubscriptions {}

// =================================================================================
// CATEGORY MODEL - Complete Strict Redesign
// =================================================================================

#[derive(Debug, Clone, Encode, Decode, Default)]
pub struct Category {
    pub id: CategoryId,
    pub name: CategoryName,
    pub description: CategoryDescription,
}

impl ModelTypeContainer for Category {
    type PrimaryKey = CategoryId;
    type SecondaryKeys = CategorySecondaryKeys;
    type RelationalKeys = CategoryRelationalKeys;
    type Subscriptions = CategorySubscriptions;
    type TreeName = StandardModelTreeName<StrictDefinitions, Category>;

    fn primary_tree_name() -> Self::TreeName {
        StandardModelTreeName::Main
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct CategoryId(pub u64);

impl Default for CategoryId {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct CategoryName(pub String);

impl Default for CategoryName {
    fn default() -> Self {
        Self(String::new())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct CategoryDescription(pub String);

impl Default for CategoryDescription {
    fn default() -> Self {
        Self(String::new())
    }
}



#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(CategorySecondaryTreeNames))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum CategorySecondaryKeys {
    Name(CategoryName),
}

impl DiscriminantName for CategorySecondaryTreeNames {}

#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(CategoryRelationalKeyType))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum CategoryRelationalKeys {
    Products(ProductId),
}

impl DiscriminantName for CategoryRelationalKeyType {}

#[derive(Debug, Clone, Copy, AsRefStr, EnumIter, Encode, Decode)]
#[strum(serialize_all = "snake_case")]
#[derive(PartialEq)]
pub enum CategorySubscriptions {
    AllProducts,
}

impl Default for CategorySubscriptions {
    fn default() -> Self {
        Self::AllProducts
    }
}

impl DiscriminantName for CategorySubscriptions {}

// =================================================================================
// REVIEW MODEL - Complete Strict Redesign
// =================================================================================

#[derive(Debug, Clone, Encode, Decode, Default)]
pub struct Review {
    pub id: ReviewId,
    pub product_id: ProductId,
    pub user_id: UserId,
    pub rating: ReviewRating,
    pub comment: ReviewComment,
    pub created_at: ReviewTimestamp,
}

impl ModelTypeContainer for Review {
    type PrimaryKey = ReviewId;
    type SecondaryKeys = ReviewSecondaryKeys;
    type RelationalKeys = ReviewRelationalKeys;
    type Subscriptions = ReviewSubscriptions;
    type TreeName = StandardModelTreeName<StrictDefinitions, Review>;

    fn primary_tree_name() -> Self::TreeName {
        StandardModelTreeName::Main
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ReviewId(pub u64);

impl Default for ReviewId {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ReviewRating(pub u8);

impl Default for ReviewRating {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct ReviewComment(pub String);

impl Default for ReviewComment {
    fn default() -> Self {
        Self(String::new())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ReviewTimestamp(pub u64);

impl Default for ReviewTimestamp {
    fn default() -> Self {
        Self(0)
    }
}



#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(ReviewSecondaryTreeNames))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum ReviewSecondaryKeys {
    Rating(ReviewRating),
    CreatedAt(ReviewTimestamp),
}

impl DiscriminantName for ReviewSecondaryTreeNames {}

#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(ReviewRelationalKeyType))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum ReviewRelationalKeys {
    ReviewedProduct(ProductId),
    Reviewer(UserId),
}

impl DiscriminantName for ReviewRelationalKeyType {}

#[derive(Debug, Clone, Copy, AsRefStr, EnumIter, Encode, Decode)]
#[strum(serialize_all = "snake_case")]
pub enum ReviewSubscriptions {
    ReviewsForProduct,
}

impl Default for ReviewSubscriptions {
    fn default() -> Self {
        Self::ReviewsForProduct
    }
}

impl DiscriminantName for ReviewSubscriptions {}

// =================================================================================
// TAG MODEL - Complete Strict Redesign
// =================================================================================

#[derive(Debug, Clone, Encode, Decode, Default)]
pub struct Tag {
    pub id: TagId,
    pub name: TagName,
}

impl ModelTypeContainer for Tag {
    type PrimaryKey = TagId;
    type SecondaryKeys = TagSecondaryKeys;
    type RelationalKeys = TagRelationalKeys;
    type Subscriptions = TagSubscriptions;
    type TreeName = StandardModelTreeName<StrictDefinitions, Tag>;

    fn primary_tree_name() -> Self::TreeName {
        StandardModelTreeName::Main
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct TagId(pub u64);

impl Default for TagId {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct TagName(pub String);

impl Default for TagName {
    fn default() -> Self {
        Self(String::new())
    }
}



#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(TagSecondaryTreeNames))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum TagSecondaryKeys {
    Name(TagName),
}

impl DiscriminantName for TagSecondaryTreeNames {}

#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(TagRelationalKeyType))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum TagRelationalKeys {
    TaggedProducts(ProductId),
}

impl DiscriminantName for TagRelationalKeyType {}

#[derive(Debug, Clone, Copy, AsRefStr, EnumIter, Encode, Decode)]
#[strum(serialize_all = "snake_case")]
#[derive(PartialEq)]
pub enum TagSubscriptions {
    TaggedItems,
}

impl Default for TagSubscriptions {
    fn default() -> Self {
        Self::TaggedItems
    }
}

impl DiscriminantName for TagSubscriptions {}

// =================================================================================
// PRODUCT-TAG JUNCTION MODEL - Complete Strict Redesign
// =================================================================================

#[derive(Debug, Clone, Encode, Decode, Default)]
pub struct ProductTag {
    pub product_id: ProductId,
    pub tag_id: TagId,
}

impl ModelTypeContainer for ProductTag {
    type PrimaryKey = ProductTagId;
    type SecondaryKeys = ProductTagSecondaryKeys;
    type RelationalKeys = ProductTagRelationalKeys;
    type Subscriptions = ProductTagSubscriptions;
    type TreeName = StandardModelTreeName<StrictDefinitions, ProductTag>;

    fn primary_tree_name() -> Self::TreeName {
        StandardModelTreeName::Main
    }
}

/// Composite primary key for product-tag relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ProductTagId {
    pub product_id: ProductId,
    pub tag_id: TagId,
}



#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(ProductTagSecondaryTreeNames))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum ProductTagSecondaryKeys {
    ProductId(ProductId),
    TagId(TagId),
}

impl DiscriminantName for ProductTagSecondaryTreeNames {}

#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(ProductTagRelationalKeyType))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum ProductTagRelationalKeys {
    Product(ProductId),
    Tag(TagId),
}

impl DiscriminantName for ProductTagRelationalKeyType {}

#[derive(Debug, Clone, Copy, AsRefStr, EnumIter, Encode, Decode)]
#[strum(serialize_all = "snake_case")]
#[derive(PartialEq)]
pub enum ProductTagSubscriptions {
    ProductTags,
}

impl Default for ProductTagSubscriptions {
    fn default() -> Self {
        Self::ProductTags
    }
}

impl DiscriminantName for ProductTagSubscriptions {}

// =================================================================================
// STRICT REDB KEY/VALUE TRAIT IMPLEMENTATIONS
// =================================================================================

/// Macro for implementing strict redb traits with complete type safety
macro_rules! impl_redb_strict {
    ($wrapper:ty, $inner:ty, $name:literal) => {
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

        impl TryFrom<Vec<u8>> for $wrapper {
            type Error = bincode::error::DecodeError;

            fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
                let (value, _): ($inner, usize) =
                    bincode::decode_from_slice(&data, bincode::config::standard())?;
                Ok(Self(value))
            }
        }

        impl TryFrom<$wrapper> for Vec<u8> {
            type Error = bincode::error::EncodeError;

            fn try_from(value: $wrapper) -> Result<Self, Self::Error> {
                bincode::encode_to_vec(value.0, bincode::config::standard())
            }
        }
    };
}

// Apply strict redb implementations to all primary key types
impl_redb_strict!(UserId, u64, "UserId");
impl_redb_strict!(UserAge, u32, "UserAge");
impl_redb_strict!(ProductId, u64, "ProductId");
impl_redb_strict!(ProductScore, u32, "ProductScore");
impl_redb_strict!(CategoryId, u64, "CategoryId");
impl_redb_strict!(ReviewId, u64, "ReviewId");
impl_redb_strict!(ReviewRating, u8, "ReviewRating");
impl_redb_strict!(ReviewTimestamp, u64, "ReviewTimestamp");
impl_redb_strict!(TagId, u64, "TagId");

// Special implementation for string-based types
macro_rules! impl_redb_string_strict {
    ($wrapper:ty, $name:literal) => {
        impl Value for $wrapper {
            type SelfType<'a> = $wrapper;
            type AsBytes<'a> = <String as Value>::AsBytes<'a>;

            fn fixed_width() -> Option<usize> {
                None
            }

            fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
            where
                Self: 'a,
            {
                Self(<String as Value>::from_bytes(data))
            }

            fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
                <String as Value>::as_bytes(&value.0)
            }

            fn type_name() -> TypeName {
                TypeName::new($name)
            }
        }

        impl Key for $wrapper {
            fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                <String as Key>::compare(data1, data2)
            }
        }

        impl TryFrom<Vec<u8>> for $wrapper {
            type Error = bincode::error::DecodeError;

            fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
                let (value, _): (String, usize) =
                    bincode::decode_from_slice(&data, bincode::config::standard())?;
                Ok(Self(value))
            }
        }

        impl TryFrom<$wrapper> for Vec<u8> {
            type Error = bincode::error::EncodeError;

            fn try_from(value: $wrapper) -> Result<Self, Self::Error> {
                bincode::encode_to_vec(value.0, bincode::config::standard())
            }
        }
    };
}

impl_redb_string_strict!(UserEmail, "UserEmail");
impl_redb_string_strict!(UserName, "UserName");
impl_redb_string_strict!(ProductTitle, "ProductTitle");
impl_redb_string_strict!(CategoryName, "CategoryName");
impl_redb_string_strict!(CategoryDescription, "CategoryDescription");
impl_redb_string_strict!(ReviewComment, "ReviewComment");
impl_redb_string_strict!(TagName, "TagName");

// Composite key implementation for ProductTagId
impl Value for ProductTagId {
    type SelfType<'a> = ProductTagId;
    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        Some(16) // 2 * u64
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let product_id = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let tag_id = u64::from_le_bytes(data[8..16].try_into().unwrap());
        ProductTagId {
            product_id: ProductId(product_id),
            tag_id: TagId(tag_id),
        }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        let mut bytes = Vec::with_capacity(16);
        bytes.extend_from_slice(&value.product_id.0.to_le_bytes());
        bytes.extend_from_slice(&value.tag_id.0.to_le_bytes());
        std::borrow::Cow::Owned(bytes)
    }

    fn type_name() -> TypeName {
        TypeName::new("ProductTagId")
    }
}

impl Key for ProductTagId {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl TryFrom<Vec<u8>> for ProductTagId {
    type Error = bincode::error::DecodeError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (product_id, tag_id): (u64, u64) =
            bincode::decode_from_slice(&data, bincode::config::standard())?.0;
        Ok(ProductTagId {
            product_id: ProductId(product_id),
            tag_id: TagId(tag_id),
        })
    }
}

impl TryFrom<ProductTagId> for Vec<u8> {
    type Error = bincode::error::EncodeError;

    fn try_from(value: ProductTagId) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(
            (value.product_id.0, value.tag_id.0),
            bincode::config::standard(),
        )
    }
}

// =================================================================================
// STRICT ENUM VALUE/KEY IMPLEMENTATIONS
// =================================================================================

/// Macro for implementing strict enum serialization with automatic discriminant handling
macro_rules! impl_enum_redb_strict {
    ($enum_type:ty, $name:literal) => {
        impl Value for $enum_type {
            type SelfType<'a> = $enum_type;
            type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

            fn fixed_width() -> Option<usize> {
                None
            }

            fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
            where
                Self: 'a,
            {
                Self::try_from(data.to_vec()).expect(concat!("Failed to deserialize ", $name))
            }

            fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
                let bytes: Vec<u8> = value
                    .clone()
                    .try_into()
                    .expect(concat!("Failed to serialize ", $name));
                std::borrow::Cow::Owned(bytes)
            }

            fn type_name() -> TypeName {
                TypeName::new($name)
            }
        }

        impl Key for $enum_type {
            fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                data1.cmp(data2)
            }
        }

        impl TryFrom<Vec<u8>> for $enum_type {
            type Error = bincode::error::DecodeError;

            fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
                let (value, _): ($enum_type, usize) =
                    bincode::decode_from_slice(&data, bincode::config::standard())?;
                Ok(value)
            }
        }

        impl TryFrom<$enum_type> for Vec<u8> {
            type Error = bincode::error::EncodeError;

            fn try_from(value: $enum_type) -> Result<Self, Self::Error> {
                bincode::encode_to_vec(value, bincode::config::standard())
            }
        }
    };
}

// Apply enum implementations to all secondary/relational key types
impl_enum_redb_strict!(UserSecondaryKeys, "UserSecondaryKeys");
impl_enum_redb_strict!(UserRelationalKeys, "UserRelationalKeys");
impl_enum_redb_strict!(UserSubscriptions, "UserSubscriptions");
impl_enum_redb_strict!(ProductSecondaryKeys, "ProductSecondaryKeys");
impl_enum_redb_strict!(ProductRelationalKeys, "ProductRelationalKeys");
impl_enum_redb_strict!(ProductSubscriptions, "ProductSubscriptions");
impl_enum_redb_strict!(CategorySecondaryKeys, "CategorySecondaryKeys");
impl_enum_redb_strict!(CategoryRelationalKeys, "CategoryRelationalKeys");
impl_enum_redb_strict!(CategorySubscriptions, "CategorySubscriptions");
impl_enum_redb_strict!(ReviewSecondaryKeys, "ReviewSecondaryKeys");
impl_enum_redb_strict!(ReviewRelationalKeys, "ReviewRelationalKeys");
impl_enum_redb_strict!(ReviewSubscriptions, "ReviewSubscriptions");
impl_enum_redb_strict!(TagSecondaryKeys, "TagSecondaryKeys");
impl_enum_redb_strict!(TagRelationalKeys, "TagRelationalKeys");
impl_enum_redb_strict!(TagSubscriptions, "TagSubscriptions");
impl_enum_redb_strict!(ProductTagSecondaryKeys, "ProductTagSecondaryKeys");
impl_enum_redb_strict!(ProductTagRelationalKeys, "ProductTagRelationalKeys");
impl_enum_redb_strict!(ProductTagSubscriptions, "ProductTagSubscriptions");

// =================================================================================
// STRICT NESTED KEY TRAIT IMPLEMENTATIONS
// =================================================================================

/// User's nested key trait implementation with strict type safety
#[derive(Debug, Clone)]
pub enum UserKeys {
    Primary(UserId),
    Secondary(UserSecondaryKeys),
    Relational(UserRelationalKeys),
}

impl NetabaseModelKeyTrait<StrictDefinitions, User> for UserKeys {
    type PrimaryKey = UserId;
    type SecondaryEnum = UserSecondaryKeys;
    type RelationalEnum = UserRelationalKeys;

    fn secondary_keys(model: &User) -> Vec<Self::SecondaryEnum> {
        vec![
            UserSecondaryKeys::Email(model.email.clone()),
            UserSecondaryKeys::Name(model.name.clone()),
            UserSecondaryKeys::Age(model.age),
        ]
    }

    fn relational_keys(model: &User) -> Vec<Self::RelationalEnum> {
        model
            .created_products
            .iter()
            .map(|&product_id| UserRelationalKeys::CreatedProducts(product_id))
            .collect()
    }

    fn as_self_secondary(s: Self::SecondaryEnum) -> Self {
        Self::Secondary(s)
    }

    fn as_self_relational(s: Self::RelationalEnum) -> Self {
        Self::Relational(s)
    }

    fn as_self_primary(s: Self::PrimaryKey) -> Self {
        Self::Primary(s)
    }
}

#[derive(Debug, Clone)]
pub enum ProductKeys {
    Primary(ProductId),
    Secondary(ProductSecondaryKeys),
    Relational(ProductRelationalKeys),
}

impl NetabaseModelKeyTrait<StrictDefinitions, Product> for ProductKeys {
    type PrimaryKey = ProductId;
    type SecondaryEnum = ProductSecondaryKeys;
    type RelationalEnum = ProductRelationalKeys;

    fn secondary_keys(model: &Product) -> Vec<Self::SecondaryEnum> {
        vec![
            ProductSecondaryKeys::Title(model.title.clone()),
            ProductSecondaryKeys::Score(model.score),
        ]
    }

    fn relational_keys(model: &Product) -> Vec<Self::RelationalEnum> {
        vec![
            ProductRelationalKeys::CreatedBy(model.created_by),
            ProductRelationalKeys::Category(model.category_id),
        ]
    }

    fn as_self_secondary(s: Self::SecondaryEnum) -> Self {
        Self::Secondary(s)
    }

    fn as_self_relational(s: Self::RelationalEnum) -> Self {
        Self::Relational(s)
    }

    fn as_self_primary(s: Self::PrimaryKey) -> Self {
        Self::Primary(s)
    }
}

#[derive(Debug, Clone)]
pub enum CategoryKeys {
    Primary(CategoryId),
    Secondary(CategorySecondaryKeys),
    Relational(ReviewRelationalKeys),
}

impl NetabaseModelTrait<StrictDefinitions> for Category {
    type Keys = CategoryKeys;

    const MODEL_TREE_NAME: <StrictDefinitions as strum::IntoDiscriminant>::Discriminant =
        StrictModelTreeNames::Category;

    type Hash = [u8; 32];

    fn primary_key(&self) -> Self::PrimaryKey {
        todo!()
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        todo!()
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        todo!()
    }

    fn get_subscriptions(&self) -> Vec<Self::Subscriptions> {
        todo!()
    }

    fn compute_hash(&self) -> Self::Hash {
        todo!()
    }
}
impl NetabaseModelKeyTrait<StrictDefinitions, Category> for CategoryKeys {
    type PrimaryKey = CategoryId;

    type SecondaryEnum = CategorySecondaryKeys;

    type RelationalEnum = CategoryRelationalKeys;

    fn as_self_secondary(s: Self::SecondaryEnum) -> Self {
        todo!()
    }

    fn as_self_relational(s: Self::RelationalEnum) -> Self {
        todo!()
    }

    fn as_self_primary(s: Self::PrimaryKey) -> Self {
        todo!()
    }

    fn secondary_keys(model: &Category) -> Vec<Self::SecondaryEnum> {
        todo!()
    }

    fn relational_keys(model: &Category) -> Vec<Self::RelationalEnum> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum ReviewKeys {
    Primary(ReviewId),
    Secondary(ReviewSecondaryKeys),
    Relational(ReviewRelationalKeys),
}

impl NetabaseModelTrait<StrictDefinitions> for Review {
    type Keys = ReviewKeys;

    const MODEL_TREE_NAME: <StrictDefinitions as strum::IntoDiscriminant>::Discriminant =
        StrictModelTreeNames::Review;

    type Hash = [u8; 32];

    fn primary_key(&self) -> Self::PrimaryKey {
        todo!()
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        todo!()
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        todo!()
    }

    fn get_subscriptions(&self) -> Vec<Self::Subscriptions> {
        todo!()
    }

    fn compute_hash(&self) -> Self::Hash {
        todo!()
    }
}

impl NetabaseModelKeyTrait<StrictDefinitions, Review> for ReviewKeys {
    type PrimaryKey = ReviewId;
    type SecondaryEnum = ReviewSecondaryKeys;
    type RelationalEnum = ReviewRelationalKeys;

    fn secondary_keys(model: &Review) -> Vec<Self::SecondaryEnum> {
        vec![
            ReviewSecondaryKeys::Rating(model.rating),
            ReviewSecondaryKeys::CreatedAt(model.created_at),
        ]
    }

    fn relational_keys(model: &Review) -> Vec<Self::RelationalEnum> {
        vec![
            ReviewRelationalKeys::ReviewedProduct(model.product_id),
            ReviewRelationalKeys::Reviewer(model.user_id),
        ]
    }

    fn as_self_secondary(s: Self::SecondaryEnum) -> Self {
        Self::Secondary(s)
    }

    fn as_self_relational(s: Self::RelationalEnum) -> Self {
        Self::Relational(s)
    }

    fn as_self_primary(s: Self::PrimaryKey) -> Self {
        Self::Primary(s)
    }
}

#[derive(Debug, Clone)]
pub enum TagKeys {
    Primary(TagId),
    Secondary(TagSecondaryKeys),
    Relational(TagRelationalKeys),
}
impl NetabaseModelTrait<StrictDefinitions> for Tag {
    type Keys = TagKeys;

    const MODEL_TREE_NAME: <StrictDefinitions as strum::IntoDiscriminant>::Discriminant =
        StrictModelTreeNames::Tag;

    type Hash = [u8; 32];

    fn primary_key(&self) -> Self::PrimaryKey {
        todo!()
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        todo!()
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        todo!()
    }

    fn get_subscriptions(&self) -> Vec<Self::Subscriptions> {
        todo!()
    }

    fn compute_hash(&self) -> Self::Hash {
        todo!()
    }
}

impl NetabaseModelKeyTrait<StrictDefinitions, Tag> for TagKeys {
    type PrimaryKey = TagId;
    type SecondaryEnum = TagSecondaryKeys;
    type RelationalEnum = TagRelationalKeys;

    fn secondary_keys(model: &Tag) -> Vec<Self::SecondaryEnum> {
        vec![TagSecondaryKeys::Name(model.name.clone())]
    }

    fn relational_keys(_model: &Tag) -> Vec<Self::RelationalEnum> {
        // Tag relationships are handled through ProductTag junction table
        vec![]
    }

    fn as_self_secondary(s: Self::SecondaryEnum) -> Self {
        todo!()
    }

    fn as_self_relational(s: Self::RelationalEnum) -> Self {
        todo!()
    }

    fn as_self_primary(s: Self::PrimaryKey) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum ProductTagKeys {
    Primary(ProductId),
    Secondary(ProductSecondaryKeys),
    Relational(ProductRelationalKeys),
}

impl NetabaseModelTrait<StrictDefinitions> for ProductTag {
    type Keys = ProductTagKeys;

    const MODEL_TREE_NAME: <StrictDefinitions as strum::IntoDiscriminant>::Discriminant =
        StrictModelTreeNames::Product;

    type Hash = [u8; 32];

    fn primary_key(&self) -> Self::PrimaryKey {
        todo!()
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        todo!()
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        todo!()
    }

    fn get_subscriptions(&self) -> Vec<Self::Subscriptions> {
        todo!()
    }

    fn compute_hash(&self) -> Self::Hash {
        todo!()
    }
}

impl NetabaseModelKeyTrait<StrictDefinitions, ProductTag> for ProductTagKeys {
    type PrimaryKey = ProductTagId;
    type SecondaryEnum = ProductTagSecondaryKeys;
    type RelationalEnum = ProductTagRelationalKeys;

    fn secondary_keys(model: &ProductTag) -> Vec<Self::SecondaryEnum> {
        vec![
            ProductTagSecondaryKeys::ProductId(model.product_id),
            ProductTagSecondaryKeys::TagId(model.tag_id),
        ]
    }

    fn relational_keys(model: &ProductTag) -> Vec<Self::RelationalEnum> {
        vec![
            ProductTagRelationalKeys::Product(model.product_id),
            ProductTagRelationalKeys::Tag(model.tag_id),
        ]
    }

    fn as_self_secondary(s: Self::SecondaryEnum) -> Self {
        todo!()
    }

    fn as_self_relational(s: Self::RelationalEnum) -> Self {
        todo!()
    }

    fn as_self_primary(s: Self::PrimaryKey) -> Self {
        todo!()
    }
}

// =================================================================================
// STRICT MODEL VALUE IMPLEMENTATIONS
// =================================================================================

/// Macro for implementing model Value traits with strict bincode serialization
macro_rules! impl_model_value_strict {
    ($model_type:ty, $name:literal) => {
        impl Value for $model_type {
            type SelfType<'a> = $model_type;
            type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

            fn fixed_width() -> Option<usize> {
                None
            }

            fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
            where
                Self: 'a,
            {
                let (value, _): ($model_type, usize) =
                    bincode::decode_from_slice(&data, bincode::config::standard())
                        .expect(concat!("Failed to deserialize ", $name));
                value
            }

            fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
                let bytes = bincode::encode_to_vec(value, bincode::config::standard())
                    .expect(concat!("Failed to serialize ", $name));
                std::borrow::Cow::Owned(bytes)
            }

            fn type_name() -> TypeName {
                TypeName::new($name)
            }
        }
    };
}

// Apply model Value implementations
impl_model_value_strict!(User, "User");
impl_model_value_strict!(Product, "Product");
impl_model_value_strict!(Category, "Category");
impl_model_value_strict!(Review, "Review");
impl_model_value_strict!(Tag, "Tag");
impl_model_value_strict!(ProductTag, "ProductTag");

// =================================================================================
// STRICT NETABASE MODEL TRAIT IMPLEMENTATIONS
// =================================================================================

/// Create blake3 hash for models
fn compute_model_hash<T: Encode>(model: &T) -> [u8; 32] {
    let bytes = bincode::encode_to_vec(model, bincode::config::standard())
        .expect("Failed to serialize model for hashing");
    blake3::hash(&bytes).into()
}

/// Simple macro for implementing NetabaseModelTrait
/// Requires explicit parameter passing but much simpler than the original
macro_rules! generate_model_trait {
    ($model:ident, $keys:ident, $subscriptions:ident, $extractor_prefix:ident) => {
        impl NetabaseModelTrait<StrictDefinitions> for $model {
            type Keys = $keys;
            type Hash = [u8; 32];

            const MODEL_TREE_NAME: <StrictDefinitions as IntoDiscriminant>::Discriminant =
                StrictModelTreeNames::$model;

            fn primary_key(&self) -> Self::PrimaryKey {
                self.id
            }

            fn compute_hash(&self) -> Self::Hash {
                compute_model_hash(self)
            }

            fn get_subscriptions(&self) -> Vec<Self::Subscriptions> {
                vec![$subscriptions::Updates]
            }

            fn get_secondary_keys(&self) -> Self::SecondaryKeys {
                paste::paste! { [<extract_ $extractor_prefix:snake _secondary_keys>](self) }
            }

            fn get_relational_keys(&self) -> Self::RelationalKeys {
                paste::paste! { [<extract_ $extractor_prefix:snake _relational_keys>](self) }
            }
        }
    };
}

/// Special macro for ProductTag with composite primary key
macro_rules! impl_netabase_model_trait_composite {
    (
        $model:ty,
        $primary_key:ty,
        $keys:ty,
        $subscription_enum:ty,
        $model_tree_name:ident,
        $default_subscription:expr,
        $secondary_keys_extractor:expr,
        $relational_keys_extractor:expr
    ) => {
        impl NetabaseModelTrait<StrictDefinitions> for $model {
            type Keys = $keys;
            type Hash = [u8; 32];

            const MODEL_TREE_NAME: <StrictDefinitions as IntoDiscriminant>::Discriminant =
                StrictModelTreeNames::$model_tree_name;

            fn primary_key(&self) -> Self::PrimaryKey {
                ProductTagId {
                    product_id: self.product_id,
                    tag_id: self.tag_id,
                }
            }

            fn compute_hash(&self) -> Self::Hash {
                compute_model_hash(self)
            }

            fn get_subscriptions(&self) -> Vec<Self::Subscriptions> {
                vec![$default_subscription]
            }

            fn get_secondary_keys(
                &self,
            ) -> <Self as NetabaseModelTrait<StrictDefinitions>>::SecondaryKeys {
                $secondary_keys_extractor(self)
            }

            fn get_relational_keys(
                &self,
            ) -> <Self as NetabaseModelTrait<StrictDefinitions>>::RelationalKeys {
                $relational_keys_extractor(self)
            }
        }
    };
}

/// Helper functions for extracting keys from models
fn extract_user_secondary_keys(user: &User) -> UserSecondaryKeys {
    // Default to email - can be customized based on business logic
    UserSecondaryKeys::Email(user.email.clone())
}

fn extract_user_relational_keys(user: &User) -> UserRelationalKeys {
    // Default implementation - can be enhanced based on relationships
    UserRelationalKeys::CreatedProducts(ProductId(0)) // Placeholder
}

fn extract_product_secondary_keys(product: &Product) -> ProductSecondaryKeys {
    ProductSecondaryKeys::Title(product.title.clone())
}

fn extract_product_relational_keys(product: &Product) -> ProductRelationalKeys {
    ProductRelationalKeys::CreatedBy(product.created_by)
}

fn extract_category_secondary_keys(category: &Category) -> CategorySecondaryKeys {
    CategorySecondaryKeys::Name(category.name.clone())
}

fn extract_category_relational_keys(_category: &Category) -> CategoryRelationalKeys {
    CategoryRelationalKeys::Products(ProductId(0)) // Placeholder
}

fn extract_review_secondary_keys(review: &Review) -> ReviewSecondaryKeys {
    ReviewSecondaryKeys::Rating(review.rating)
}

fn extract_review_relational_keys(review: &Review) -> ReviewRelationalKeys {
    ReviewRelationalKeys::ReviewedProduct(review.product_id)
}

fn extract_tag_secondary_keys(tag: &Tag) -> TagSecondaryKeys {
    TagSecondaryKeys::Name(tag.name.clone())
}

fn extract_tag_relational_keys(_tag: &Tag) -> TagRelationalKeys {
    TagRelationalKeys::TaggedProducts(ProductId(0)) // Placeholder
}

fn extract_product_tag_secondary_keys(product_tag: &ProductTag) -> ProductTagSecondaryKeys {
    ProductTagSecondaryKeys::ProductId(product_tag.product_id)
}

fn extract_product_tag_relational_keys(product_tag: &ProductTag) -> ProductTagRelationalKeys {
    ProductTagRelationalKeys::Product(product_tag.product_id)
}

// Individual implementations for each model with all required methods
impl NetabaseModelTrait<StrictDefinitions> for User {
    type Keys = UserKeys;
    type Hash = [u8; 32];

    const MODEL_TREE_NAME: <StrictDefinitions as IntoDiscriminant>::Discriminant =
        StrictModelTreeNames::User;

    fn primary_key(&self) -> Self::PrimaryKey {
        self.id
    }

    fn compute_hash(&self) -> Self::Hash {
        compute_model_hash(self)
    }

    fn get_subscriptions(&self) -> Vec<Self::Subscriptions> {
        vec![UserSubscriptions::Updates]
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        extract_user_secondary_keys(self)
    }

    fn get_relational_keys(&self) -> Self::RelationalKeys {
        extract_user_relational_keys(self)
    }
}

// =================================================================================
// STRICT NESTED DEFINITION SYSTEM
// =================================================================================

/// Strict definition with nested type safety - replaces flat enum approach
#[derive(Debug, Clone, EnumDiscriminants, TryInto, EnumIter)]
#[strum_discriminants(name(StrictModelTreeNames))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "PascalCase"))]
pub enum StrictDefinitions {
    User(User),
    Product(Product),
    Category(Category),
    Review(Review),
    Tag(Tag),
    ProductTag(ProductTag),
}

impl DiscriminantName for StrictModelTreeNames {}

/// Nested associated types enum with strict typing and minimal flat structure
#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(name(StrictKeyTypes))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter))]
pub enum StrictDefinitionKeys {
    UserKeys(UserKeys),
    ProductKeys(ProductKeys),
    CategoryKeys(CategoryKeys),
    ReviewKeys(ReviewKeys),
    TagKeys(TagKeys),
    ProductTagKeys(ProductTagKeys),
}

/// Permission system using compile-time markers instead of runtime enums
#[derive(Debug, Clone, Copy, AsRefStr, EnumIter, EnumDiscriminants, Eq, PartialEq)]
#[strum_discriminants(derive(Hash, EnumIter))]
#[strum(serialize_all = "snake_case")]
pub enum StrictPermissions {
    None,
    Read,
    Write,
    Admin,
}

impl Default for StrictPermissions {
    fn default() -> Self {
        Self::None
    }
}

impl netabase_store::traits::permission::PermissionEnumTrait for StrictPermissions {
    fn permission_level(&self) -> netabase_store::traits::permission::PermissionLevel {
        match self {
            Self::None => netabase_store::traits::permission::PermissionLevel::None,
            Self::Read => netabase_store::traits::permission::PermissionLevel::Read,
            Self::Write => netabase_store::traits::permission::PermissionLevel::Write,
            Self::Admin => netabase_store::traits::permission::PermissionLevel::Admin,
        }
    }

    fn grants_access_to<R>(&self, _resource: &<R as IntoDiscriminant>::Discriminant) -> bool
    where
        R: IntoDiscriminant,
        <R as IntoDiscriminant>::Discriminant:
            strum::IntoEnumIterator + std::hash::Hash + Eq + std::fmt::Debug + Clone,
    {
        match self {
            Self::None => false,
            Self::Read => true, // Read permissions grant access to read resources
            Self::Write => true, // Write permissions grant access to read/write resources
            Self::Admin => true, // Admin permissions grant access to all resources
        }
    }
}

/// Implement the strict NetabaseDefinition trait with nested type safety
impl NetabaseDefinition for StrictDefinitions {
    type Keys = StrictDefinitionKeys;
    type Permissions = StrictPermissions;

    fn name(&self) -> String {
        "StrictDefinitions".to_owned()
    }
}

/// TreeManager implementation with type-safe string conversion
impl TreeManager<StrictDefinitions> for StrictDefinitions {}

/// NetabaseDefinitionKeyTrait implementation
impl NetabaseDefinitionKeyTrait<StrictDefinitions> for StrictDefinitionKeys {
    fn inner<M: NetabaseModelTrait<StrictDefinitions>>(&self) -> M::Keys
    where
        Self: TryInto<M::Keys>,
        <Self as TryInto<M::Keys>>::Error: std::fmt::Debug,
    {
        // This would need proper implementation based on the nested approach
        panic!("Nested key implementation needed")
    }
}

// =================================================================================
// REDB-SPECIFIC TRAIT IMPLEMENTATIONS
// =================================================================================

/// Implement RedbNetabaseModelTrait for all models with strict type safety
macro_rules! impl_redb_model_trait {
    ($model:ty, $primary_key:ty, $table_name:literal) => {
        impl ModelTreeManager<StrictDefinitions> for $model {}

        impl RedbNetabaseModelTrait<StrictDefinitions> for $model {
            fn definition(store: &RedbStore<StrictDefinitions>) -> TableDefinition<$primary_key, $model> {
                TableDefinition::new($table_name)
            }
        }
    };
}

impl_redb_model_trait!(User, UserId, "User");
impl_redb_model_trait!(Product, ProductId, "Product");
impl_redb_model_trait!(Category, CategoryId, "Category");
impl_redb_model_trait!(Review, ReviewId, "Review");
impl_redb_model_trait!(Tag, TagId, "Tag");
impl_redb_model_trait!(ProductTag, ProductTagId, "ProductTag");

// =================================================================================
// DEMONSTRATION AND TESTING
// =================================================================================

/// Create test data with strict typing
pub fn create_strict_test_data() -> (
    Vec<User>,
    Vec<Product>,
    Vec<Category>,
    Vec<Review>,
    Vec<Tag>,
    Vec<ProductTag>,
) {
    let users = vec![
        User {
            id: UserId(1),
            email: UserEmail("alice@example.com".to_string()),
            name: UserName("Alice Johnson".to_string()),
            age: UserAge(28),
            created_products: vec![ProductId(100), ProductId(101)],
        },
        User {
            id: UserId(2),
            email: UserEmail("bob@example.com".to_string()),
            name: UserName("Bob Smith".to_string()),
            age: UserAge(34),
            created_products: vec![ProductId(102)],
        },
    ];

    let categories = vec![
        Category {
            id: CategoryId(1),
            name: CategoryName("Electronics".to_string()),
            description: CategoryDescription("Electronic devices and accessories".to_string()),
        },
        Category {
            id: CategoryId(2),
            name: CategoryName("Furniture".to_string()),
            description: CategoryDescription("Home and office furniture".to_string()),
        },
    ];

    let products = vec![
        Product {
            id: ProductId(100),
            title: ProductTitle("Laptop Pro".to_string()),
            score: ProductScore(95),
            created_by: UserId(1),
            category_id: CategoryId(1),
        },
        Product {
            id: ProductId(101),
            title: ProductTitle("Wireless Mouse".to_string()),
            score: ProductScore(85),
            created_by: UserId(1),
            category_id: CategoryId(1),
        },
        Product {
            id: ProductId(102),
            title: ProductTitle("Gaming Chair".to_string()),
            score: ProductScore(92),
            created_by: UserId(2),
            category_id: CategoryId(2),
        },
    ];

    let reviews = vec![
        Review {
            id: ReviewId(1),
            product_id: ProductId(100),
            user_id: UserId(2),
            rating: ReviewRating(5),
            comment: ReviewComment("Excellent laptop! Very fast and reliable.".to_string()),
            created_at: ReviewTimestamp(1609459200),
        },
        Review {
            id: ReviewId(2),
            product_id: ProductId(101),
            user_id: UserId(2),
            rating: ReviewRating(4),
            comment: ReviewComment("Good mouse, very responsive.".to_string()),
            created_at: ReviewTimestamp(1609545600),
        },
    ];

    let tags = vec![
        Tag {
            id: TagId(1),
            name: TagName("premium".to_string()),
        },
        Tag {
            id: TagId(2),
            name: TagName("wireless".to_string()),
        },
        Tag {
            id: TagId(3),
            name: TagName("ergonomic".to_string()),
        },
    ];

    let product_tags = vec![
        ProductTag {
            product_id: ProductId(100),
            tag_id: TagId(1), // premium
        },
        ProductTag {
            product_id: ProductId(101),
            tag_id: TagId(2), // wireless
        },
        ProductTag {
            product_id: ProductId(102),
            tag_id: TagId(3), // ergonomic
        },
    ];

    (users, products, categories, reviews, tags, product_tags)
}

/// Demonstrate the strict type system advantages
pub fn demonstrate_strict_type_safety() {
    println!("=== Strict Type Safety Demonstration ===");

    // 1. Type-safe tree names using AsRefStr instead of &str
    let user_tree_name = StandardModelTreeName::<StrictDefinitions, User>::Main;
    println!("User main tree: {}", user_tree_name.as_ref());

    // 2. Compile-time permission checking
    let permission = PermissionLevel::ReadWrite;
    println!("Can write: {}", permission.can_write());

    // 3. Nested type containers prevent flat enum issues
    let user_types = User::primary_tree_name();
    println!("User primary tree name: {}", user_types.as_ref());

    // 4. Strong typing prevents mixing incompatible types
    let user_id = UserId(1);
    let product_id = ProductId(100);
    // This would cause a compile error:
    // let user_product: UserId = product_id; // Cannot assign ProductId to UserId

    println!("User ID: {:?}, Product ID: {:?}", user_id, product_id);

    // 5. Type-safe enum iteration using strum
    println!("User secondary key types:");
    for key_type in <UserSecondaryTreeNames as strum::IntoEnumIterator>::iter() {
        println!("  - {}", key_type.as_ref());
    }

    println!("=== Demonstration Complete ===");
}

/// Main function demonstrating the strict boilerplate system
pub fn main() {
    println!("=== Strict Netabase Boilerplate Example ===");

    // Create test data using strict typing
    let (users, products, categories, reviews, tags, product_tags) = create_strict_test_data();

    println!("Created test data:");
    println!("- {} users", users.len());
    println!("- {} products", products.len());
    println!("- {} categories", categories.len());
    println!("- {} reviews", reviews.len());
    println!("- {} tags", tags.len());
    println!("- {} product-tag relationships", product_tags.len());

    // Demonstrate type safety features
    demonstrate_strict_type_safety();

    // Show hash computation
    let user = &users[0];
    let user_hash = user.compute_hash();
    println!("User hash: {:?}", &user_hash[..8]); // Show first 8 bytes

    // Show subscription handling
    let subscriptions = user.get_subscriptions();
    println!("User subscriptions: {:?}", subscriptions);

    println!("=== Example Complete ===");
}
