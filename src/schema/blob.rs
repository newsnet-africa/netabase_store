//! Blob storage support for large data fields.
//!
//! This module provides types and traits for handling large binary data that needs
//! to be split into chunks for efficient storage and retrieval. Data larger than
//! 60KB is automatically chunked when stored.
//!
//! # Overview
//!
//! Blob fields in models are marked with the `#[blob]` attribute, and custom blob
//! types require the `NetabaseBlobItem` derive macro. The system handles:
//! - Automatic chunking of data > 60KB
//! - Efficient serialization with postcard
//! - Lazy loading via references
//! - Reconstruction from chunks
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                        Model Record                         │
//! │  ┌─────────────────────────────────────────────────────┐   │
//! │  │ BlobItem::Reference(key) ─────┐                      │   │
//! │  └─────────────────────────────────│─────────────────────┘   │
//! └──────────────────────────────────────│───────────────────────┘
//!                                        │
//!                                        ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Blob Table                              │
//! │  key_0 → [chunk 0: 60KB]                                     │
//! │  key_1 → [chunk 1: 60KB]                                     │
//! │  key_2 → [chunk 2: remaining bytes]                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Chunk Size
//!
//! Data is split into 60KB (60,000 byte) chunks to balance:
//! - Database page efficiency (fits within typical page sizes)
//! - Memory usage during reads (predictable allocation)
//! - Parallel fetch granularity (chunks can be fetched independently)
//!
//! # Using Blob Fields
//!
//! Mark large fields with `#[blob]` in your model:
//!
//! ```rust,no_run
//! #[derive(NetabaseModel)]
//! pub struct Document {
//!     #[primary_key]
//!     pub id: DocumentId,
//!     
//!     pub title: String,
//!     
//!     #[blob]
//!     pub content: LargeContent,  // Stored separately in chunks
//! }
//! ```
//!
//! # Custom Blob Types
//!
//! Create custom blob types with the `NetabaseBlobItem` derive:
//!
//! ```rust,no_run
//! use netabase_macros::NetabaseBlobItem;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(NetabaseBlobItem, Serialize, Deserialize, Clone)]
//! pub struct LargeFile {
//!     pub data: Vec<u8>,
//!     pub metadata: FileMetadata,
//! }
//! ```
//!
//! # The BlobItem Enum
//!
//! When reading a model, blob fields are wrapped in [`BlobItem`]:
//!
//! - `BlobItem::Full(data)` - The complete blob data (after explicit load)
//! - `BlobItem::Reference(key)` - Just a reference (default, lazy load)
//!
//! This allows efficient model reads without loading large blobs.
//!
//! # Performance Considerations
//!
//! - **Lazy loading**: Blobs are not loaded by default, reducing memory usage
//! - **Chunking**: Large data is stored efficiently across multiple records
//! - **Network-ready**: Chunk size is optimized for network transmission
//!
//! # Generated Tables and Keys
//!
//! When you place `#[blob]` on a field in a `NetabaseModel`, the macros
//! generate:
//!
//! - A `{Model}BlobKeys` enum implementing `BlobKey` for addressing chunks
//! - A `{Model}Blobs` helper type that owns `Vec<BlobChunk>` values
//! - Additional entries in the definition's `TreeNames` enum for the blob table
//! - Redb table definitions wired into `RedbTransaction` for that blob table
//!
//! This means every blob field corresponds to a **separate physical table**
//! keyed by a blob ID and chunk index, which is why blob operations are
//! isolated from regular row-level reads.

use serde::{Serialize, de::DeserializeOwned};

/// Link type for blob data, either complete or chunked.
///
/// This enum represents blob data in two states:
/// - `Complete`: The full blob item (used before storage)
/// - `Blobs`: A vec of blob chunks (used during storage/retrieval)
///
/// # Type Parameters
///
/// - `T`: The blob item type implementing [`NetabaseBlobItem`]
pub enum BlobLink<T: NetabaseBlobItem> {
    /// Complete blob item, not yet chunked.
    ///
    /// This variant is used when you have the full data in memory
    /// and want to store it.
    Complete(T),
    
    /// Blob split into chunks for storage.
    ///
    /// This variant is used during the storage process and when
    /// reconstructing the blob from stored chunks.
    Blobs(Vec<T::Blobs>),
}

/// A wrapper for blob items stored in models.
/// 
/// This enum allows a model to hold either the full blob data (loaded/new)
/// or a reference key to the blob stored separately. This enables lazy
/// loading of large data.
///
/// # Type Parameters
///
/// - `T`: The full blob data type
/// - `K`: The reference key type (typically a primary key + field identifier)
///
/// # Example
///
/// ```rust
/// use netabase_store::schema::blob::BlobItem;
///
/// // A blob that's been loaded
/// let loaded: BlobItem<Vec<u8>, String> = BlobItem::Full(vec![1, 2, 3]);
///
/// // A blob reference (not yet loaded)
/// let reference: BlobItem<Vec<u8>, String> = BlobItem::Reference("doc_123_content".into());
///
/// // Check if data is loaded
/// match loaded {
///     BlobItem::Full(data) => println!("Loaded {} bytes", data.len()),
///     BlobItem::Reference(key) => println!("Reference to {}", key),
/// }
/// ```
#[derive(Debug, Clone, Serialize, serde::Deserialize, PartialEq)]
pub enum BlobItem<T, K> {
    /// Full data of the blob item.
    ///
    /// Contains the complete blob data in memory. This variant is used
    /// when:
    /// - Creating a new model with blob data
    /// - After explicitly loading a blob from storage
    Full(T),
    
    /// Reference key to the stored blob.
    ///
    /// Contains only a key that can be used to load the blob data
    /// on demand. This variant is used when:
    /// - Reading a model without loading blobs
    /// - Storing a model (the data is stored separately)
    Reference(K),
}

impl<T, K> BlobItem<T, K> {
    /// Returns `true` if the blob data is loaded.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::schema::blob::BlobItem;
    ///
    /// let loaded: BlobItem<Vec<u8>, String> = BlobItem::Full(vec![1, 2, 3]);
    /// let reference: BlobItem<Vec<u8>, String> = BlobItem::Reference("key".into());
    ///
    /// assert!(loaded.is_full());
    /// assert!(!reference.is_full());
    /// ```
    pub fn is_full(&self) -> bool {
        matches!(self, BlobItem::Full(_))
    }

    /// Returns `true` if this is a reference (not loaded).
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::schema::blob::BlobItem;
    ///
    /// let reference: BlobItem<Vec<u8>, String> = BlobItem::Reference("key".into());
    /// assert!(reference.is_reference());
    /// ```
    pub fn is_reference(&self) -> bool {
        matches!(self, BlobItem::Reference(_))
    }

    /// Returns the full data if loaded, or `None` if it's a reference.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::schema::blob::BlobItem;
    ///
    /// let loaded: BlobItem<Vec<u8>, String> = BlobItem::Full(vec![1, 2, 3]);
    /// assert_eq!(loaded.as_full(), Some(&vec![1, 2, 3]));
    ///
    /// let reference: BlobItem<Vec<u8>, String> = BlobItem::Reference("key".into());
    /// assert_eq!(reference.as_full(), None);
    /// ```
    pub fn as_full(&self) -> Option<&T> {
        match self {
            BlobItem::Full(data) => Some(data),
            BlobItem::Reference(_) => None,
        }
    }

    /// Returns the reference key if not loaded, or `None` if full data is present.
    pub fn as_reference(&self) -> Option<&K> {
        match self {
            BlobItem::Full(_) => None,
            BlobItem::Reference(key) => Some(key),
        }
    }

    /// Consumes the BlobItem and returns the full data if loaded.
    ///
    /// # Returns
    ///
    /// - `Ok(T)` if the blob is fully loaded
    /// - `Err(K)` if only a reference is available
    pub fn into_full(self) -> Result<T, K> {
        match self {
            BlobItem::Full(data) => Ok(data),
            BlobItem::Reference(key) => Err(key),
        }
    }
}

/// Trait for types that can be stored as blobs with automatic chunking.
///
/// Types implementing this trait can be automatically split into 60KB chunks
/// for storage and reconstructed on retrieval. The trait is typically derived
/// using the `#[derive(NetabaseBlobItem)]` macro from `netabase_macros`.
///
/// # Chunk Size
///
/// Data is split into 60KB (60,000 byte) chunks to balance:
/// - Database page efficiency
/// - Memory usage during reads
/// - Parallel fetch granularity
///
/// # Serialization
///
/// Uses postcard for efficient binary serialization that's suitable for
/// network transmission in decentralized systems.
///
/// # Implementing Manually
///
/// While the derive macro handles most cases, you can implement manually:
///
/// ```rust,no_run
/// use netabase_store::schema::blob::NetabaseBlobItem;
///
/// struct MyBlob {
///     data: Vec<u8>,
/// }
///
/// // Define your chunk type
/// enum MyBlobChunks {
///     Data { index: u8, bytes: Vec<u8> },
/// }
///
/// impl NetabaseBlobItem for MyBlob {
///     type Blobs = MyBlobChunks;
///
///     fn split_into_blobs(&self) -> Vec<Self::Blobs> {
///         const CHUNK_SIZE: usize = 60_000;
///         self.data
///             .chunks(CHUNK_SIZE)
///             .enumerate()
///             .map(|(i, chunk)| MyBlobChunks::Data {
///                 index: i as u8,
///                 bytes: chunk.to_vec(),
///             })
///             .collect()
///     }
///
///     fn reconstruct_from_blobs(blobs: Vec<Self::Blobs>) -> Self {
///         let mut data = Vec::new();
///         for blob in blobs {
///             match blob {
///                 MyBlobChunks::Data { bytes, .. } => data.extend(bytes),
///             }
///         }
///         MyBlob { data }
///     }
/// }
/// ```
pub trait NetabaseBlobItem: Sized + Serialize + DeserializeOwned {
    /// The associated enum type for blob chunks.
    ///
    /// Generated by the macro, each variant represents a chunk.
    /// The chunk type should contain:
    /// - An index for ordering during reconstruction
    /// - The chunk data bytes
    type Blobs;

    /// Split this item into blob chunks.
    ///
    /// Serializes the item (or its parts) and splits into 60KB chunks.
    /// Each chunk is wrapped in the appropriate enum variant.
    ///
    /// # Returns
    ///
    /// A vector of blob chunks in order. The chunks can be stored
    /// independently and reassembled later.
    fn split_into_blobs(&self) -> Vec<Self::Blobs>;

    /// Reconstruct the original item from blob chunks.
    ///
    /// Takes a vector of blob chunks, sorts them by index (if applicable),
    /// concatenates the data, and deserializes back into the original item.
    ///
    /// # Arguments
    ///
    /// * `blobs` - Vector of blob chunks (may be out of order)
    ///
    /// # Returns
    ///
    /// The reconstructed item
    ///
    /// # Panics
    ///
    /// May panic if the chunks are corrupted or incomplete.
    fn reconstruct_from_blobs(blobs: Vec<Self::Blobs>) -> Self;

    /// Get the index of a blob chunk.
    ///
    /// Returns the index of this chunk if it represents a part of a split item.
    /// Returns None if the item is not chunked or if the index is not available.
    ///
    /// This is used during reconstruction to ensure chunks are assembled in
    /// the correct order.
    fn get_blob_index(&self) -> Option<u8> {
        None
    }
}