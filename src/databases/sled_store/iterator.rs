use crate::error::NetabaseError;
use crate::traits::convert::ToIVec;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::NetabaseModelTrait;
use crate::{MaybeSend, MaybeSync, NetabaseModelTraitKey};
use std::marker::PhantomData;
use std::str::FromStr;
use strum::IntoDiscriminant;

/// Iterator over models in a SledStoreTree
pub struct SledIter<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    <D as IntoDiscriminant>::Discriminant: Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    pub(crate) inner: sled::Iter,
    pub(crate) _phantom_d: PhantomData<D>,
    pub(crate) _phantom_m: PhantomData<M>,
}

impl<D, M> Iterator for SledIter<D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec,
    M: NetabaseModelTrait<D> + TryFrom<D>,
    <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey: bincode::Decode<()>,
    <D as IntoDiscriminant>::Discriminant: Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    type Item = Result<(<M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey, M), NetabaseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|result| {
            result.map_err(|e| e.into()).and_then(|(k, v)| {
                let (key, _) = bincode::decode_from_slice::<
                    <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
                    _,
                >(&k, bincode::config::standard())
                .map_err(crate::error::EncodingDecodingError::from)?;
                let definition = D::from_ivec(&v)?;
                let model = M::try_from(definition).map_err(|_| {
                    crate::error::NetabaseError::Conversion(
                        crate::error::EncodingDecodingError::Decoding(
                            bincode::error::DecodeError::Other("Type conversion failed"),
                        ),
                    )
                })?;
                Ok((key, model))
            })
        })
    }
}
