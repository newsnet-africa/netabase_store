#[cfg(feature = "redb")]
pub mod redb_store;

#[cfg(feature = "sled")]
pub mod sled_store;

#[cfg(feature = "wasm")]
pub mod indexeddb_store;

// In-memory backend (always available)
pub mod memory_store;

// libp2p RecordStore implementation module
#[cfg(feature = "libp2p")]
pub mod record_store;
