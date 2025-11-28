//! Shared types for Redb backend
//!
//! This module contains wrapper types for bincode serialization with redb.

use redb::{Key, TypeName, Value};
use std::cmp::Ordering;
use std::fmt::Debug;

/// Wrapper type for bincode serialization with redb
///
/// This implements redb's Key and Value traits for any type that supports bincode.
#[derive(Debug, Clone)]
pub struct BincodeWrapper<T>(pub T);

impl<T> Value for BincodeWrapper<T>
where
    T: Debug + bincode::Encode + bincode::Decode<()>,
{
    type SelfType<'a>
        = T
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
        bincode::decode_from_slice(data, bincode::config::standard())
            .unwrap()
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::encode_to_vec(value, bincode::config::standard()).unwrap()
    }

    fn type_name() -> TypeName {
        TypeName::new(&format!("BincodeWrapper<{}>", std::any::type_name::<T>()))
    }
}

impl<T> Key for BincodeWrapper<T>
where
    T: Debug + bincode::Encode + bincode::Decode<()> + Ord,
{
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        Self::from_bytes(data1).cmp(&Self::from_bytes(data2))
    }
}

impl<T> std::borrow::Borrow<T> for BincodeWrapper<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

/// Composite key type for secondary index lookups.
///
/// This type combines a secondary key with a primary key for efficient secondary index operations.
/// Unlike tuples, this implements redb's Key and Value traits directly with proper borrowing semantics.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, bincode::Encode, bincode::Decode)]
pub struct CompositeKey<S, P> {
    pub secondary: S,
    pub primary: P,
}

impl<S, P> CompositeKey<S, P> {
    pub fn new(secondary: S, primary: P) -> Self {
        Self { secondary, primary }
    }
}

impl<S, P> Value for CompositeKey<S, P>
where
    S: Debug + bincode::Encode + bincode::Decode<()> + Clone,
    P: Debug + bincode::Encode + bincode::Decode<()> + Clone,
{
    type SelfType<'a>
        = CompositeKey<S, P>
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
        bincode::decode_from_slice(data, bincode::config::standard())
            .unwrap()
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::encode_to_vec(value, bincode::config::standard()).unwrap()
    }

    fn type_name() -> TypeName {
        TypeName::new(&format!(
            "CompositeKey<{}, {}>",
            std::any::type_name::<S>(),
            std::any::type_name::<P>()
        ))
    }
}

impl<S, P> Key for CompositeKey<S, P>
where
    S: Debug + bincode::Encode + bincode::Decode<()> + Clone + Ord,
    P: Debug + bincode::Encode + bincode::Decode<()> + Clone + Ord,
{
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        Self::from_bytes(data1).cmp(&Self::from_bytes(data2))
    }
}
