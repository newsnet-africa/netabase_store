use crate::traits::registery::{
    definition::NetabaseDefinition,
    models::{StoreKey, StoreValue},
};

pub enum BlobLink<T: NetabaseBlobItem> {
    Complete(T),
    Blobs(Vec<T::Blobs>),
}

pub trait NetabaseBlobItem {
    type Blobs;
    fn split_into_blobs(&self) -> Vec<Self::Blobs>;
    fn reconstruct_from_blobs(blobs: Vec<Self::Blobs>) -> Self;
}
