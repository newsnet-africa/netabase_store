use std::borrow::Cow;

use libp2p::kad::{Record, RecordKey};

use crate::{
    error::EncodingDecodingError,
    traits::{
        definition::{NetabaseDefinition, NetabaseDefinitionKeys},
        store::Store,
    },
};

pub trait KademilaRecord: NetabaseDefinition + bincode::Encode + bincode::Decode<()> {
    type NetabaseRecordKey: KademliaRecordKey;

    fn record_keys(&self) -> Self::NetabaseRecordKey;

    fn try_to_vec(&self) -> Result<Vec<u8>, bincode::error::EncodeError> {
        bincode::encode_to_vec(self, bincode::config::standard())
    }
    fn try_from_vec<V: AsRef<[u8]>>(vec: V) -> Result<Self, bincode::error::DecodeError> {
        Ok(bincode::decode_from_slice(vec.as_ref(), bincode::config::standard())?.0)
    }
    fn try_to_record(&self) -> Result<Record, EncodingDecodingError> {
        Ok(Record {
            key: self.record_keys().try_to_record_key()?,
            value: self.try_to_vec()?,
            publisher: None,
            expires: None,
        })
    }
    fn try_from_record(record: Record) -> Result<Self, EncodingDecodingError> {
        Ok(Self::try_from_vec(record.value)?)
    }
}

pub trait KademliaRecordKey:
    NetabaseDefinitionKeys + bincode::Encode + bincode::Decode<()>
{
    fn try_to_vec(&self) -> Result<Vec<u8>, bincode::error::EncodeError> {
        bincode::encode_to_vec(self, bincode::config::standard())
    }
    fn try_from_vec<V: AsRef<[u8]>>(vec: V) -> Result<Self, bincode::error::DecodeError> {
        Ok(bincode::decode_from_slice(vec.as_ref(), bincode::config::standard())?.0)
    }
    fn try_to_record_key(&self) -> Result<RecordKey, EncodingDecodingError> {
        Ok(RecordKey::new(&self.try_to_vec()?))
    }
    fn try_from_record_key(record: Record) -> Result<Self, EncodingDecodingError> {
        Ok(Self::try_from_vec(record.value)?)
    }
}
