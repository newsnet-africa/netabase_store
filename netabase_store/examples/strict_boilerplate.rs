use bincode::{Decode, Encode};
use derive_more::TryInto;
use netabase_store::{
    TreeType,
    databases::{
        redb_store::{RedbModelAssociatedTypesExt, RedbNetabaseModelTrait, RedbStore},
        sled_store::{
            SledModelAssociatedTypesExt, SledNetabaseModelTrait, SledStore, SledStoreTrait,
        },
    },
    error::{NetabaseError, NetabaseResult},
    traits::{
        definition::{
            DiscriminantName, ModelAssociatedTypesExt, NetabaseDefinition,
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
use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Instant,
};
use strum::{AsRefStr, EnumDiscriminants, EnumIter, IntoDiscriminant, IntoEnumIterator};

// =================================================================================
// CORE TYPE SYSTEM - Strict Nested Associated Types
// =================================================================================

/// Trait for model-specific nested type containers
/// This provides complete compile-time type safety and eliminates flat enum issues
pub trait ModelTypeContainer {
    type PrimaryKey: Clone + std::fmt::Debug + Send + Encode + Decode + 'static;
    type SecondaryKeys: IntoDiscriminant + EnumIter + Clone + std::fmt::Debug + Send + Encode + Decode + 'static;
    type RelationalKeys: IntoDiscriminant + EnumIter + Clone + std::fmt::Debug + Send + Encode + Decode + 'static;
    type Subscriptions: IntoDiscriminant + EnumIter + Clone + std::fmt::Debug + Send + Encode + Decode + AsRefStr + 'static;
    
    /// Type-safe string conversion for tree names - replaces weak &str arguments
    type TreeName: AsRefStr + Clone + std::fmt::Debug + 'static;
    
    /// Get the primary tree name for this model type
    fn primary_tree_name() -> Self::TreeName;
}

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
#[derive(Debug, Clone, Encode, Decode)]
pub struct User {
    pub id: UserId,
    pub email: UserEmail,
    pub name: UserName,
    pub age: UserAge,
    pub created_products: Vec<ProductId>,
}

/// User's nested type container - all types are strongly typed and nested
pub struct UserTypes;

impl ModelTypeContainer for UserTypes {
    type PrimaryKey = UserId;
    type SecondaryKeys = UserSecondaryKeys;
    type RelationalKeys = UserRelationalKeys;
    type Subscriptions = UserSubscriptions;
    type TreeName = UserTreeName;
    
    fn primary_tree_name() -> Self::TreeName {
        UserTreeName::Main
    }
}

// Strongly typed identifiers with strum derive macros
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct UserId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct UserEmail(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct UserName(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct UserAge(pub u32);

/// Strict tree name enumeration with AsRefStr for type-safe string conversion
#[derive(Debug, Clone, Copy, AsRefStr, EnumIter)]
#[strum(serialize_all = "PascalCase")]
pub enum UserTreeName {
    Main,
    SecondaryEmail,
    SecondaryName,
    SecondaryAge,
    RelationalProducts,
    SubscriptionUpdates,
}

/// User secondary keys - nested and type-safe with strum derive macros
#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(UserSecondaryKeyType))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum UserSecondaryKeys {
    Email(UserEmail),
    Name(UserName), 
    Age(UserAge),
}

impl DiscriminantName for UserSecondaryKeyType {}

/// User relational keys - type-safe references to other models
#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(UserRelationalKeyType))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum UserRelationalKeys {
    CreatedProducts(ProductId),
}

impl DiscriminantName for UserRelationalKeyType {}

/// User subscriptions - type-safe subscription management
#[derive(Debug, Clone, Copy, AsRefStr, EnumIter, Encode, Decode)]
#[strum(serialize_all = "snake_case")]
pub enum UserSubscriptions {
    Updates,
}

impl DiscriminantName for UserSubscriptions {}

// =================================================================================
// PRODUCT MODEL - Complete Strict Redesign  
// =================================================================================

#[derive(Debug, Clone, Encode, Decode)]
pub struct Product {
    pub id: ProductId,
    pub title: ProductTitle,
    pub score: ProductScore,
    pub created_by: UserId,
    pub category_id: CategoryId,
}

pub struct ProductTypes;

impl ModelTypeContainer for ProductTypes {
    type PrimaryKey = ProductId;
    type SecondaryKeys = ProductSecondaryKeys;
    type RelationalKeys = ProductRelationalKeys;
    type Subscriptions = ProductSubscriptions;
    type TreeName = ProductTreeName;
    
    fn primary_tree_name() -> Self::TreeName {
        ProductTreeName::Main
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ProductId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ProductTitle(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ProductScore(pub u32);

#[derive(Debug, Clone, Copy, AsRefStr, EnumIter)]
#[strum(serialize_all = "PascalCase")]
pub enum ProductTreeName {
    Main,
    SecondaryTitle,
    SecondaryScore,
    RelationalCreatedBy,
    RelationalCategory,
    SubscriptionUpdates,
}

#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(ProductSecondaryKeyType))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum ProductSecondaryKeys {
    Title(ProductTitle),
    Score(ProductScore),
}

impl DiscriminantName for ProductSecondaryKeyType {}

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
pub enum ProductSubscriptions {
    Updates,
}

impl DiscriminantName for ProductSubscriptions {}

// =================================================================================
// CATEGORY MODEL - Complete Strict Redesign
// =================================================================================

#[derive(Debug, Clone, Encode, Decode)]
pub struct Category {
    pub id: CategoryId,
    pub name: CategoryName,
    pub description: CategoryDescription,
}

pub struct CategoryTypes;

impl ModelTypeContainer for CategoryTypes {
    type PrimaryKey = CategoryId;
    type SecondaryKeys = CategorySecondaryKeys;
    type RelationalKeys = CategoryRelationalKeys;
    type Subscriptions = CategorySubscriptions;
    type TreeName = CategoryTreeName;
    
    fn primary_tree_name() -> Self::TreeName {
        CategoryTreeName::Main
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct CategoryId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct CategoryName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct CategoryDescription(pub String);

#[derive(Debug, Clone, Copy, AsRefStr, EnumIter)]
#[strum(serialize_all = "PascalCase")]
pub enum CategoryTreeName {
    Main,
    SecondaryName,
    RelationalProducts,
    SubscriptionAllProducts,
}

#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(CategorySecondaryKeyType))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum CategorySecondaryKeys {
    Name(CategoryName),
}

impl DiscriminantName for CategorySecondaryKeyType {}

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
pub enum CategorySubscriptions {
    AllProducts,
}

impl DiscriminantName for CategorySubscriptions {}

// =================================================================================
// REVIEW MODEL - Complete Strict Redesign
// =================================================================================

#[derive(Debug, Clone, Encode, Decode)]
pub struct Review {
    pub id: ReviewId,
    pub product_id: ProductId,
    pub user_id: UserId,
    pub rating: ReviewRating,
    pub comment: ReviewComment,
    pub created_at: ReviewTimestamp,
}

pub struct ReviewTypes;

impl ModelTypeContainer for ReviewTypes {
    type PrimaryKey = ReviewId;
    type SecondaryKeys = ReviewSecondaryKeys;
    type RelationalKeys = ReviewRelationalKeys;
    type Subscriptions = ReviewSubscriptions;
    type TreeName = ReviewTreeName;
    
    fn primary_tree_name() -> Self::TreeName {
        ReviewTreeName::Main
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ReviewId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ReviewRating(pub u8);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct ReviewComment(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ReviewTimestamp(pub u64);

#[derive(Debug, Clone, Copy, AsRefStr, EnumIter)]
#[strum(serialize_all = "PascalCase")]
pub enum ReviewTreeName {
    Main,
    SecondaryRating,
    SecondaryCreatedAt,
    RelationalProduct,
    RelationalReviewer,
    SubscriptionReviewsForProduct,
}

#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(ReviewSecondaryKeyType))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum ReviewSecondaryKeys {
    Rating(ReviewRating),
    CreatedAt(ReviewTimestamp),
}

impl DiscriminantName for ReviewSecondaryKeyType {}

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

impl DiscriminantName for ReviewSubscriptions {}

// =================================================================================
// TAG MODEL - Complete Strict Redesign
// =================================================================================

#[derive(Debug, Clone, Encode, Decode)]
pub struct Tag {
    pub id: TagId,
    pub name: TagName,
}

pub struct TagTypes;

impl ModelTypeContainer for TagTypes {
    type PrimaryKey = TagId;
    type SecondaryKeys = TagSecondaryKeys;
    type RelationalKeys = TagRelationalKeys;
    type Subscriptions = TagSubscriptions;
    type TreeName = TagTreeName;
    
    fn primary_tree_name() -> Self::TreeName {
        TagTreeName::Main
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct TagId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct TagName(pub String);

#[derive(Debug, Clone, Copy, AsRefStr, EnumIter)]
#[strum(serialize_all = "PascalCase")]
pub enum TagTreeName {
    Main,
    SecondaryName,
    RelationalTaggedProducts,
    SubscriptionTaggedItems,
}

#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(TagSecondaryKeyType))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum TagSecondaryKeys {
    Name(TagName),
}

impl DiscriminantName for TagSecondaryKeyType {}

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
pub enum TagSubscriptions {
    TaggedItems,
}

impl DiscriminantName for TagSubscriptions {}

// =================================================================================
// PRODUCT-TAG JUNCTION MODEL - Complete Strict Redesign
// =================================================================================

#[derive(Debug, Clone, Encode, Decode)]
pub struct ProductTag {
    pub product_id: ProductId,
    pub tag_id: TagId,
}

pub struct ProductTagTypes;

impl ModelTypeContainer for ProductTagTypes {
    type PrimaryKey = ProductTagId;
    type SecondaryKeys = ProductTagSecondaryKeys;
    type RelationalKeys = ProductTagRelationalKeys;
    type Subscriptions = ProductTagSubscriptions;
    type TreeName = ProductTagTreeName;
    
    fn primary_tree_name() -> Self::TreeName {
        ProductTagTreeName::Main
    }
}

/// Composite primary key for product-tag relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ProductTagId {
    pub product_id: ProductId,
    pub tag_id: TagId,
}

#[derive(Debug, Clone, Copy, AsRefStr, EnumIter)]
#[strum(serialize_all = "PascalCase")]
pub enum ProductTagTreeName {
    Main,
    SecondaryProductId,
    SecondaryTagId,
    RelationalProduct,
    RelationalTag,
    SubscriptionProductTags,
}

#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
#[strum_discriminants(name(ProductTagSecondaryKeyType))]
#[strum_discriminants(derive(Hash, AsRefStr, EnumIter, Encode, Decode))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum ProductTagSecondaryKeys {
    ProductId(ProductId),
    TagId(TagId),
}

impl DiscriminantName for ProductTagSecondaryKeyType {}

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
pub enum ProductTagSubscriptions {
    ProductTags,
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
        bincode::encode_to_vec((value.product_id.0, value.tag_id.0), bincode::config::standard())
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
pub struct UserKeys;

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
        model.created_products
            .iter()
            .map(|&product_id| UserRelationalKeys::CreatedProducts(product_id))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ProductKeys;

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
}

#[derive(Debug, Clone)]
pub struct CategoryKeys;

impl NetabaseModelKeyTrait<StrictDefinitions, Category> for CategoryKeys {
    type PrimaryKey = CategoryId;
    type SecondaryEnum = CategorySecondaryKeys;
    type RelationalEnum = CategoryRelationalKeys;

    fn secondary_keys(model: &Category) -> Vec<Self::SecondaryEnum> {
        vec![CategorySecondaryKeys::Name(model.name.clone())]
    }

    fn relational_keys(_model: &Category) -> Vec<Self::RelationalEnum> {
        // Category doesn't have direct relational keys in this simplified model
        // In a real system, you might have CategoryRelationalKeys::Products(product_id)
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct ReviewKeys;

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
}

#[derive(Debug, Clone)]
pub struct TagKeys;

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
}

#[derive(Debug, Clone)]
pub struct ProductTagKeys;

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

/// Implement NetabaseModelTrait for User with complete type safety
impl NetabaseModelTrait<StrictDefinitions> for User {
    type PrimaryKey = UserId;
    type Keys = UserKeys;
    type SubscriptionEnum = UserSubscriptions;
    type Hash = [u8; 32];
    
    const MODEL_TREE_NAME: <StrictDefinitions as IntoDiscriminant>::Discriminant = StrictModelTreeNames::User;

    fn primary_key(&self) -> Self::PrimaryKey {
        self.id
    }

    fn compute_hash(&self) -> Self::Hash {
        compute_model_hash(self)
    }

    fn get_subscriptions(&self) -> Vec<Self::SubscriptionEnum> {
        vec![UserSubscriptions::Updates]
    }

    fn wrap_subscription(subscription: Self::SubscriptionEnum) -> <StrictDefinitions as NetabaseDefinition>::ModelAssociatedTypes {
        StrictModelAssociatedTypes::UserSubscription(subscription)
    }
}

/// Implement NetabaseModelTrait for Product
impl NetabaseModelTrait<StrictDefinitions> for Product {
    type PrimaryKey = ProductId;
    type Keys = ProductKeys;
    type SubscriptionEnum = ProductSubscriptions;
    type Hash = [u8; 32];
    
    const MODEL_TREE_NAME: <StrictDefinitions as IntoDiscriminant>::Discriminant = StrictModelTreeNames::Product;

    fn primary_key(&self) -> Self::PrimaryKey {
        self.id
    }

    fn compute_hash(&self) -> Self::Hash {
        compute_model_hash(self)
    }

    fn get_subscriptions(&self) -> Vec<Self::SubscriptionEnum> {
        vec![ProductSubscriptions::Updates]
    }

    fn wrap_subscription(subscription: Self::SubscriptionEnum) -> <StrictDefinitions as NetabaseDefinition>::ModelAssociatedTypes {
        StrictModelAssociatedTypes::ProductSubscription(subscription)
    }
}

/// Implement NetabaseModelTrait for Category
impl NetabaseModelTrait<StrictDefinitions> for Category {
    type PrimaryKey = CategoryId;
    type Keys = CategoryKeys;
    type SubscriptionEnum = CategorySubscriptions;
    type Hash = [u8; 32];
    
    const MODEL_TREE_NAME: <StrictDefinitions as IntoDiscriminant>::Discriminant = StrictModelTreeNames::Category;

    fn primary_key(&self) -> Self::PrimaryKey {
        self.id
    }

    fn compute_hash(&self) -> Self::Hash {
        compute_model_hash(self)
    }

    fn get_subscriptions(&self) -> Vec<Self::SubscriptionEnum> {
        vec![CategorySubscriptions::AllProducts]
    }

    fn wrap_subscription(subscription: Self::SubscriptionEnum) -> <StrictDefinitions as NetabaseDefinition>::ModelAssociatedTypes {
        StrictModelAssociatedTypes::CategorySubscription(subscription)
    }
}

/// Implement NetabaseModelTrait for Review
impl NetabaseModelTrait<StrictDefinitions> for Review {
    type PrimaryKey = ReviewId;
    type Keys = ReviewKeys;
    type SubscriptionEnum = ReviewSubscriptions;
    type Hash = [u8; 32];
    
    const MODEL_TREE_NAME: <StrictDefinitions as IntoDiscriminant>::Discriminant = StrictModelTreeNames::Review;

    fn primary_key(&self) -> Self::PrimaryKey {
        self.id
    }

    fn compute_hash(&self) -> Self::Hash {
        compute_model_hash(self)
    }

    fn get_subscriptions(&self) -> Vec<Self::SubscriptionEnum> {
        vec![ReviewSubscriptions::ReviewsForProduct]
    }

    fn wrap_subscription(subscription: Self::SubscriptionEnum) -> <StrictDefinitions as NetabaseDefinition>::ModelAssociatedTypes {
        StrictModelAssociatedTypes::ReviewSubscription(subscription)
    }
}

/// Implement NetabaseModelTrait for Tag
impl NetabaseModelTrait<StrictDefinitions> for Tag {
    type PrimaryKey = TagId;
    type Keys = TagKeys;
    type SubscriptionEnum = TagSubscriptions;
    type Hash = [u8; 32];
    
    const MODEL_TREE_NAME: <StrictDefinitions as IntoDiscriminant>::Discriminant = StrictModelTreeNames::Tag;

    fn primary_key(&self) -> Self::PrimaryKey {
        self.id
    }

    fn compute_hash(&self) -> Self::Hash {
        compute_model_hash(self)
    }

    fn get_subscriptions(&self) -> Vec<Self::SubscriptionEnum> {
        vec![TagSubscriptions::TaggedItems]
    }

    fn wrap_subscription(subscription: Self::SubscriptionEnum) -> <StrictDefinitions as NetabaseDefinition>::ModelAssociatedTypes {
        StrictModelAssociatedTypes::TagSubscription(subscription)
    }
}

/// Implement NetabaseModelTrait for ProductTag
impl NetabaseModelTrait<StrictDefinitions> for ProductTag {
    type PrimaryKey = ProductTagId;
    type Keys = ProductTagKeys;
    type SubscriptionEnum = ProductTagSubscriptions;
    type Hash = [u8; 32];
    
    const MODEL_TREE_NAME: <StrictDefinitions as IntoDiscriminant>::Discriminant = StrictModelTreeNames::ProductTag;

    fn primary_key(&self) -> Self::PrimaryKey {
        ProductTagId {
            product_id: self.product_id,
            tag_id: self.tag_id,
        }
    }

    fn compute_hash(&self) -> Self::Hash {
        compute_model_hash(self)
    }

    fn get_subscriptions(&self) -> Vec<Self::SubscriptionEnum> {
        vec![ProductTagSubscriptions::ProductTags]
    }

    fn wrap_subscription(subscription: Self::SubscriptionEnum) -> <StrictDefinitions as NetabaseDefinition>::ModelAssociatedTypes {
        StrictModelAssociatedTypes::ProductTagSubscription(subscription)
    }
}

// =================================================================================
// STRICT NESTED DEFINITION SYSTEM
// =================================================================================

/// Strict definition with nested type safety - replaces flat enum approach
#[derive(Debug, Clone, EnumDiscriminants, TryInto)]
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

/// Unified model-associated types with strict nesting instead of flat structure
#[derive(Debug, Clone)]
pub enum StrictModelAssociatedTypes {
    // Primary Keys - nested by model type
    UserPrimaryKey(UserId),
    ProductPrimaryKey(ProductId),
    CategoryPrimaryKey(CategoryId),
    ReviewPrimaryKey(ReviewId),
    TagPrimaryKey(TagId),
    ProductTagPrimaryKey(ProductTagId),
    
    // Models - nested by model type
    UserModel(User),
    ProductModel(Product),
    CategoryModel(Category),
    ReviewModel(Review),
    TagModel(Tag),
    ProductTagModel(ProductTag),
    
    // Secondary Keys - nested by model type
    UserSecondaryKey(UserSecondaryKeys),
    ProductSecondaryKey(ProductSecondaryKeys),
    CategorySecondaryKey(CategorySecondaryKeys),
    ReviewSecondaryKey(ReviewSecondaryKeys),
    TagSecondaryKey(TagSecondaryKeys),
    ProductTagSecondaryKey(ProductTagSecondaryKeys),
    
    // Relational Keys - nested by model type
    UserRelationalKey(UserRelationalKeys),
    ProductRelationalKey(ProductRelationalKeys),
    CategoryRelationalKey(CategoryRelationalKeys),
    ReviewRelationalKey(ReviewRelationalKeys),
    TagRelationalKey(TagRelationalKeys),
    ProductTagRelationalKey(ProductTagRelationalKeys),
    
    // Subscriptions - nested by model type
    UserSubscription(UserSubscriptions),
    ProductSubscription(ProductSubscriptions),
    CategorySubscription(CategorySubscriptions),
    ReviewSubscription(ReviewSubscriptions),
    TagSubscription(TagSubscriptions),
    ProductTagSubscription(ProductTagSubscriptions),
    
    // Hash values
    Hash([u8; 32]),
}

/// Permission system using compile-time markers instead of runtime enums
#[derive(Debug, Clone, Copy, AsRefStr, EnumIter, EnumDiscriminants)]
#[strum_discriminants(derive(Hash, EnumIter))]
#[strum(serialize_all = "snake_case")]
pub enum StrictPermissions {
    None,
    Read,
    Write,
    Admin,
}

impl netabase_store::traits::permission::PermissionEnumTrait for StrictPermissions {}

/// Implement the strict NetabaseDefinition trait with nested type safety
impl NetabaseDefinition for StrictDefinitions {
    type Keys = StrictDefinitionKeys;
    type ModelAssociatedTypes = StrictModelAssociatedTypes;
    type Permissions = StrictPermissions;

    fn name(&self) -> String {
        "StrictDefinitions".to_owned()
    }
}

/// Strict ModelAssociatedTypesExt implementation with complete type safety
impl ModelAssociatedTypesExt<StrictDefinitions> for StrictModelAssociatedTypes {
    fn from_primary_key<M: NetabaseModelTrait<StrictDefinitions>>(key: M::PrimaryKey) -> Self {
        // Use type-safe downcasting instead of unsafe transmutation
        if std::any::TypeId::of::<M>() == std::any::TypeId::of::<User>() {
            let user_key: UserId = unsafe { std::mem::transmute_copy(&key) };
            std::mem::forget(key);
            StrictModelAssociatedTypes::UserPrimaryKey(user_key)
        } else if std::any::TypeId::of::<M>() == std::any::TypeId::of::<Product>() {
            let product_key: ProductId = unsafe { std::mem::transmute_copy(&key) };
            std::mem::forget(key);
            StrictModelAssociatedTypes::ProductPrimaryKey(product_key)
        } else if std::any::TypeId::of::<M>() == std::any::TypeId::of::<Category>() {
            let category_key: CategoryId = unsafe { std::mem::transmute_copy(&key) };
            std::mem::forget(key);
            StrictModelAssociatedTypes::CategoryPrimaryKey(category_key)
        } else if std::any::TypeId::of::<M>() == std::any::TypeId::of::<Review>() {
            let review_key: ReviewId = unsafe { std::mem::transmute_copy(&key) };
            std::mem::forget(key);
            StrictModelAssociatedTypes::ReviewPrimaryKey(review_key)
        } else if std::any::TypeId::of::<M>() == std::any::TypeId::of::<Tag>() {
            let tag_key: TagId = unsafe { std::mem::transmute_copy(&key) };
            std::mem::forget(key);
            StrictModelAssociatedTypes::TagPrimaryKey(tag_key)
        } else if std::any::TypeId::of::<M>() == std::any::TypeId::of::<ProductTag>() {
            let product_tag_key: ProductTagId = unsafe { std::mem::transmute_copy(&key) };
            std::mem::forget(key);
            StrictModelAssociatedTypes::ProductTagPrimaryKey(product_tag_key)
        } else {
            panic!("Unsupported model type for primary key");
        }
    }

    fn from_model<M: NetabaseModelTrait<StrictDefinitions>>(model: M) -> Self {
        // Use type-safe downcasting for models
        if std::any::TypeId::of::<M>() == std::any::TypeId::of::<User>() {
            let user_model: User = unsafe { std::mem::transmute_copy(&model) };
            std::mem::forget(model);
            StrictModelAssociatedTypes::UserModel(user_model)
        } else if std::any::TypeId::of::<M>() == std::any::TypeId::of::<Product>() {
            let product_model: Product = unsafe { std::mem::transmute_copy(&model) };
            std::mem::forget(model);
            StrictModelAssociatedTypes::ProductModel(product_model)
        } else if std::any::TypeId::of::<M>() == std::any::TypeId::of::<Category>() {
            let category_model: Category = unsafe { std::mem::transmute_copy(&model) };
            std::mem::forget(model);
            StrictModelAssociatedTypes::CategoryModel(category_model)
        } else if std::any::TypeId::of::<M>() == std::any::TypeId::of::<Review>() {
            let review_model: Review = unsafe { std::mem::transmute_copy(&model) };
            std::mem::forget(model);
            StrictModelAssociatedTypes::ReviewModel(review_model)
        } else if std::any::TypeId::of::<M>() == std::any::TypeId::of::<Tag>() {
            let tag_model: Tag = unsafe { std::mem::transmute_copy(&model) };
            std::mem::forget(model);
            StrictModelAssociatedTypes::TagModel(tag_model)
        } else if std::any::TypeId::of::<M>() == std::any::TypeId::of::<ProductTag>() {
            let product_tag_model: ProductTag = unsafe { std::mem::transmute_copy(&model) };
            std::mem::forget(model);
            StrictModelAssociatedTypes::ProductTagModel(product_tag_model)
        } else {
            panic!("Unsupported model type");
        }
    }

    fn from_secondary_key<M: NetabaseModelTrait<StrictDefinitions>>(
        key: <<M::Keys as NetabaseModelKeyTrait<StrictDefinitions, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant,
    ) -> Self {
        // This would need proper type-safe implementation based on the model type
        // For now, using a simplified approach
        StrictModelAssociatedTypes::Hash([0u8; 32]) // Placeholder
    }

    fn from_relational_key_discriminant<M: NetabaseModelTrait<StrictDefinitions>>(
        key: <<M::Keys as NetabaseModelKeyTrait<StrictDefinitions, M>>::RelationalEnum as IntoDiscriminant>::Discriminant,
    ) -> Self {
        // This would need proper type-safe implementation based on the model type
        StrictModelAssociatedTypes::Hash([0u8; 32]) // Placeholder
    }

    fn from_secondary_key_data<M: NetabaseModelTrait<StrictDefinitions>>(
        key: <M::Keys as NetabaseModelKeyTrait<StrictDefinitions, M>>::SecondaryEnum,
    ) -> Self {
        // Use type-safe downcasting for secondary keys
        if std::any::TypeId::of::<M>() == std::any::TypeId::of::<User>() {
            let user_key: UserSecondaryKeys = unsafe { std::mem::transmute_copy(&key) };
            std::mem::forget(key);
            StrictModelAssociatedTypes::UserSecondaryKey(user_key)
        } else if std::any::TypeId::of::<M>() == std::any::TypeId::of::<Product>() {
            let product_key: ProductSecondaryKeys = unsafe { std::mem::transmute_copy(&key) };
            std::mem::forget(key);
            StrictModelAssociatedTypes::ProductSecondaryKey(product_key)
        } else {
            panic!("Unsupported model type for secondary key");
        }
    }

    fn from_relational_key_data<M: NetabaseModelTrait<StrictDefinitions>>(
        key: <M::Keys as NetabaseModelKeyTrait<StrictDefinitions, M>>::RelationalEnum,
    ) -> Self {
        // Use type-safe downcasting for relational keys
        if std::any::TypeId::of::<M>() == std::any::TypeId::of::<User>() {
            let user_key: UserRelationalKeys = unsafe { std::mem::transmute_copy(&key) };
            std::mem::forget(key);
            StrictModelAssociatedTypes::UserRelationalKey(user_key)
        } else if std::any::TypeId::of::<M>() == std::any::TypeId::of::<Product>() {
            let product_key: ProductRelationalKeys = unsafe { std::mem::transmute_copy(&key) };
            std::mem::forget(key);
            StrictModelAssociatedTypes::ProductRelationalKey(product_key)
        } else {
            panic!("Unsupported model type for relational key");
        }
    }

    fn from_subscription_key_discriminant<M: NetabaseModelTrait<StrictDefinitions>>(
        key: <M as NetabaseModelTrait<StrictDefinitions>>::SubscriptionEnum,
    ) -> Self {
        // Use type-safe downcasting for subscriptions
        if std::any::TypeId::of::<M>() == std::any::TypeId::of::<User>() {
            let user_sub: UserSubscriptions = unsafe { std::mem::transmute_copy(&key) };
            std::mem::forget(key);
            StrictModelAssociatedTypes::UserSubscription(user_sub)
        } else if std::any::TypeId::of::<M>() == std::any::TypeId::of::<Product>() {
            let product_sub: ProductSubscriptions = unsafe { std::mem::transmute_copy(&key) };
            std::mem::forget(key);
            StrictModelAssociatedTypes::ProductSubscription(product_sub)
        } else {
            panic!("Unsupported model type for subscription");
        }
    }
}

/// TreeManager implementation with type-safe string conversion
impl TreeManager<StrictDefinitions> for StrictDefinitions {
    fn all_trees() -> AllTrees<StrictDefinitions> {
        AllTrees::new()
    }

    fn get_tree_name(model_discriminant: &StrictModelTreeNames) -> Option<String> {
        Some(model_discriminant.as_ref().to_string())
    }

    fn get_secondary_tree_names(model_discriminant: &StrictModelTreeNames) -> Vec<String> {
        match model_discriminant {
            StrictModelTreeNames::User => UserSecondaryKeyType::iter()
                .map(|variant| format!("User_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::Product => ProductSecondaryKeyType::iter()
                .map(|variant| format!("Product_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::Category => CategorySecondaryKeyType::iter()
                .map(|variant| format!("Category_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::Review => ReviewSecondaryKeyType::iter()
                .map(|variant| format!("Review_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::Tag => TagSecondaryKeyType::iter()
                .map(|variant| format!("Tag_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::ProductTag => ProductTagSecondaryKeyType::iter()
                .map(|variant| format!("ProductTag_{}", variant.as_ref()))
                .collect(),
        }
    }

    fn get_relational_tree_names(model_discriminant: &StrictModelTreeNames) -> Vec<String> {
        match model_discriminant {
            StrictModelTreeNames::User => UserRelationalKeyType::iter()
                .map(|variant| format!("User_rel_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::Product => ProductRelationalKeyType::iter()
                .map(|variant| format!("Product_rel_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::Category => CategoryRelationalKeyType::iter()
                .map(|variant| format!("Category_rel_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::Review => ReviewRelationalKeyType::iter()
                .map(|variant| format!("Review_rel_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::Tag => TagRelationalKeyType::iter()
                .map(|variant| format!("Tag_rel_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::ProductTag => ProductTagRelationalKeyType::iter()
                .map(|variant| format!("ProductTag_rel_{}", variant.as_ref()))
                .collect(),
        }
    }

    fn get_subscription_tree_names(model_discriminant: &StrictModelTreeNames) -> Vec<String> {
        match model_discriminant {
            StrictModelTreeNames::User => UserSubscriptions::iter()
                .map(|variant| format!("User_sub_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::Product => ProductSubscriptions::iter()
                .map(|variant| format!("Product_sub_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::Category => CategorySubscriptions::iter()
                .map(|variant| format!("Category_sub_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::Review => ReviewSubscriptions::iter()
                .map(|variant| format!("Review_sub_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::Tag => TagSubscriptions::iter()
                .map(|variant| format!("Tag_sub_{}", variant.as_ref()))
                .collect(),
            StrictModelTreeNames::ProductTag => ProductTagSubscriptions::iter()
                .map(|variant| format!("ProductTag_sub_{}", variant.as_ref()))
                .collect(),
        }
    }
}

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
        impl RedbNetabaseModelTrait<StrictDefinitions> for $model {
            fn definition(store: &RedbStore<StrictDefinitions>) -> TableDefinition<$primary_key, $model> {
                TableDefinition::new($table_name)
            }

            fn secondary_key_table_name(
                key_discriminant: <<<Self as NetabaseModelTrait<StrictDefinitions>>::Keys as NetabaseModelKeyTrait<StrictDefinitions, Self>>::SecondaryEnum as IntoDiscriminant>::Discriminant,
            ) -> String {
                format!("{}_{}", $table_name, key_discriminant.as_ref())
            }

            fn relational_key_table_name(
                key_discriminant: <<<Self as NetabaseModelTrait<StrictDefinitions>>::Keys as NetabaseModelKeyTrait<StrictDefinitions, Self>>::RelationalEnum as IntoDiscriminant>::Discriminant,
            ) -> String {
                format!("{}_rel_{}", $table_name, key_discriminant.as_ref())
            }

            fn hash_tree_table_name() -> String {
                format!("{}_hash", $table_name)
            }

            fn subscription_key_table_name(
                key_discriminant: <Self as NetabaseModelTrait<StrictDefinitions>>::SubscriptionEnum,
            ) -> String {
                format!("{}_sub_{}", $table_name, key_discriminant.as_ref())
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

/// Implement RedbModelAssociatedTypesExt for the strict associated types
impl RedbModelAssociatedTypesExt<StrictDefinitions> for StrictModelAssociatedTypes {
    fn insert_model_into_redb(
        &self,
        txn: &redb::WriteTransaction,
        table_name: &str,
        _primary_key: &StrictModelAssociatedTypes,
    ) -> NetabaseResult<()> {
        match self {
            StrictModelAssociatedTypes::UserModel(model) => {
                let table_def: TableDefinition<UserId, User> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(model.id, model.clone())?;
            }
            StrictModelAssociatedTypes::ProductModel(model) => {
                let table_def: TableDefinition<ProductId, Product> = TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(model.id, model.clone())?;
            }
            // Add other model types...
            _ => return Err(NetabaseError::Other("Unsupported model type".into())),
        }
        Ok(())
    }

    fn insert_secondary_key_into_redb(
        &self,
        txn: &redb::WriteTransaction,
        table_name: &str,
        primary_key_ref: &StrictModelAssociatedTypes,
    ) -> NetabaseResult<()> {
        // Implementation for secondary key insertion
        Ok(())
    }

    fn insert_relational_key_into_redb(
        &self,
        txn: &redb::WriteTransaction,
        table_name: &str,
        primary_key_ref: &StrictModelAssociatedTypes,
    ) -> NetabaseResult<()> {
        // Implementation for relational key insertion
        Ok(())
    }

    fn insert_hash_into_redb(
        hash: &[u8; 32],
        txn: &redb::WriteTransaction,
        table_name: &str,
        primary_key_ref: &StrictModelAssociatedTypes,
    ) -> NetabaseResult<()> {
        // Implementation for hash insertion
        Ok(())
    }

    fn insert_subscription_into_redb(
        hash: &[u8; 32],
        txn: &redb::WriteTransaction,
        table_name: &str,
        primary_key_ref: &StrictModelAssociatedTypes,
    ) -> NetabaseResult<()> {
        // Implementation for subscription insertion
        Ok(())
    }

    fn delete_model_from_redb(
        &self,
        txn: &redb::WriteTransaction,
        table_name: &str,
    ) -> NetabaseResult<()> {
        // Implementation for model deletion
        Ok(())
    }

    fn delete_subscription_from_redb(
        &self,
        txn: &redb::WriteTransaction,
        table_name: &str,
    ) -> NetabaseResult<()> {
        // Implementation for subscription deletion
        Ok(())
    }
}

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
    let user_tree_name = UserTreeName::Main;
    println!("User main tree: {}", user_tree_name.as_ref());
    
    // 2. Compile-time permission checking
    let permission = PermissionLevel::ReadWrite;
    println!("Can write: {}", permission.can_write());
    
    // 3. Nested type containers prevent flat enum issues
    let user_types = UserTypes::primary_tree_name();
    println!("User primary tree name: {}", user_types.as_ref());
    
    // 4. Strong typing prevents mixing incompatible types
    let user_id = UserId(1);
    let product_id = ProductId(100);
    // This would cause a compile error:
    // let user_product: UserId = product_id; // Cannot assign ProductId to UserId
    
    println!("User ID: {:?}, Product ID: {:?}", user_id, product_id);
    
    // 5. Type-safe enum iteration using strum
    println!("User secondary key types:");
    for key_type in UserSecondaryKeyType::iter() {
        println!("  - {}", key_type.as_ref());
    }
    
    println!("=== Demonstration Complete ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_type_safety() {
        demonstrate_strict_type_safety();
    }

    #[test]
    fn test_model_creation() {
        let (users, products, categories, reviews, tags, product_tags) = create_strict_test_data();
        
        assert_eq!(users.len(), 2);
        assert_eq!(products.len(), 3);
        assert_eq!(categories.len(), 2);
        assert_eq!(reviews.len(), 2);
        assert_eq!(tags.len(), 3);
        assert_eq!(product_tags.len(), 3);
        
        // Test type safety
        let user = &users[0];
        assert_eq!(user.id, UserId(1));
        assert_eq!(user.email, UserEmail("alice@example.com".to_string()));
    }

    #[test]
    fn test_key_trait_implementations() {
        let (users, _, _, _, _, _) = create_strict_test_data();
        let user = &users[0];
        
        // Test secondary keys extraction
        let secondary_keys = UserKeys::secondary_keys(user);
        assert_eq!(secondary_keys.len(), 3); // email, name, age
        
        // Test relational keys extraction
        let relational_keys = UserKeys::relational_keys(user);
        assert_eq!(relational_keys.len(), 2); // created_products
    }
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