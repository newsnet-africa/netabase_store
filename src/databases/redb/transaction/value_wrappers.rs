use serde::{Serialize, Deserialize};
use redb::{Value, Key, TypeName};
use std::borrow::Cow;
use std::cmp::Ordering;

#[cfg(feature = "libp2p")]
use libp2p::kad::ProviderRecord;
#[cfg(feature = "libp2p")]
use libp2p::{PeerId, Multiaddr};

#[cfg(feature = "libp2p")]
#[derive(Debug, Clone)]
pub struct Libp2pProviderRecordWrapper(pub ProviderRecord);

#[cfg(feature = "libp2p")]
#[derive(Serialize, Deserialize)]
struct ProviderRecordDto {
    key: Vec<u8>,
    provider: PeerId,
    // expires: Option<std::time::SystemTime>, // Expiration logic is complex, skipping for MVP prototype
    addresses: Vec<Multiaddr>,
}

#[cfg(feature = "libp2p")]
impl Serialize for Libp2pProviderRecordWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Assuming fields are public as is common in recent libp2p versions
        let dto = ProviderRecordDto {
            key: self.0.key.as_ref().to_vec(),
            provider: self.0.provider,
            addresses: self.0.addresses.clone(),
        };
        dto.serialize(serializer)
    }
}

#[cfg(feature = "libp2p")]
impl<'de> Deserialize<'de> for Libp2pProviderRecordWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let dto = ProviderRecordDto::deserialize(deserializer)?;
        let key = libp2p::kad::RecordKey::new(&dto.key);
        let record = ProviderRecord::new(key, dto.provider, dto.addresses);
        Ok(Libp2pProviderRecordWrapper(record))
    }
}

#[cfg(feature = "libp2p")]
impl Value for Libp2pProviderRecordWrapper {
    type SelfType<'a> = Libp2pProviderRecordWrapper;
    type AsBytes<'a> = Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        postcard::from_bytes(data).unwrap()
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        Cow::Owned(postcard::to_allocvec(value).unwrap())
    }

    fn type_name() -> TypeName {
        TypeName::new("Libp2pProviderRecordWrapper")
    }
}

#[cfg(feature = "libp2p")]
impl Key for Libp2pProviderRecordWrapper {
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        data1.cmp(data2)
    }
}