///! Database backend implementations for Netabase Store.
///!
///! This module contains implementations of various storage backends, each with different
///! performance characteristics and use cases.
///!
///! ## Available Backends
///!
///! ### Native Backends (for desktop/server applications)
///!
///! #### Redb (`redb_store`)
///! - **Best for**: Write-heavy workloads, ACID guarantees
///! - **Features**: MVCC transactions, excellent write performance, efficient storage
///! - **API Options**:
///!   - Standard wrapper: Simple API with auto-commit per operation
///!   - Bulk methods: `put_many()`, `get_many()`, `get_many_by_secondary_keys()`
///!   - ZeroCopy: Explicit transaction management for maximum performance
///! - **Overhead**: 118-133% for bulk operations vs raw redb
///! - **Enable with**: `features = ["redb"]`
///!
///! Example:
///! ```rust,no_run
///! # use netabase_store::databases::redb_store::RedbStore;
///! # fn example() -> Result<(), Box<dyn std::error::Error>> {
///! # let models = vec![];
///! let store = RedbStore::<MyDefinition>::new("./database.redb")?;
///! let tree = store.open_tree::<MyModel>();
///!
///! // Bulk insert - 8-9x faster than loop
///! tree.put_many(models)?;
///! # Ok(())
///! # }
///! ```
///!
///! #### Redb ZeroCopy (`redb_zerocopy`)
///! - **Best for**: High-performance scenarios requiring explicit control
///! - **Features**: Direct transaction management, zero-copy reads where possible
///! - **Performance**: Up to 54x faster for secondary key queries
///! - **Complexity**: Requires manual transaction management
///! - **Enable with**: `features = ["redb", "redb-zerocopy"]`
///!
///! Example:
///! ```rust,no_run
///! # use netabase_store::databases::redb_zerocopy::{RedbStoreZeroCopy, with_write_transaction};
///! # fn example() -> Result<(), Box<dyn std::error::Error>> {
///! # let models = vec![];
///! let store = RedbStoreZeroCopy::<MyDefinition>::new("./database.redb")?;
///!
///! with_write_transaction(&store, |txn| {
///!     let mut tree = txn.open_tree::<MyModel>()?;
///!     tree.put_many(models)?;
///!     Ok(())
///! })?;
///! # Ok(())
///! # }
///! ```
///!
///! #### Sled (`sled_store`)
///! - **Best for**: Read-heavy workloads
///! - **Features**: Battle-tested, very low read overhead (~20%)
///! - **Performance**: Excellent for read-heavy applications
///! - **Enable with**: `features = ["sled"]`
///!
///! Example:
///! ```rust,no_run
///! # use netabase_store::databases::sled_store::SledStore;
///! # fn example() -> Result<(), Box<dyn std::error::Error>> {
///! let store = SledStore::<MyDefinition>::new("./database")?;
///! let tree = store.open_tree::<MyModel>();
///! # Ok(())
///! # }
///! ```
///!
///! ### WASM Backend (for browser applications)
///!
///! #### IndexedDB (`indexeddb_store`)
///! - **Best for**: Browser-based applications
///! - **Features**: Async API, browser-native storage
///! - **Note**: All operations are async
///! - **Enable with**: `features = ["wasm"]` on wasm32 targets
///!
///! Example:
///! ```rust,no_run
///! # use netabase_store::databases::indexeddb_store::IndexedDBStore;
///! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
///! # let model = todo!();
///! let store = IndexedDBStore::<MyDefinition>::new("my_db").await?;
///! let tree = store.open_tree::<MyModel>();
///! tree.put(model).await?;
///! # Ok(())
///! # }
///! ```
///!
///! ### Testing Backend
///!
///! #### Memory Store (`memory_store`)
///! - **Best for**: Unit tests, caching
///! - **Features**: No I/O, no cleanup needed, fast
///! - **Note**: Data is lost when dropped
///! - **Always available**: No feature flag needed
///!
///! Example:
///! ```rust,no_run
///! # use netabase_store::databases::memory_store::MemoryStore;
///! let store = MemoryStore::<MyDefinition>::new();
///! let tree = store.open_tree::<MyModel>();
///! ```
///!
///! ## Performance Comparison
///!
///! | Backend | Insert (1000 items) | Get (1000 items) | Best For |
///! |---------|-------------------|------------------|----------|
///! | Redb (bulk) | 3.10 ms | 382 µs | Write-heavy |
///! | Sled | ~4 ms | ~305 µs | Read-heavy |
///! | ZeroCopy | 3.51 ms | 692 µs | High-performance |
///! | Memory | <1 ms | <50 µs | Testing |
///!
///! ## Choosing a Backend
///!
///! 1. **For production applications**:
///!    - Write-heavy → `redb_store` with bulk methods
///!    - Read-heavy → `sled_store`
///!    - Need maximum performance → `redb_zerocopy`
///!
///! 2. **For browser applications**:
///!    - Use `indexeddb_store` (only option for WASM)
///!
///! 3. **For testing**:
///!    - Use `memory_store` for fast, isolated tests
///!
///! ## Bulk Methods vs Transactions
///!
///! All native backends support bulk operations for better performance:
///!
///! ```rust,ignore
///! // ❌ Slow: Creates 1000 transactions
///! for model in models {
///!     tree.put(model)?;
///! }
///!
///! // ✅ Fast: Single transaction
///! tree.put_many(models)?;  // 8-9x faster!
///! ```
///!
///! Available bulk methods:
///! - `put_many(Vec<M>)` - Bulk insert
///! - `get_many(Vec<M::Keys>)` - Bulk read
///! - `get_many_by_secondary_keys(Vec<SecondaryKey>)` - Bulk secondary queries

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
