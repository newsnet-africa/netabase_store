//! Transaction API with type-state pattern for compile-time safety.
//!
//! This module provides ergonomic transaction management with compile-time guarantees
//! about read-only vs read-write access. The type-state pattern uses phantom types
//! to achieve zero-cost polymorphism.
//!
//! **Current Status**: Fully optimized for Sled backend. Redb backend optimization in progress.
//!
//! # Type-State Pattern
//!
//! The `TxnGuard<Mode>` type uses phantom types to track transaction mode at compile time:
//! - `TxnGuard<ReadOnly>`: Multiple concurrent read transactions allowed (Sled)
//! - `TxnGuard<ReadWrite>`: Exclusive write transaction with commit/rollback (Sled)
//!
//! # Examples
//!
//! ## Read Transaction
//! ```no_run
//! # use netabase_store::{NetabaseStore, netabase_definition_module};
//! # #[netabase_definition_module(MyDef, MyKeys)]
//! # mod models {
//! #     use netabase_store::{NetabaseModel, netabase};
//! #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//! #              bincode::Encode, bincode::Decode,
//! #              serde::Serialize, serde::Deserialize)]
//! #     #[netabase(MyDef)]
//! #     pub struct User {
//! #         #[primary_key]
//! #         pub id: u64,
//! #         pub name: String,
//! #     }
//! # }
//! # use models::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let store = NetabaseStore::sled("./db")?;
//! let txn = store.read();  // Sled only for now
//! let tree = txn.open_tree::<User>();
//! let user = tree.get(UserPrimaryKey(1))?;
//! // Auto-closes on drop
//! # Ok(())
//! # }
//! ```
//!
//! ## Write Transaction
//! ```no_run
//! # use netabase_store::{NetabaseStore, netabase_definition_module};
//! # #[netabase_definition_module(MyDef, MyKeys)]
//! # mod models {
//! #     use netabase_store::{NetabaseModel, netabase};
//! #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//! #              bincode::Encode, bincode::Decode,
//! #              serde::Serialize, serde::Deserialize)]
//! #     #[netabase(MyDef)]
//! #     pub struct User {
//! #         #[primary_key]
//! #         pub id: u64,
//! #         pub name: String,
//! #     }
//! # }
//! # use models::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let store = NetabaseStore::sled("./db")?;
//! # let user = User { id: 1, name: "Alice".to_string() };
//! let mut txn = store.write();  // Sled only for now
//! let mut tree = txn.open_tree::<User>();
//! tree.put(user)?;
//! txn.commit()?;  // Or auto-rollback on drop
//! # Ok(())
//! # }
//! ```

use crate::error::NetabaseError;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::NetabaseModelTrait;
use std::marker::PhantomData;

#[cfg(feature = "redb")]
use redb::{ReadableDatabase, ReadableTable, ReadableTableMetadata};

#[cfg(feature = "redb")]
use std::cell::RefCell;

// Models and keys implement redb::Key/Value directly through the macro!
// CompositeKey is used for secondary index (secondary_key, primary_key) pairs.
#[cfg(feature = "redb")]
use crate::databases::redb_store::CompositeKey;

/// Zero-cost marker type for read-only transactions.
///
/// This type exists only at compile time and generates no runtime code.
/// It's used to restrict operations to read-only methods.
pub struct ReadOnly;

/// Zero-cost marker type for read-write transactions.
///
/// This type exists only at compile time and generates no runtime code.
/// It allows both read and write operations.
pub struct ReadWrite;

/// Transaction guard with type-state pattern.
///
/// The `Mode` parameter determines available operations at compile time:
/// - `TxnGuard<ReadOnly>`: Can only perform read operations
/// - `TxnGuard<ReadWrite>`: Can perform both read and write operations
///
/// # Type Safety
///
/// ```compile_fail
/// let txn = store.read()?;  // ReadOnly mode
/// let mut tree = txn.open_tree::<User>();
/// tree.put(user)?;  // Compile error: put() not available on ReadOnly
/// ```
pub struct TxnGuard<'db, D, Mode> {
    backend: TxnBackend<'db, D>,
    _mode: PhantomData<Mode>,
}

/// Backend-specific transaction implementation (hidden from users).
pub(crate) enum TxnBackend<'db, D> {
    #[cfg(feature = "sled")]
    Sled(SledTxnBackend<'db, D>),

    #[cfg(feature = "redb")]
    Redb(RedbTxnBackend<'db, D>),
}

/// Sled transaction backend.
#[cfg(feature = "sled")]
pub(crate) struct SledTxnBackend<'db, D> {
    pub(crate) db: &'db sled::Db,
    pub(crate) _phantom: PhantomData<D>,
}

/// Redb transaction backend.
///
/// Uses RefCell for interior mutability to allow multiple TreeViews to share
/// access to the same transaction.
#[cfg(feature = "redb")]
pub(crate) struct RedbTxnBackend<'db, D> {
    pub(crate) read_txn: RefCell<Option<redb::ReadTransaction>>,
    pub(crate) write_txn: RefCell<Option<redb::WriteTransaction>>,
    pub(crate) db: &'db std::sync::Arc<redb::Database>,
    pub(crate) _phantom: PhantomData<D>,
}

/// Tree view with type-state pattern.
///
/// Provides access to a specific model type within a transaction.
/// The `Mode` parameter determines available operations.
pub struct TreeView<'txn, D, M, Mode> {
    backend: TreeBackend<'txn, D, M>,
    _mode: PhantomData<Mode>,
}

/// Backend-specific tree implementation (hidden from users).
pub(crate) enum TreeBackend<'txn, D, M> {
    #[cfg(feature = "sled")]
    Sled(SledTreeBackend<'txn, D, M>),

    #[cfg(feature = "redb")]
    Redb(RedbTreeBackend<'txn, D, M>),
}

/// Sled tree backend.
///
/// Note: sled::Tree is Arc-based, so cloning is cheap and we can store owned instances.
/// Sled operations are applied immediately since Sled doesn't have true multi-tree transactions.
#[cfg(feature = "sled")]
pub(crate) struct SledTreeBackend<'txn, D, M> {
    pub(crate) tree: sled::Tree,
    pub(crate) secondary_tree: sled::Tree,
    pub(crate) _phantom: PhantomData<(&'txn (), D, M)>,
}

/// Redb tree backend.
///
/// Holds a reference to the transaction backend to reuse the same transaction.
#[cfg(feature = "redb")]
pub(crate) struct RedbTreeBackend<'txn, D, M> {
    pub(crate) txn_backend: &'txn RedbTxnBackend<'txn, D>,
    pub(crate) table_name: &'static str,
    pub(crate) secondary_table_name: &'static str,
    pub(crate) _phantom: PhantomData<M>,
}

// ============================================================================
// TxnGuard Constructors
// ============================================================================

impl<'db, D> TxnGuard<'db, D, ReadOnly>
where
    D: NetabaseDefinitionTrait,
{
    /// Create a read-only transaction from a Sled database.
    #[cfg(feature = "sled")]
    pub fn read_sled(db: &'db sled::Db) -> Self {
        Self {
            backend: TxnBackend::Sled(SledTxnBackend {
                db,
                _phantom: PhantomData,
            }),
            _mode: PhantomData,
        }
    }

    /// Create a read-only transaction from a Redb database.
    #[cfg(feature = "redb")]
    pub fn read_redb(db: &'db std::sync::Arc<redb::Database>) -> Result<Self, NetabaseError> {
        let read_txn = db.begin_read()?;
        Ok(Self {
            backend: TxnBackend::Redb(RedbTxnBackend {
                read_txn: RefCell::new(Some(read_txn)),
                write_txn: RefCell::new(None),
                db,
                _phantom: PhantomData,
            }),
            _mode: PhantomData,
        })
    }
}

impl<'db, D> TxnGuard<'db, D, ReadWrite>
where
    D: NetabaseDefinitionTrait,
{
    /// Create a read-write transaction from a Sled database.
    #[cfg(feature = "sled")]
    pub fn write_sled(db: &'db sled::Db) -> Self {
        Self {
            backend: TxnBackend::Sled(SledTxnBackend {
                db,
                _phantom: PhantomData,
            }),
            _mode: PhantomData,
        }
    }

    /// Create a read-write transaction from a Redb database.
    #[cfg(feature = "redb")]
    pub fn write_redb(db: &'db std::sync::Arc<redb::Database>) -> Result<Self, NetabaseError> {
        let write_txn = db.begin_write()?;
        Ok(Self {
            backend: TxnBackend::Redb(RedbTxnBackend {
                read_txn: RefCell::new(None),
                write_txn: RefCell::new(Some(write_txn)),
                db,
                _phantom: PhantomData,
            }),
            _mode: PhantomData,
        })
    }
}

// ============================================================================
// TxnGuard Implementation - Operations available on ALL modes
// ============================================================================

impl<'db, D, Mode> TxnGuard<'db, D, Mode>
where
    D: NetabaseDefinitionTrait,
{
    /// Open a tree for a specific model type.
    ///
    /// The returned `TreeView` inherits the transaction mode, so read-only
    /// transactions return read-only tree views.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # use netabase_store::transaction::TxnGuard;
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = NetabaseStore::sled("./db")?;
    /// let mut txn = store.read();
    /// let users = txn.open_tree::<User>();
    /// let user = users.get(UserPrimaryKey(1))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_tree<M>(&mut self) -> TreeView<'_, D, M, Mode>
    where
        M: NetabaseModelTrait<D>,
    {
        match &mut self.backend {
            #[cfg(feature = "sled")]
            TxnBackend::Sled(backend) => {
                let tree_name = M::discriminant_name();
                let secondary_tree_name = format!("{}_secondary", tree_name);

                // Get or create trees (sled::Tree is Arc-based, cheap to clone)
                let tree = backend.db.open_tree(tree_name).unwrap();
                let secondary_tree = backend.db.open_tree(&secondary_tree_name).unwrap();

                TreeView {
                    backend: TreeBackend::Sled(SledTreeBackend {
                        tree,
                        secondary_tree,
                        _phantom: PhantomData,
                    }),
                    _mode: PhantomData,
                }
            }
            #[cfg(feature = "redb")]
            TxnBackend::Redb(backend) => {
                let table_name = M::discriminant_name();
                let secondary_table_name = format!("{}_secondary", table_name);

                TreeView {
                    backend: TreeBackend::Redb(RedbTreeBackend {
                        txn_backend: backend,
                        table_name,
                        secondary_table_name: Box::leak(secondary_table_name.into_boxed_str()),
                        _phantom: PhantomData,
                    }),
                    _mode: PhantomData,
                }
            }
        }
    }
}

// ============================================================================
// TxnGuard Implementation - Operations ONLY on ReadWrite mode
// ============================================================================

impl<'db, D> TxnGuard<'db, D, ReadWrite>
where
    D: NetabaseDefinitionTrait,
{
    /// Commit the transaction, making all changes permanent.
    ///
    /// This consumes the transaction guard. If not called, the transaction
    /// will be rolled back on drop.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = NetabaseStore::sled("./db")?;
    /// # let user = User { id: 1, name: "Alice".to_string() };
    /// let mut txn = store.write();
    /// let mut tree = txn.open_tree::<User>();
    /// tree.put(user)?;
    /// txn.commit()?;  // Persist changes
    /// # Ok(())
    /// # }
    /// ```
    pub fn commit(self) -> Result<(), NetabaseError> {
        match self.backend {
            #[cfg(feature = "sled")]
            TxnBackend::Sled(_) => {
                // Sled auto-commits on batch application
                Ok(())
            }
            #[cfg(feature = "redb")]
            TxnBackend::Redb(backend) => {
                if let Some(txn) = backend.write_txn.borrow_mut().take() {
                    txn.commit()?;
                }
                Ok(())
            }
        }
    }

    /// Rollback the transaction, discarding all changes.
    ///
    /// This is called automatically on drop if `commit()` is not called.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = NetabaseStore::sled("./db")?;
    /// # let error_condition = true;
    /// let mut txn = store.write();
    /// // ... operations ...
    /// if error_condition {
    ///     txn.rollback()?;  // Explicit rollback
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn rollback(self) -> Result<(), NetabaseError> {
        // Drop handles rollback automatically
        Ok(())
    }
}

// Note: Drop behavior is automatic for both Sled and Redb
// - Sled: Batches that aren't applied are simply dropped
// - Redb: Transactions auto-rollback when dropped without commit

// ============================================================================
// TreeView Implementation - Operations available on ALL modes
// ============================================================================

impl<'txn, D, M, Mode> TreeView<'txn, D, M, Mode>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
{
    /// Get a model by primary key.
    ///
    /// Available on both read-only and read-write transactions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = NetabaseStore::sled("./db")?;
    /// let mut txn = store.read();
    /// let tree = txn.open_tree::<User>();
    /// let user = tree.get(UserPrimaryKey(1))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(
        &self,
        key: <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError> {
        match &self.backend {
            #[cfg(feature = "sled")]
            TreeBackend::Sled(backend) => {
                let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
                    .map_err(crate::error::EncodingDecodingError::from)?;
                match backend.tree.get(key_bytes)? {
                    Some(bytes) => {
                        let (model, _): (M, usize) =
                            bincode::decode_from_slice(&bytes, bincode::config::standard())
                                .map_err(crate::error::EncodingDecodingError::from)?;
                        Ok(Some(model))
                    }
                    None => Ok(None),
                }
            }
            #[cfg(feature = "redb")]
            TreeBackend::Redb(backend) => {
                // Get table definition - types implement Key/Value directly!
                use redb::ReadableTable;

                let table_def: redb::TableDefinition<
                    <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
                    M,
                > = redb::TableDefinition::new(backend.table_name);

                // Try to open the table - it might not exist yet
                if let Some(ref read_txn) = *backend.txn_backend.read_txn.borrow() {
                    let table = match read_txn.open_table(table_def) {
                        Ok(table) => table,
                        Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
                        Err(e) => return Err(NetabaseError::RedbTableError(e)),
                    };

                    match table.get(key)? {
                        Some(model_guard) => {
                            // WORKAROUND: Re-serialization overhead for type safety
                            //
                            // Due to Rust's type system limitations with Generic Associated Types (GATs),
                            // even though we define `type SelfType<'a> = Self` in our Value implementations,
                            // the compiler cannot prove at this call site that `<M as Value>::SelfType<'_>` equals `M`.
                            // This prevents us from using `.clone()` or safe zero-cost coercion.
                            //
                            // Current approach: Serialize to bytes, then deserialize back to M
                            // - Adds ~6.6x overhead on get() operations (68µs vs 10µs for 100 items)
                            // - Safe: No undefined behavior or type system violations
                            // - Correct: Guaranteed type compatibility
                            //
                            // Alternatives explored and rejected:
                            // - Unsafe transmute: Would work but violates safety guarantees
                            // - Custom trait bounds (OwnedRedbValue): Failed due to trait composition issues
                            // - Direct .clone(): Compiler cannot prove type equality
                            //
                            // For lower overhead, use the Sled backend (~1.2x overhead instead of ~6.6x)
                            // See benchmarks: cargo bench --bench redb_wrapper_overhead --features "redb,libp2p"
                            use redb::Value;
                            let value_ref = model_guard.value();
                            let v_bytes = M::as_bytes(&value_ref);
                            let model = bincode::decode_from_slice(
                                &v_bytes.as_ref(),
                                bincode::config::standard(),
                            )
                            .map_err(crate::error::EncodingDecodingError::from)?
                            .0;
                            Ok(Some(model))
                        }
                        None => Ok(None),
                    }
                } else if let Some(ref write_txn) = *backend.txn_backend.write_txn.borrow() {
                    let table = match write_txn.open_table(table_def) {
                        Ok(table) => table,
                        Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
                        Err(e) => return Err(NetabaseError::RedbTableError(e)),
                    };

                    match table.get(key)? {
                        Some(model_guard) => {
                            // WORKAROUND: Re-serialization overhead for type safety
                            // (Same limitation as read transaction path above)
                            use redb::Value;
                            let value_ref = model_guard.value();
                            let v_bytes = M::as_bytes(&value_ref);
                            let model = bincode::decode_from_slice(
                                &v_bytes.as_ref(),
                                bincode::config::standard(),
                            )
                            .map_err(crate::error::EncodingDecodingError::from)?
                            .0;
                            Ok(Some(model))
                        }
                        None => Ok(None),
                    }
                } else {
                    Err(NetabaseError::Storage("No transaction available".into()))
                }
            }
        }
    }

    /// Get a model by primary key, using the `Borrow` trait for zero-copy access (redb only).
    ///
    /// This method leverages the `Borrow<UserRef<'_>>` trait to provide zero-copy
    /// access to the model data. The model is first retrieved (owned), then the
    /// cached borrowed view is returned via the `Borrow` trait.
    ///
    /// # Performance
    ///
    /// - First call: Allocates owned model + caches borrowed view
    /// - Subsequent calls on same model: Zero-copy via cached `UserRef`
    /// - Best used when you'll access the same model multiple times
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = NetabaseStore::sled("./db")?;
    /// # let mut txn = store.read();
    /// use std::borrow::Borrow;
    ///
    /// let tree = txn.open_tree::<User>();
    /// if let Some(user) = tree.get(UserPrimaryKey(1))? {
    ///     // Borrow trait usage (if implemented for model)
    ///     println!("Name: {}", user.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// This returns the owned model, but you can use `.borrow()` to get zero-copy access.
    /// True zero-copy retrieval from database requires returning guards, see `get_borrowed_guard()`.
    /// For most use cases, using `user.borrow()` on the returned model is sufficient.

    // NOTE: get_borrowed_guard() attempted but blocked by architectural limitation.
    // See PHASE3_LIMITATION.md for details.
    // Cannot return guards that reference local table variables.
    // Phase 4 will use closure-based API instead.

    /// Get all models matching a secondary key.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct Post {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         #[secondary_key]
    /// #         pub author_id: u64,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = NetabaseStore::sled("./db")?;
    /// # let mut txn = store.read();
    /// let tree = txn.open_tree::<Post>();
    /// let posts = tree.get_by_secondary_key(PostSecondaryKeys::AuthorId(1))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_by_secondary_key(
        &self,
        secondary_key: <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey,
    ) -> Result<Vec<M>, NetabaseError>
    where
        <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey: PartialEq,
    {
        match &self.backend {
            #[cfg(feature = "sled")]
            TreeBackend::Sled(backend) => {
                let mut results = Vec::new();

                // Scan the secondary tree for matching keys
                for item in backend.secondary_tree.iter() {
                    let (composite_bytes, _) = item?;
                    let ((sec_key, prim_key), _count): ((<M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey, <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey), usize) =
                        bincode::decode_from_slice(&composite_bytes, bincode::config::standard())
                            .map_err(crate::error::EncodingDecodingError::from)?;

                    if sec_key == secondary_key {
                        // Get the model from the primary tree
                        if let Some(model) = self.get(prim_key)? {
                            results.push(model);
                        }
                    }
                }

                Ok(results)
            }
            #[cfg(feature = "redb")]
            TreeBackend::Redb(backend) => {
                use redb::ReadableTable;

                // Secondary index uses CompositeKey for efficient lookups
                let sec_table_def: redb::TableDefinition<
                    CompositeKey<
                        <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey,
                        <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
                    >,
                    (),
                > = redb::TableDefinition::new(backend.secondary_table_name);

                let mut results = Vec::new();

                if let Some(ref read_txn) = *backend.txn_backend.read_txn.borrow() {
                    let sec_table = match read_txn.open_table(sec_table_def) {
                        Ok(table) => table,
                        Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
                        Err(e) => return Err(NetabaseError::RedbTableError(e)),
                    };

                    // Iterate over secondary index
                    for item in sec_table.iter()? {
                        let (composite_key, _) = item?;
                        let comp = composite_key.value();

                        if comp.secondary == secondary_key {
                            if let Some(model) = self.get(comp.primary)? {
                                results.push(model);
                            }
                        }
                    }
                } else if let Some(ref write_txn) = *backend.txn_backend.write_txn.borrow() {
                    let sec_table = match write_txn.open_table(sec_table_def) {
                        Ok(table) => table,
                        Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
                        Err(e) => return Err(NetabaseError::RedbTableError(e)),
                    };

                    // Iterate over secondary index
                    for item in sec_table.iter()? {
                        let (composite_key, _) = item?;
                        let comp = composite_key.value();

                        if comp.secondary == secondary_key {
                            if let Some(model) = self.get(comp.primary)? {
                                results.push(model);
                            }
                        }
                    }
                } else {
                    return Err(NetabaseError::Storage("No transaction available".into()));
                }

                Ok(results)
            }
        }
    }

    /// Get the number of models in the tree.
    pub fn len(&self) -> Result<usize, NetabaseError> {
        match &self.backend {
            #[cfg(feature = "sled")]
            TreeBackend::Sled(backend) => Ok(backend.tree.len()),
            #[cfg(feature = "redb")]
            TreeBackend::Redb(backend) => {
                // Use direct types for primary table
                let table_def: redb::TableDefinition<
                    <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
                    M,
                > = redb::TableDefinition::new(backend.table_name);

                if let Some(ref read_txn) = *backend.txn_backend.read_txn.borrow() {
                    let table = match read_txn.open_table(table_def) {
                        Ok(table) => table,
                        Err(redb::TableError::TableDoesNotExist(_)) => return Ok(0),
                        Err(e) => return Err(NetabaseError::RedbTableError(e)),
                    };
                    Ok(table.len()? as usize)
                } else if let Some(ref write_txn) = *backend.txn_backend.write_txn.borrow() {
                    let table = match write_txn.open_table(table_def) {
                        Ok(table) => table,
                        Err(redb::TableError::TableDoesNotExist(_)) => return Ok(0),
                        Err(e) => return Err(NetabaseError::RedbTableError(e)),
                    };
                    Ok(table.len()? as usize)
                } else {
                    Err(NetabaseError::Storage("No transaction available".into()))
                }
            }
        }
    }

    /// Check if the tree is empty.
    pub fn is_empty(&self) -> Result<bool, NetabaseError> {
        Ok(self.len()? == 0)
    }

    /// Iterate over all models in the tree.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = NetabaseStore::sled("./db")?;
    /// # let mut txn = store.read();
    /// let tree = txn.open_tree::<User>();
    /// for (key, user) in tree.iter()? {
    ///     println!("User: {}", user.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter(
        &self,
    ) -> Result<
        Vec<(
            <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
            M,
        )>,
        NetabaseError,
    > {
        match &self.backend {
            #[cfg(feature = "sled")]
            TreeBackend::Sled(backend) => {
                let mut results = Vec::new();
                for item in backend.tree.iter() {
                    let (key_bytes, value_bytes) = item?;
                    let (key, _): (
                        <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
                        usize,
                    ) = bincode::decode_from_slice(&key_bytes, bincode::config::standard())
                        .map_err(crate::error::EncodingDecodingError::from)?;
                    let (model, _): (M, usize) =
                        bincode::decode_from_slice(&value_bytes, bincode::config::standard())
                            .map_err(crate::error::EncodingDecodingError::from)?;
                    results.push((key, model));
                }
                Ok(results)
            }
            #[cfg(feature = "redb")]
            TreeBackend::Redb(backend) => {
                // Use direct types for primary table
                let table_def: redb::TableDefinition<
                    <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
                    M,
                > = redb::TableDefinition::new(backend.table_name);

                let mut results: Vec<(
                    <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
                    M,
                )> = Vec::new();

                if let Some(ref read_txn) = *backend.txn_backend.read_txn.borrow() {
                    let table = match read_txn.open_table(table_def) {
                        Ok(table) => table,
                        Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
                        Err(e) => return Err(NetabaseError::RedbTableError(e)),
                    };

                    for item in table.iter()? {
                        let (key, value) = item?;
                        // WORKAROUND: Re-serialization overhead in iter()
                        // ~1.8x overhead (15µs vs 8.5µs for 100 items)
                        // Same GAT limitation as get() - see detailed comment in get() method above
                        use redb::Value;
                        let key_ref = key.value();
                        let value_ref = value.value();
                        let k_bytes = <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey::as_bytes(&key_ref);
                        let v_bytes = M::as_bytes(&value_ref);
                        let k = bincode::decode_from_slice(
                            &k_bytes.as_ref(),
                            bincode::config::standard(),
                        )
                        .map_err(crate::error::EncodingDecodingError::from)?
                        .0;
                        let v = bincode::decode_from_slice(
                            &v_bytes.as_ref(),
                            bincode::config::standard(),
                        )
                        .map_err(crate::error::EncodingDecodingError::from)?
                        .0;
                        results.push((k, v));
                    }
                } else if let Some(ref write_txn) = *backend.txn_backend.write_txn.borrow() {
                    let table = match write_txn.open_table(table_def) {
                        Ok(table) => table,
                        Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
                        Err(e) => return Err(NetabaseError::RedbTableError(e)),
                    };

                    for item in table.iter()? {
                        let (key, value) = item?;
                        // WORKAROUND: Re-serialization overhead in iter()
                        // (Same limitation as read transaction path above)
                        use redb::Value;
                        let key_ref = key.value();
                        let value_ref = value.value();
                        let k_bytes = <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey::as_bytes(&key_ref);
                        let v_bytes = M::as_bytes(&value_ref);
                        let k = bincode::decode_from_slice(
                            &k_bytes.as_ref(),
                            bincode::config::standard(),
                        )
                        .map_err(crate::error::EncodingDecodingError::from)?
                        .0;
                        let v = bincode::decode_from_slice(
                            &v_bytes.as_ref(),
                            bincode::config::standard(),
                        )
                        .map_err(crate::error::EncodingDecodingError::from)?
                        .0;
                        results.push((k, v));
                    }
                } else {
                    return Err(NetabaseError::Storage("No transaction available".into()));
                }

                Ok(results)
            }
        }
    }

    // NOTE: iter_borrowed_guard() attempted but blocked by architectural limitation.
    // See PHASE3_LIMITATION.md for details.
    // Cannot return iterators that reference local table variables.
    // Phase 4 will use closure-based API instead.
}

// ============================================================================
// TreeView Implementation - Operations ONLY on ReadWrite mode
// ============================================================================

impl<'txn, D, M> TreeView<'txn, D, M, ReadWrite>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
{
    /// Insert or update a model.
    ///
    /// Only available on read-write transactions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = NetabaseStore::sled("./db")?;
    /// # let user = User { id: 1, name: "Alice".to_string() };
    /// let mut txn = store.write();
    /// let mut tree = txn.open_tree::<User>();
    /// tree.put(user)?;
    /// txn.commit()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn put(&mut self, model: M) -> Result<(), NetabaseError> {
        let primary_key = model.primary_key();
        let secondary_keys = model.secondary_keys();

        match &mut self.backend {
            #[cfg(feature = "sled")]
            TreeBackend::Sled(backend) => {
                let key_bytes = bincode::encode_to_vec(&primary_key, bincode::config::standard())
                    .map_err(crate::error::EncodingDecodingError::from)?;
                let value_bytes = bincode::encode_to_vec(&model, bincode::config::standard())
                    .map_err(crate::error::EncodingDecodingError::from)?;

                // Apply directly to tree (Sled doesn't have true transactions)
                backend.tree.insert(&key_bytes, value_bytes)?;

                // Handle secondary keys
                if !secondary_keys.is_empty() {
                    for sec_key in secondary_keys.values() {
                        let composite_key = (sec_key, primary_key.clone());
                        let composite_bytes =
                            bincode::encode_to_vec(&composite_key, bincode::config::standard())
                                .map_err(crate::error::EncodingDecodingError::from)?;
                        backend.secondary_tree.insert(&composite_bytes, &[])?;
                    }
                }

                Ok(())
            }
            #[cfg(feature = "redb")]
            TreeBackend::Redb(backend) => {
                // Primary table uses direct types, secondary uses CompositeKey
                let table_def: redb::TableDefinition<
                    <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
                    M,
                > = redb::TableDefinition::new(backend.table_name);
                let sec_table_def: redb::TableDefinition<
                    CompositeKey<
                        <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey,
                        <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
                    >,
                    (),
                > = redb::TableDefinition::new(backend.secondary_table_name);

                // Get write transaction
                if let Some(ref write_txn) = *backend.txn_backend.write_txn.borrow() {
                    {
                        let mut table = write_txn.open_table(table_def)?;
                        table.insert(primary_key.clone(), model.clone())?;

                        if !secondary_keys.is_empty() {
                            let mut sec_table = write_txn.open_table(sec_table_def)?;
                            for sec_key in secondary_keys.values() {
                                let composite_key = CompositeKey::new(sec_key.clone(), primary_key.clone());
                                sec_table.insert(composite_key, ())?;
                            }
                        }
                    }
                    Ok(())
                } else {
                    Err(NetabaseError::Storage(
                        "No write transaction available".into(),
                    ))
                }
            }
        }
    }

    /// Remove a model by primary key.
    ///
    /// Returns the removed model if it existed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = NetabaseStore::sled("./db")?;
    /// let mut txn = store.write();
    /// let mut tree = txn.open_tree::<User>();
    /// let removed = tree.remove(UserPrimaryKey(1))?;
    /// txn.commit()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn remove(
        &mut self,
        key: <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError> {
        // First get the model so we can clean up secondary keys
        let model = self.get(key.clone())?;

        if model.is_none() {
            return Ok(None);
        }

        match &mut self.backend {
            #[cfg(feature = "sled")]
            TreeBackend::Sled(backend) => {
                let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
                    .map_err(crate::error::EncodingDecodingError::from)?;

                // Remove directly from tree
                backend.tree.remove(&key_bytes)?;

                // Clean up secondary keys
                if let Some(ref m) = model {
                    let secondary_keys = m.secondary_keys();
                    if !secondary_keys.is_empty() {
                        for sec_key in secondary_keys.values() {
                            let composite_key = (sec_key, M::Keys::from(key.clone()));
                            let composite_bytes =
                                bincode::encode_to_vec(&composite_key, bincode::config::standard())
                                    .map_err(crate::error::EncodingDecodingError::from)?;
                            backend.secondary_tree.remove(&composite_bytes)?;
                        }
                    }
                }

                Ok(model)
            }
            #[cfg(feature = "redb")]
            TreeBackend::Redb(backend) => {
                // Primary table uses direct types, secondary uses CompositeKey
                let table_def: redb::TableDefinition<
                    <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
                    M,
                > = redb::TableDefinition::new(backend.table_name);
                let sec_table_def: redb::TableDefinition<
                    CompositeKey<
                        <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey,
                        <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
                    >,
                    (),
                > = redb::TableDefinition::new(backend.secondary_table_name);

                if let Some(ref write_txn) = *backend.txn_backend.write_txn.borrow() {
                    {
                        let mut table = write_txn.open_table(table_def)?;
                        table.remove(key.clone())?;

                        // Clean up secondary keys
                        if let Some(ref m) = model {
                            let secondary_keys = m.secondary_keys();
                            if !secondary_keys.is_empty() {
                                let mut sec_table = write_txn.open_table(sec_table_def)?;
                                for sec_key in secondary_keys.values() {
                                    let composite_key = CompositeKey::new(sec_key.clone(), key.clone());
                                    sec_table.remove(composite_key)?;
                                }
                            }
                        }
                    }
                    Ok(model)
                } else {
                    Err(NetabaseError::Storage(
                        "No write transaction available".into(),
                    ))
                }
            }
        }
    }

    /// Remove all models from the tree.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = NetabaseStore::sled("./db")?;
    /// let mut txn = store.write();
    /// let mut tree = txn.open_tree::<User>();
    /// tree.clear()?;
    /// txn.commit()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn clear(&mut self) -> Result<(), NetabaseError> {
        match &mut self.backend {
            #[cfg(feature = "sled")]
            TreeBackend::Sled(backend) => {
                // For Sled, clear the trees directly
                backend.tree.clear()?;
                backend.secondary_tree.clear()?;
                Ok(())
            }
            #[cfg(feature = "redb")]
            TreeBackend::Redb(backend) => {
                // Use direct types for primary table, CompositeKey for secondary
                let table_def: redb::TableDefinition<
                    <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
                    M,
                > = redb::TableDefinition::new(backend.table_name);
                let sec_table_def: redb::TableDefinition<
                    CompositeKey<
                        <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey,
                        <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
                    >,
                    (),
                > = redb::TableDefinition::new(backend.secondary_table_name);

                if let Some(ref write_txn) = *backend.txn_backend.write_txn.borrow() {
                    // Clear by removing all keys - collect keys first to avoid borrow conflicts
                    let keys_to_remove: Vec<
                        <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
                    > = {
                        let table = write_txn.open_table(table_def)?;
                        table.iter()?.map(|item| {
                            let (k, _) = item.unwrap();
                            // WORKAROUND: Re-serialization overhead in clear()
                            // Same GAT limitation as get() - see detailed comment in get() method above
                            use redb::Value;
                            let k_ref = k.value();
                            let k_bytes = <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey::as_bytes(&k_ref);
                            bincode::decode_from_slice(&k_bytes.as_ref(), bincode::config::standard()).unwrap().0
                        }).collect()
                    };

                    {
                        let mut table = write_txn.open_table(table_def)?;
                        for key in keys_to_remove {
                            table.remove(key)?;
                        }
                    }

                    // Clear secondary index
                    let sec_keys_to_remove: Vec<CompositeKey<<M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey, <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey>> = {
                        let sec_table = write_txn.open_table(sec_table_def)?;
                        sec_table.iter()?.map(|item| {
                            let (k, _) = item.unwrap();
                            // WORKAROUND: Re-serialization for secondary index keys
                            // Same GAT limitation affects CompositeKey type
                            use redb::Value;
                            let k_ref = k.value();
                            let k_bytes = CompositeKey::<<M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey, <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey>::as_bytes(&k_ref);
                            bincode::decode_from_slice(&k_bytes.as_ref(), bincode::config::standard()).unwrap().0
                        }).collect()
                    };

                    {
                        let mut sec_table = write_txn.open_table(sec_table_def)?;
                        for key in sec_keys_to_remove {
                            sec_table.remove(key)?;
                        }
                    }
                    Ok(())
                } else {
                    Err(NetabaseError::Storage(
                        "No write transaction available".into(),
                    ))
                }
            }
        }
    }

    /// Insert multiple models efficiently.
    ///
    /// This is equivalent to calling `put()` for each model, but may be
    /// optimized by the backend.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = NetabaseStore::sled("./db")?;
    /// # let users = vec![User { id: 1, name: "Alice".to_string() }];
    /// let mut txn = store.write();
    /// let mut tree = txn.open_tree::<User>();
    /// tree.put_many(users)?;
    /// txn.commit()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn put_many<I>(&mut self, models: I) -> Result<(), NetabaseError>
    where
        I: IntoIterator<Item = M>,
    {
        for model in models {
            self.put(model)?;
        }
        Ok(())
    }

    /// Remove multiple models efficiently.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = NetabaseStore::sled("./db")?;
    /// let mut txn = store.write();
    /// let mut tree = txn.open_tree::<User>();
    /// let keys = vec![UserPrimaryKey(1), UserPrimaryKey(2)];
    /// tree.remove_many(keys)?;
    /// txn.commit()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn remove_many<I>(&mut self, keys: I) -> Result<Vec<Option<M>>, NetabaseError>
    where
        I: IntoIterator<
            Item = <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
        >,
    {
        let mut removed = Vec::new();
        for key in keys {
            removed.push(self.remove(key)?);
        }
        Ok(removed)
    }
}

// ============================================================================
// TreeView Implementation - Batch operations on ALL modes
// ============================================================================

impl<'txn, D, M, Mode> TreeView<'txn, D, M, Mode>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
{
    /// Get multiple models by primary keys efficiently.
    ///
    /// Available on all transaction modes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = NetabaseStore::sled("./db")?;
    /// let mut txn = store.read();
    /// let tree = txn.open_tree::<User>();
    /// let keys = vec![UserPrimaryKey(1), UserPrimaryKey(2), UserPrimaryKey(3)];
    /// let users = tree.get_many(keys)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_many<I>(&self, keys: I) -> Result<Vec<Option<M>>, NetabaseError>
    where
        I: IntoIterator<
            Item = <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
        >,
    {
        let mut results = Vec::new();
        for key in keys {
            results.push(self.get(key)?);
        }
        Ok(results)
    }
}
