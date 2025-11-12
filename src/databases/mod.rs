#[cfg(feature = "redb")]
pub mod redb_store;

#[cfg(feature = "redb")]
pub mod redb_zerocopy;

#[cfg(feature = "sled")]
pub mod sled_store;

#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub mod indexeddb_store;

// In-memory backend (always available)
pub mod memory_store;

// libp2p RecordStore implementation module (native-only, requires mio/networking)
#[cfg(all(feature = "libp2p", not(target_arch = "wasm32")))]
pub mod record_store;
