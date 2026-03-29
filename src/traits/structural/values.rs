pub trait StoreValue: Sized {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobValue(pub &'static [u8]);
impl StoreValue for BlobValue {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ModelHash<M>(pub [u8; 32], std::marker::PhantomData<M>);

impl<M> ModelHash<M> {
    pub const fn new(hash: [u8; 32]) -> Self {
        Self(hash, std::marker::PhantomData)
    }
}

impl<M> StoreValue for ModelHash<M> {}
