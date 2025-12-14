use netabase_store::traits::registery::{
    definition::{NetabaseDefinition, NetabaseDefinitionTreeNames},
    models::{
        StoreKey, StoreValue, StoreValueMarker,
        keys::{
            NetabaseModelKeys, NetabaseModelPrimaryKey, NetabaseModelRelationalKey,
            NetabaseModelSecondaryKey,
        },
        model::{
            DiscriminantTableName, ModelTreeNames, NetabaseModel, NetabaseModelMarker,
            RedbNetbaseModel,
        },
    },
};

use bincode::{Decode, Encode};
use redb::{Key, Value};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use strum::{AsRefStr, EnumDiscriminants};

// --- User Model ---

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct User {
    id: UserID,
    name: String,
    age: u8,
    partner: UserID,
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct UserID(String);
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct UserName(String);
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct UserAge(u8);

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct UserPartner(UserID);

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
)]
#[strum_discriminants(name(UserRelationalKeysDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum UserRelationalKeys {
    Partner(UserPartner),
}

pub enum UserKeys {
    Primary(UserID),
    Secondary(UserSecondaryKeys),
    Relational(UserRelationalKeys),
}

// --- Post Model ---

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Post {
    id: PostID,
    title: String,
    author: UserID,
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct PostID(String);
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct PostTitle(String);

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
)]
#[strum_discriminants(name(PostRelationalKeysDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum PostRelationalKeys {
    Author(UserID),
}

pub enum PostKeys {
    Primary(PostID),
    Secondary(PostSecondaryKeys),
    Relational(PostRelationalKeys),
}

// --- Definition ---

#[derive(EnumDiscriminants)]
#[strum_discriminants(name(DefinitionDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum Definition {
    User(User),
    Post(Post),
}

pub struct DefinitionTreeNames;
impl NetabaseDefinitionTreeNames for DefinitionTreeNames {}

impl NetabaseDefinition for Definition {
    type TreeNames = DefinitionTreeNames;
}

// --- User Implementation ---

impl NetabaseModel<Definition> for User {
    type Keys = UserKeys;

    fn get_primary_key<'a>(&'a self) -> UserID {
        self.id.clone()
    }

    fn get_secondary_keys<'a>(&'a self) -> Vec<UserSecondaryKeys> {
        vec![
            UserSecondaryKeys::Name(UserName(self.name.clone())),
            UserSecondaryKeys::Age(UserAge(self.age)),
        ]
    }

    fn get_relational_keys<'a>(&'a self) -> Vec<UserRelationalKeys> {
        vec![UserRelationalKeys::Partner(UserPartner(
            self.partner.clone(),
        ))]
    }
}

impl StoreValueMarker for User {}
impl StoreValueMarker for UserID {}

impl StoreValue<Definition, UserID> for User {}
impl StoreKey<Definition, User> for UserID {}

impl StoreKey<Definition, UserID> for UserSecondaryKeys {}
impl StoreKey<Definition, UserID> for UserRelationalKeys {}

impl StoreValue<Definition, UserSecondaryKeys> for UserID {}
impl StoreValue<Definition, UserRelationalKeys> for UserID {}

impl NetabaseModelMarker for User {}

impl NetabaseModelKeys<Definition, User> for UserKeys {
    type Primary<'a> = UserID;
    type Secondary<'a> = UserSecondaryKeys;
    type Relational<'a> = UserRelationalKeys;
}

impl<'a> NetabaseModelPrimaryKey<'a, Definition, User, UserKeys> for UserID {}
impl<'a> NetabaseModelSecondaryKey<'a, Definition, User, UserKeys> for UserSecondaryKeys {}
impl<'a> NetabaseModelRelationalKey<'a, Definition, User, UserKeys> for UserRelationalKeys {}

// --- Post Implementation ---

impl NetabaseModel<Definition> for Post {
    type Keys = PostKeys;

    fn get_primary_key<'a>(&'a self) -> PostID {
        self.id.clone()
    }

    fn get_secondary_keys<'a>(&'a self) -> Vec<PostSecondaryKeys> {
        vec![PostSecondaryKeys::Title(PostTitle(self.title.clone()))]
    }

    fn get_relational_keys<'a>(&'a self) -> Vec<PostRelationalKeys> {
        vec![PostRelationalKeys::Author(self.author.clone())]
    }
}

impl StoreValueMarker for Post {}
impl StoreValueMarker for PostID {}

impl StoreValue<Definition, PostID> for Post {}
impl StoreKey<Definition, Post> for PostID {}

impl StoreKey<Definition, PostID> for PostSecondaryKeys {}
impl StoreKey<Definition, PostID> for PostRelationalKeys {}

impl StoreValue<Definition, PostSecondaryKeys> for PostID {}
impl StoreValue<Definition, PostRelationalKeys> for PostID {}

impl NetabaseModelMarker for Post {}

impl NetabaseModelKeys<Definition, Post> for PostKeys {
    type Primary<'a> = PostID;
    type Secondary<'a> = PostSecondaryKeys;
    type Relational<'a> = PostRelationalKeys;
}

impl<'a> NetabaseModelPrimaryKey<'a, Definition, Post, PostKeys> for PostID {}
impl<'a> NetabaseModelSecondaryKey<'a, Definition, Post, PostKeys> for PostSecondaryKeys {}
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

impl_redb_value_key_for_owned!(User);
impl_redb_value_key_for_owned!(UserSecondaryKeys);
impl_redb_value_key_for_owned!(UserRelationalKeys);
impl_redb_value_key_for_owned!(Post);
impl_redb_value_key_for_owned!(PostSecondaryKeys);
impl_redb_value_key_for_owned!(PostRelationalKeys);

// RedbNetbaseModel impls - using constant table names with slices
impl<'db> RedbNetbaseModel<'db, Definition> for User {
    const TREE_NAMES: ModelTreeNames<'db, Definition, Self> = ModelTreeNames {
        main: DiscriminantTableName::new(DefinitionDiscriminants::User, "User:User:Primary:Main"),
        secondary: &[
            DiscriminantTableName::new(
                UserSecondaryKeysDiscriminants::Name,
                "Defintion:User:Secondary:Name",
            ),
            DiscriminantTableName::new(
                UserSecondaryKeysDiscriminants::Age,
                "Defintion:User:Secondary:Age",
            ),
        ],
        relational: &[DiscriminantTableName::new(
            UserRelationalKeysDiscriminants::Partner,
            "Definition:User:Relational:Partner",
        )],
    };
}

impl<'db> RedbNetbaseModel<'db, Definition> for Post {
    const TREE_NAMES: ModelTreeNames<'db, Definition, Self> = ModelTreeNames {
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
    };
}

fn main() {
    println!("Testing NetabaseStore type system");

    // Test data creation
    let user_id = UserID("user1".to_string());
    let user = User {
        id: user_id.clone(),
        name: "Alice".to_string(),
        age: 30,
        partner: user_id.clone(),
    };

    let post_id = PostID("post1".to_string());
    let post = Post {
        id: post_id.clone(),
        title: "Hello World".to_string(),
        author: user_id.clone(),
    };

    println!("Created User: {:?}", user);
    println!("Created Post: {:?}", post);

    // Test primary keys
    println!("User primary key: {:?}", user.get_primary_key());
    println!("Post primary key: {:?}", post.get_primary_key());

    // Test secondary keys
    println!("User secondary keys: {:?}", user.get_secondary_keys());
    println!("Post secondary keys: {:?}", post.get_secondary_keys());

    // Test relational keys
    println!("User relational keys: {:?}", user.get_relational_keys());
    println!("Post relational keys: {:?}", post.get_relational_keys());

    // Test discriminants
    println!(
        "User discriminant: {:?}",
        DefinitionDiscriminants::User.as_ref()
    );
    println!(
        "Post discriminant: {:?}",
        DefinitionDiscriminants::Post.as_ref()
    );

    // Test tree names structure with formatted table names
    println!("User main table: {}", User::TREE_NAMES.main.table_name);
    println!("User secondary tables:");
    for sec in User::TREE_NAMES.secondary {
        println!("  - {} -> {}", sec.discriminant.as_ref(), sec.table_name);
    }
    println!("User relational tables:");
    for rel in User::TREE_NAMES.relational {
        println!("  - {} -> {}", rel.discriminant.as_ref(), rel.table_name);
    }

    println!("Post main table: {}", Post::TREE_NAMES.main.table_name);
    println!("Post secondary tables:");
    for sec in Post::TREE_NAMES.secondary {
        println!("  - {} -> {}", sec.discriminant.as_ref(), sec.table_name);
    }
    println!("Post relational tables:");
    for rel in Post::TREE_NAMES.relational {
        println!("  - {} -> {}", rel.discriminant.as_ref(), rel.table_name);
    }

    println!("Type system test completed successfully!");
}
