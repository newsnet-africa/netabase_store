use crate::boilerplate_lib::models::user::User;
use crate::{Definition, DefinitionSubscriptions};
use netabase_store::blob::NetabaseBlobItem;
use netabase_store::databases::redb::transaction::ModelOpenTables;
use netabase_store::relational::RelationalLink;
use netabase_store::traits::registery::models::{
    StoreKey, StoreKeyMarker, StoreValue, StoreValueMarker,
    keys::{
        NetabaseModelKeys, NetabaseModelPrimaryKey, NetabaseModelRelationalKey,
        NetabaseModelSecondaryKey, NetabaseModelSubscriptionKey, blob::NetabaseModelBlobKey,
    },
    model::{NetabaseModel, NetabaseModelMarker, RedbNetbaseModel},
    treenames::{DiscriminantTableName, ModelTreeNames},
};
use bincode::{BorrowDecode, Decode, Encode};
use derive_more::Display;
use redb::{Key, Value};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;
use strum::{AsRefStr, EnumDiscriminants};

// --- Helpers ---
macro_rules! impl_redb_value_key_for_owned {
    ($type:ty) => {
        impl Value for $type {
            type SelfType<'a> = $type;
            type AsBytes<'a> = Cow<'a, [u8]>;
            fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> where Self: 'a {
                bincode::decode_from_slice(data, bincode::config::standard()).unwrap().0
            }
            fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> where Self: 'a {
                Cow::Owned(bincode::encode_to_vec(value, bincode::config::standard()).unwrap())
            }
            fn fixed_width() -> Option<usize> { None }
            fn type_name() -> redb::TypeName { redb::TypeName::new(std::any::type_name::<$type>()) }
        }
        impl Key for $type {
            fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering { data1.cmp(data2) }
        }
    };
}

// --- Heavy Model ---

#[derive(Debug, Clone, Encode, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HeavyModel {
    pub id: HeavyID,
    
    // Standard fields for basic load
    pub name: String,
    pub title: String,
    
    // Secondary Key fields
    pub category_label: String, // Secondary: Category
    pub score: u64,             // Secondary: Score

    // Relationships (Hydration testing)
    pub creator: RelationalLink<'static, Definition, Definition, User>,
    pub related_heavy: RelationalLink<'static, Definition, Definition, HeavyModel>,

    // Subscriptions
    pub subscriptions: Vec<DefinitionSubscriptions>,

    // Blobs (Large payload testing)
    pub attachment: HeavyAttachment,
    
    // Extra payload to ensure base object size is non-trivial
    pub matrix: Vec<u64>,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct HeavyAttachment {
    pub mime_type: String,
    pub data: Vec<u8>,
}

// Custom Deserialize for HeavyModel to handle 'static RelationalLinks
impl<'de> serde::Deserialize<'de> for HeavyModel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Id,
            Name,
            Title,
            CategoryLabel,
            Score,
            Creator,
            RelatedHeavy,
            Subscriptions,
            Attachment,
            Matrix,
        }

        struct HeavyModelVisitor;

        impl<'de> serde::de::Visitor<'de> for HeavyModelVisitor {
            type Value = HeavyModel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct HeavyModel")
            }

            fn visit_map<V>(self, mut map: V) -> Result<HeavyModel, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut name = None;
                let mut title = None;
                let mut category_label = None;
                let mut score = None;
                let mut creator = None;
                let mut related_heavy = None;
                let mut subscriptions = None;
                let mut attachment = None;
                let mut matrix = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => { id = Some(map.next_value()?); }
                        Field::Name => { name = Some(map.next_value()?); }
                        Field::Title => { title = Some(map.next_value()?); }
                        Field::CategoryLabel => { category_label = Some(map.next_value()?); }
                        Field::Score => { score = Some(map.next_value()?); }
                        Field::Creator => { creator = Some(map.next_value()?); }
                        Field::RelatedHeavy => { related_heavy = Some(map.next_value()?); }
                        Field::Subscriptions => { subscriptions = Some(map.next_value()?); }
                        Field::Attachment => { attachment = Some(map.next_value()?); }
                        Field::Matrix => { matrix = Some(map.next_value()?); }
                    }
                }

                Ok(HeavyModel {
                    id: id.ok_or_else(|| serde::de::Error::missing_field("id"))?,
                    name: name.ok_or_else(|| serde::de::Error::missing_field("name"))?,
                    title: title.ok_or_else(|| serde::de::Error::missing_field("title"))?,
                    category_label: category_label.ok_or_else(|| serde::de::Error::missing_field("category_label"))?,
                    score: score.ok_or_else(|| serde::de::Error::missing_field("score"))?,
                    creator: creator.ok_or_else(|| serde::de::Error::missing_field("creator"))?,
                    related_heavy: related_heavy.ok_or_else(|| serde::de::Error::missing_field("related_heavy"))?,
                    subscriptions: subscriptions.ok_or_else(|| serde::de::Error::missing_field("subscriptions"))?,
                    attachment: attachment.unwrap_or_default(),
                    matrix: matrix.unwrap_or_default(),
                })
            }
        }

        const FIELDS: &[&str] = &[
            "id", "name", "title", "category_label", "score", "creator", "related_heavy", "subscriptions", "attachment", "matrix"
        ];
        deserializer.deserialize_struct("HeavyModel", FIELDS, HeavyModelVisitor)
    }
}

// Manual Decode needed due to RelationalLink lifetimes
impl Decode<()> for HeavyModel {
    fn decode<D: bincode::de::Decoder<Context = ()>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            id: HeavyID::decode(decoder)?,
            name: String::decode(decoder)?,
            title: String::decode(decoder)?,
            category_label: String::decode(decoder)?,
            score: u64::decode(decoder)?,
            creator: RelationalLink::decode(decoder)?,
            related_heavy: RelationalLink::decode(decoder)?,
            subscriptions: Vec::decode(decoder)?,
            attachment: HeavyAttachment::decode(decoder)?,
            matrix: Vec::decode(decoder)?,
        })
    }
}

impl<'de> BorrowDecode<'de, ()> for HeavyModel {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = ()>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        // Reuse decode since we don't strictly need borrowing for the strings here for simplicity
        Self::decode(decoder)
    }
}


#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash, Display)]
pub struct HeavyID(pub String);

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash, Display)]
pub struct HeavyCategory(pub String);

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash, Display)]
pub struct HeavyScore(pub u64);

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash, Display)]
pub struct HeavyCreator(pub crate::boilerplate_lib::models::user::UserID);

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, Serialize, Deserialize, Hash, Display)]
pub struct HeavyRelation(pub HeavyID);


#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, EnumDiscriminants, Serialize, Deserialize, Hash)]
#[strum_discriminants(name(HeavySecondaryKeysDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum HeavySecondaryKeys {
    Category(HeavyCategory),
    Score(HeavyScore),
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, EnumDiscriminants, Serialize, Deserialize, Hash)]
#[strum_discriminants(name(HeavyRelationalKeysDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum HeavyRelationalKeys {
    Creator(HeavyCreator),
    RelatedHeavy(HeavyRelation),
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, EnumDiscriminants, Serialize, Deserialize, Hash)]
#[strum_discriminants(name(HeavySubscriptionKeysDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum HeavySubscriptionKeys {
    Topic1(DefinitionSubscriptions),
    Topic2(DefinitionSubscriptions),
    Topic3(DefinitionSubscriptions),
    Topic4(DefinitionSubscriptions),
}

impl NetabaseModelSubscriptionKey<Definition, HeavyModel, HeavyKeys> for HeavySubscriptionKeys {}

// Map from generic DefinitionSubscriptions to HeavySubscriptionKeys
impl From<DefinitionSubscriptions> for HeavySubscriptionKeys {
    fn from(value: DefinitionSubscriptions) -> Self {
        match value {
            DefinitionSubscriptions::Topic1 => HeavySubscriptionKeys::Topic1(value),
            DefinitionSubscriptions::Topic2 => HeavySubscriptionKeys::Topic2(value),
            DefinitionSubscriptions::Topic3 => HeavySubscriptionKeys::Topic3(value),
            DefinitionSubscriptions::Topic4 => HeavySubscriptionKeys::Topic4(value),
        }
    }
}

impl TryInto<DefinitionSubscriptions> for HeavySubscriptionKeys {
    type Error = ();
    fn try_into(self) -> Result<DefinitionSubscriptions, Self::Error> {
        match self {
            HeavySubscriptionKeys::Topic1(s) => Ok(s),
            HeavySubscriptionKeys::Topic2(s) => Ok(s),
            HeavySubscriptionKeys::Topic3(s) => Ok(s),
            HeavySubscriptionKeys::Topic4(s) => Ok(s),
        }
    }
}


#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Encode, Decode, EnumDiscriminants, Serialize, Deserialize, Hash)]
#[strum_discriminants(name(HeavyBlobKeysDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum HeavyBlobKeys {
    Attachment { owner: HeavyID },
}

impl<'a> NetabaseModelBlobKey<'a, Definition, HeavyModel, HeavyKeys> for HeavyBlobKeys {
    type PrimaryKey = HeavyID;
    type BlobItem = HeavyBlobItem;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, Serialize, Deserialize)]
pub enum HeavyBlobItem {
    Attachment{ index: u8, value: Vec<u8> },
}

impl NetabaseBlobItem for HeavyAttachment {
    type Blobs = HeavyBlobItem;
    fn wrap_blob(index: u8, data: Vec<u8>) -> Self::Blobs {
        HeavyBlobItem::Attachment { index, value: data }
    }
    fn unwrap_blob(blob: &Self::Blobs) -> Option<(u8, Vec<u8>)> {
        if let HeavyBlobItem::Attachment { index, value } = blob {
            Some((*index, value.clone()))
        } else {
            None
        }
    }
}

impl NetabaseBlobItem for HeavyBlobItem {
    type Blobs = Self;
    fn wrap_blob(_index: u8, _data: Vec<u8>) -> Self::Blobs { unimplemented!() }
    fn unwrap_blob(_blob: &Self::Blobs) -> Option<(u8, Vec<u8>)> { unimplemented!() }
    fn split_into_blobs(&self) -> Vec<Self::Blobs> { vec![self.clone()] }
    fn reconstruct_from_blobs(blobs: Vec<Self::Blobs>) -> Self { blobs.into_iter().next().unwrap() }
}


#[derive(Clone, Debug)]
pub enum HeavyKeys {
    Primary(HeavyID),
    Secondary(HeavySecondaryKeys),
    Relational(HeavyRelationalKeys),
    Subscription(HeavySubscriptionKeys),
    Blob(HeavyBlobKeys),
}

// --- Implementation ---

impl NetabaseModel<Definition> for HeavyModel {
    type Keys = HeavyKeys;

    const TREE_NAMES: ModelTreeNames<'static, Definition, Self> = ModelTreeNames {
        main: DiscriminantTableName::new(crate::DefinitionDiscriminants::Heavy, "Definition:Heavy:Primary:Main"),
        secondary: &[
            DiscriminantTableName::new(
                HeavySecondaryKeysDiscriminants::Category,
                "Definition:Heavy:Secondary:Category",
            ),
            DiscriminantTableName::new(
                HeavySecondaryKeysDiscriminants::Score,
                "Definition:Heavy:Secondary:Score",
            ),
        ],
        relational: &[
            DiscriminantTableName::new(
                HeavyRelationalKeysDiscriminants::Creator,
                "Definition:Heavy:Relational:Creator",
            ),
            DiscriminantTableName::new(
                HeavyRelationalKeysDiscriminants::RelatedHeavy,
                "Definition:Heavy:Relational:RelatedHeavy",
            ),
        ],
        subscription: Some(&[
            DiscriminantTableName::new(HeavySubscriptionKeysDiscriminants::Topic1, "Definition:Subscription:Heavy:Topic1"),
            DiscriminantTableName::new(HeavySubscriptionKeysDiscriminants::Topic2, "Definition:Subscription:Heavy:Topic2"),
            DiscriminantTableName::new(HeavySubscriptionKeysDiscriminants::Topic3, "Definition:Subscription:Heavy:Topic3"),
            DiscriminantTableName::new(HeavySubscriptionKeysDiscriminants::Topic4, "Definition:Subscription:Heavy:Topic4"),
        ]),
        blob: &[
            DiscriminantTableName::new(
                HeavyBlobKeysDiscriminants::Attachment,
                "Definition:Heavy:Blob:Attachment",
            ),
        ],
    };

    fn get_primary_key<'b>(&'b self) -> HeavyID {
        self.id.clone()
    }

    fn get_secondary_keys<'b>(&'b self) -> Vec<HeavySecondaryKeys> {
        vec![
            HeavySecondaryKeys::Category(HeavyCategory(self.category_label.clone())),
            HeavySecondaryKeys::Score(HeavyScore(self.score)),
        ]
    }

    fn get_relational_keys<'b>(&'b self) -> Vec<HeavyRelationalKeys> {
        vec![
            HeavyRelationalKeys::Creator(HeavyCreator(self.creator.get_primary_key().clone())),
            HeavyRelationalKeys::RelatedHeavy(HeavyRelation(self.related_heavy.get_primary_key().clone())),
        ]
    }

    fn get_subscription_keys<'b>(&'b self) -> Vec<HeavySubscriptionKeys> {
        self.subscriptions.iter().map(|s| s.clone().into()).collect()
    }

    fn get_blob_entries<'a>(&'a self) -> Vec<Vec<(
        <Self::Keys as NetabaseModelKeys<Definition, Self>>::Blob<'a>,
        <<Self::Keys as NetabaseModelKeys<Definition, Self>>::Blob<'a> as NetabaseModelBlobKey<'a, Definition, Self, Self::Keys>>::BlobItem,
    )>> {
        let mut attachment_entries = Vec::new();
        for blob in self.attachment.split_into_blobs() {
            attachment_entries.push((
                HeavyBlobKeys::Attachment { owner: self.id.clone() },
                blob
            ));
        }
        vec![attachment_entries]
    }
}

// Boilerplate marker impls
impl StoreValueMarker<Definition> for HeavyModel {}
impl StoreValueMarker<Definition> for HeavyID {}
impl StoreKeyMarker<Definition> for HeavyID {}
impl StoreKey<Definition, HeavyModel> for HeavyID {}
impl StoreValue<Definition, HeavyID> for HeavyModel {}

impl StoreKeyMarker<Definition> for HeavySecondaryKeys {}
impl StoreKeyMarker<Definition> for HeavyRelationalKeys {}
impl StoreKeyMarker<Definition> for HeavySubscriptionKeys {}
impl StoreKeyMarker<Definition> for HeavyBlobKeys {}
impl StoreKeyMarker<Definition> for HeavyBlobItem {}

impl StoreKey<Definition, HeavyID> for HeavySecondaryKeys {}
impl StoreKey<Definition, HeavyID> for HeavyRelationalKeys {}
impl StoreKey<Definition, HeavyID> for HeavySubscriptionKeys {}

impl StoreValue<Definition, HeavySecondaryKeys> for HeavyID {}
impl StoreValue<Definition, HeavyRelationalKeys> for HeavyID {}
impl StoreValue<Definition, HeavySubscriptionKeys> for HeavyID {}

impl NetabaseModelMarker<Definition> for HeavyModel {}

impl NetabaseModelKeys<Definition, HeavyModel> for HeavyKeys {
    type Primary<'a> = HeavyID;
    type Secondary<'a> = HeavySecondaryKeys;
    type Relational<'a> = HeavyRelationalKeys;
    type Subscription<'a> = HeavySubscriptionKeys;
    type Blob<'a> = HeavyBlobKeys;
}

impl<'a> NetabaseModelPrimaryKey<'a, Definition, HeavyModel, HeavyKeys> for HeavyID {}
impl<'a> NetabaseModelSecondaryKey<'a, Definition, HeavyModel, HeavyKeys> for HeavySecondaryKeys {
    type PrimaryKey = HeavyID;
}
impl<'a> NetabaseModelRelationalKey<'a, Definition, HeavyModel, HeavyKeys> for HeavyRelationalKeys {}


// Redb Value/Key impls
impl Value for HeavyModel {
    type SelfType<'a> = HeavyModel;
    type AsBytes<'a> = Cow<'a, [u8]>;
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> where Self: 'a {
        bincode::decode_from_slice(data, bincode::config::standard()).unwrap().0
    }
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> where Self: 'a {
        Cow::Owned(bincode::encode_to_vec(value, bincode::config::standard()).unwrap())
    }
    fn fixed_width() -> Option<usize> { None }
    fn type_name() -> redb::TypeName { redb::TypeName::new(std::any::type_name::<Self>()) }
}
impl Key for HeavyModel {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering { data1.cmp(data2) }
}

impl Value for HeavyID {
    type SelfType<'a> = Self;
    type AsBytes<'a> = Cow<'a, [u8]>;
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> where Self: 'a {
        HeavyID(String::from_utf8(data.to_vec()).unwrap())
    }
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> where Self: 'a {
        Cow::Owned(value.0.as_bytes().to_vec())
    }
    fn fixed_width() -> Option<usize> { None }
    fn type_name() -> redb::TypeName { redb::TypeName::new(std::any::type_name::<Self>()) }
}
impl Key for HeavyID {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering { data1.cmp(data2) }
}

impl_redb_value_key_for_owned!(HeavySecondaryKeys);
impl_redb_value_key_for_owned!(HeavyRelationalKeys);
impl_redb_value_key_for_owned!(HeavySubscriptionKeys);
impl_redb_value_key_for_owned!(HeavyBlobKeys);
impl_redb_value_key_for_owned!(HeavyBlobItem);

impl<'db> RedbNetbaseModel<'db, Definition> for HeavyModel {
    type RedbTables = ModelOpenTables<'db, 'db, Definition, Self>;
    type TableV = HeavyModel;
}