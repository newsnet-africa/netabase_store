#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkSize {
    Default,
    Size(usize),
}

pub trait NetabaseBlobItem:
    Sized + IntoIterator<Item = crate::results::NetabaseResult<Self::Chunk>, IntoIter = Self::BlobIter>
{
    type Chunk: BlobItemChunk;
    type BlobIter: Iterator<Item = crate::results::NetabaseResult<Self::Chunk>>;
    const DEFAULT_CHUNK_SIZE: usize;

    fn into_chunks(self, size: ChunkSize) -> Box<dyn Iterator<Item = Self::Chunk>>;

    fn into_chunks_iter(self, size: ChunkSize) -> Self::BlobIter;

    fn try_from_chunks(
        chunks: impl Iterator<Item = Self::Chunk>,
        size: ChunkSize,
    ) -> crate::results::NetabaseResult<Self>;

    fn get_blob_index(chunk: &Self::Chunk) -> &<Self::Chunk as BlobItemChunk>::Index {
        chunk.get_index()
    }

    fn get_blob(&self) -> &Self::Chunk;
}

pub trait BlobItemChunk {
    type Index: PartialEq + Eq + PartialOrd + Ord;

    fn get_index(&self) -> &Self::Index;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobItem<T: NetabaseBlobItem> {
    Whole(T),
    Chunk(T::Chunk),
}
