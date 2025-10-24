#[cfg(feature = "wasm")]
use crate::error::NetabaseError;
#[cfg(feature = "wasm")]
use crate::traits::convert::ToIVec;
#[cfg(feature = "wasm")]
use crate::traits::definition::NetabaseDefinitionTrait;
#[cfg(feature = "wasm")]
use crate::traits::model::NetabaseModelTrait;
#[cfg(feature = "wasm")]
use crate::traits::tree::NetabaseTreeAsync;
#[cfg(feature = "wasm")]
use indexed_db_futures::prelude::*;
#[cfg(feature = "wasm")]
use std::marker::PhantomData;
#[cfg(feature = "wasm")]
use strum::IntoEnumIterator;
#[cfg(feature = "wasm")]
use wasm_bindgen::JsValue;
#[cfg(feature = "wasm")]
use web_sys::{IdbCursorDirection, IdbTransactionMode};

/// Type-safe wrapper around IndexedDB that works with NetabaseDefinitionTrait types.
///
/// The IndexedDBStore provides a type-safe interface to the browser's IndexedDB,
/// using discriminants as object store names and ensuring all operations are type-checked.
#[cfg(feature = "wasm")]
pub struct IndexedDBStore<D>
where
    D: NetabaseDefinitionTrait,
{
    db: std::sync::Arc<IdbDatabase>,
    db_name: String,
    pub trees: Vec<D::Discriminant>,
    _phantom: PhantomData<D>,
}

#[cfg(feature = "wasm")]
impl<D> IndexedDBStore<D>
where
    D: NetabaseDefinitionTrait,
{
    /// Get direct access to the underlying IndexedDB database
    pub fn db(&self) -> &IdbDatabase {
        &self.db
    }

    /// Get the database name
    pub fn db_name(&self) -> &str {
        &self.db_name
    }

    /// Get all tree names (discriminants) in the database
    pub fn tree_names(&self) -> Vec<D::Discriminant> {
        D::Discriminant::iter().collect()
    }
}

#[cfg(feature = "wasm")]
impl<D> IndexedDBStore<D>
where
    D: NetabaseDefinitionTrait + ToIVec,
{
    /// Open a new IndexedDBStore with the given database name
    pub async fn new(db_name: &str) -> Result<Self, NetabaseError> {
        Self::new_with_version(db_name, 1).await
    }

    /// Open a new IndexedDBStore with a specific version number
    pub async fn new_with_version(db_name: &str, version: u32) -> Result<Self, NetabaseError> {
        let mut db_req = IdbDatabase::open_u32(db_name, version)
            .map_err(|e| NetabaseError::Storage(format!("Failed to open IndexedDB: {:?}", e)))?;

        // Set up object stores on upgrade
        db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            let db = evt.db();

            // Create object stores for each discriminant
            for disc in D::Discriminants::iter() {
                let store_name: String = disc.into();

                // Check if object store already exists
                if !db.object_store_names().any(|name| name == store_name) {
                    // Create object store with auto-incrementing keys disabled
                    // We'll use our own keys
                    let _store = db.create_object_store(&store_name)?;
                }
            }

            // Create secondary key stores
            for disc in D::Discriminants::iter() {
                let store_name: String = disc.into();
                let sec_store_name = format!("{}_secondary", store_name);

                if !db.object_store_names().any(|name| name == sec_store_name) {
                    let _ = db.create_object_store(&sec_store_name)?;
                }
            }

            Ok(())
        }));

        let db = db_req
            .await
            .map_err(|e| NetabaseError::Storage(format!("Failed to open IndexedDB: {:?}", e)))?;

        Ok(Self {
            db: std::sync::Arc::new(db),
            db_name: db_name.to_string(),
            trees: D::Discriminant::iter().collect(),
            _phantom: PhantomData,
        })
    }

    /// Open a tree for a specific model type
    /// This creates a tree abstraction that can handle dynamic object store creation
    pub fn open_tree<M>(&self) -> IndexedDBStoreTree<D, M>
    where
        M: NetabaseModelTrait<D> + TryFrom<D> + Into<D>,
        D: TryFrom<M> + ToIVec,
    {
        IndexedDBStoreTree::new(std::sync::Arc::clone(&self.db), M::DISCRIMINANT)
    }

    /// Get all store names (discriminants) in the database
    pub fn store_names(&self) -> Vec<String> {
        D::Discriminants::iter().map(|d| d.into()).collect()
    }

    /// Close the database connection
    pub fn close(&self) {
        self.db.close();
    }
}

/// Type-safe wrapper around an IndexedDB object store for a specific model type.
///
/// IndexedDBStoreTree provides async CRUD operations for a single model type with automatic
/// encoding/decoding and secondary key management. It handles dynamic object store creation
/// similar to sled's Tree abstraction.
#[cfg(feature = "wasm")]
pub struct IndexedDBStoreTree<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
{
    db: std::sync::Arc<IdbDatabase>,
    discriminant: D::Discriminant,
    _phantom_d: PhantomData<D>,
    _phantom_m: PhantomData<M>,
}

#[cfg(feature = "wasm")]
impl<D, M> IndexedDBStoreTree<D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec,
    M: NetabaseModelTrait<D> + TryFrom<D> + Into<D>,
{
    /// Create a new IndexedDBStoreTree with shared database access
    /// Uses discriminant directly instead of string conversion
    fn new(db: std::sync::Arc<IdbDatabase>, discriminant: D::Discriminant) -> Self {
        Self {
            db,
            discriminant,
            _phantom_d: PhantomData,
            _phantom_m: PhantomData,
        }
    }

    /// Get the store name from discriminant
    fn store_name(&self) -> String {
        self.discriminant.to_string()
    }

    /// Get the secondary store name using discriminant-based naming
    fn secondary_store_name(&self) -> String {
        format!("{}_secondary", self.discriminant)
    }

    /// Open a secondary tree for indexing using discriminant-based naming
    /// This allows dynamic creation of object stores for secondary key indexing and future graph features
    pub fn open_secondary_tree(&self, suffix: &str) -> IndexedDBTree {
        let tree_name = format!("{}_{}", self.discriminant, suffix);
        IndexedDBTree::new(std::sync::Arc::clone(&self.db), tree_name)
    }

    /// Insert or update a model in the store
    pub async fn put(&self, model: M) -> Result<(), NetabaseError>
    where
        D: From<M>,
    {
        let primary_key = model.primary_key();
        let key_bytes = bincode::encode_to_vec(&primary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

        let definition: D = model.clone().into();
        let value_bytes = definition.to_ivec()?;

        // Convert bytes to JsValue
        let key_js = js_sys::Uint8Array::from(&key_bytes[..]);
        let value_js = js_sys::Uint8Array::from(&value_bytes[..]);

        let store_name = self.store_name();

        // Create transaction
        let tx = self
            .db
            .transaction_on_one_with_mode(&store_name, IdbTransactionMode::Readwrite)
            .map_err(|e| {
                NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
            })?;

        let store = tx
            .object_store(&store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        let _request = store
            .put_key_val(&key_js, &value_js)
            .map_err(|e| NetabaseError::Storage(format!("Failed to put value: {:?}", e)))?;

        // Wait for transaction to complete
        let _ = tx
            .await
            .into_result()
            .map_err(|e| NetabaseError::Storage(format!("Transaction failed: {:?}", e)))?;

        // Handle secondary keys using secondary trees
        let secondary_keys = model.secondary_keys();
        for sec_key in secondary_keys {
            self.insert_secondary_key(&sec_key, &primary_key).await?;
        }

        Ok(())
    }

    /// Get a model by its primary key
    pub async fn get(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

        let key_js = js_sys::Uint8Array::from(&key_bytes[..]);

        let store_name = self.store_name();

        let tx = self.db.transaction_on_one(&store_name).map_err(|e| {
            NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
        })?;

        let store = tx
            .object_store(&store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        let value = store
            .get(&key_js)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get value: {:?}", e)))?
            .await
            .map_err(|e| NetabaseError::Storage(format!("Get request failed: {:?}", e)))?;

        match value {
            Some(js_value) => {
                // Convert JsValue to bytes
                let uint8_array = js_sys::Uint8Array::new(&js_value);
                let mut bytes = vec![0u8; uint8_array.length() as usize];
                uint8_array.copy_to(&mut bytes);

                let definition = D::from_ivec(&bytes)?;
                match M::try_from(definition) {
                    Ok(model) => Ok(Some(model)),
                    Err(_) => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    /// Delete a model by its primary key
    pub async fn remove(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        // First get the model so we can clean up secondary keys
        let model = self.get(key.clone()).await?;

        if model.is_none() {
            return Ok(None);
        }

        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;
        let key_js = js_sys::Uint8Array::from(&key_bytes[..]);

        let store_name = self.store_name();

        let tx = self
            .db
            .transaction_on_one_with_mode(&store_name, IdbTransactionMode::Readwrite)
            .map_err(|e| {
                NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
            })?;

        let store = tx
            .object_store(&store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        store
            .delete(&key_js)
            .map_err(|e| NetabaseError::Storage(format!("Failed to delete value: {:?}", e)))?
            .await
            .map_err(|e| NetabaseError::Storage(format!("Delete request failed: {:?}", e)))?;

        let _ = tx
            .await
            .into_result()
            .map_err(|e| NetabaseError::Storage(format!("Transaction failed: {:?}", e)))?;

        // Clean up secondary keys
        if let Some(ref m) = model {
            let secondary_keys = m.secondary_keys();
            for sec_key in secondary_keys {
                self.remove_secondary_key(&sec_key, &key).await?;
            }
        }

        Ok(model)
    }

    /// Get the number of models in the store
    pub async fn len(&self) -> Result<usize, NetabaseError> {
        let store_name = self.store_name();

        let tx = self.db.transaction_on_one(&store_name).map_err(|e| {
            NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
        })?;

        let store = tx
            .object_store(&store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        let count = store
            .count()
            .map_err(|e| NetabaseError::Storage(format!("Failed to count: {:?}", e)))?
            .await
            .map_err(|e| NetabaseError::Storage(format!("Count request failed: {:?}", e)))?;

        Ok(count as usize)
    }

    /// Check if the store is empty
    pub async fn is_empty(&self) -> Result<bool, NetabaseError> {
        Ok(self.len().await? == 0)
    }

    /// Clear all models from the store
    pub async fn clear(&self) -> Result<(), NetabaseError> {
        let store_name = self.store_name();

        let tx = self
            .db
            .transaction_on_one_with_mode(&store_name, IdbTransactionMode::Readwrite)
            .map_err(|e| {
                NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
            })?;

        let store = tx
            .object_store(&store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        store
            .clear()
            .map_err(|e| NetabaseError::Storage(format!("Failed to clear: {:?}", e)))?
            .await
            .map_err(|e| NetabaseError::Storage(format!("Clear request failed: {:?}", e)))?;

        let _ = tx
            .await
            .into_result()
            .map_err(|e| NetabaseError::Storage(format!("Transaction failed: {:?}", e)))?;

        // Also clear secondary keys using secondary tree
        let sec_tree = self.open_secondary_tree("secondary");
        sec_tree.clear().await?;

        Ok(())
    }

    /// Iterate over all models in the store
    pub async fn iter(&self) -> Result<Vec<(M::PrimaryKey, M)>, NetabaseError>
    where
        M::PrimaryKey: bincode::Decode<()>,
    {
        let store_name = self.store_name();

        let tx = self.db.transaction_on_one(&store_name).map_err(|e| {
            NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
        })?;

        let store = tx
            .object_store(&store_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        let cursor_request = store
            .open_cursor()
            .map_err(|e| NetabaseError::Storage(format!("Failed to open cursor: {:?}", e)))?;

        let cursor = cursor_request
            .await
            .map_err(|e| NetabaseError::Storage(format!("Cursor request failed: {:?}", e)))?;

        let mut results = Vec::new();

        if let Some(mut cursor) = cursor {
            loop {
                let key_js = cursor.key().ok_or_else(|| {
                    NetabaseError::Storage("Failed to get cursor key".to_string())
                })?;
                let value_js = cursor.value();

                // Convert key
                let key_array = js_sys::Uint8Array::new(&key_js);
                let mut key_bytes = vec![0u8; key_array.length() as usize];
                key_array.copy_to(&mut key_bytes);

                let (key, _) = bincode::decode_from_slice::<M::PrimaryKey, _>(
                    &key_bytes,
                    bincode::config::standard(),
                )
                .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

                // Convert value
                let value_array = js_sys::Uint8Array::new(&value_js);
                let mut value_bytes = vec![0u8; value_array.length() as usize];
                value_array.copy_to(&mut value_bytes);

                let definition = D::from_ivec(&value_bytes)?;
                let model = M::try_from(definition).map_err(|_| {
                    crate::error::NetabaseError::Conversion(
                        crate::error::EncodingDecodingError::Decoding(
                            bincode::error::DecodeError::Other("Type conversion failed"),
                        ),
                    )
                })?;

                results.push((key, model));

                // Move to next
                let continue_request = cursor.continue_cursor().map_err(|e| {
                    NetabaseError::Storage(format!("Failed to continue cursor: {:?}", e))
                })?;

                let has_next = continue_request.await.map_err(|e| {
                    NetabaseError::Storage(format!("Cursor continue failed: {:?}", e))
                })?;

                if !has_next {
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Insert a secondary key mapping using a secondary tree
    async fn insert_secondary_key(
        &self,
        secondary_key: &M::SecondaryKeys,
        primary_key: &M::PrimaryKey,
    ) -> Result<(), NetabaseError> {
        let sec_tree = self.open_secondary_tree("secondary");

        let mut composite_key = bincode::encode_to_vec(secondary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;
        let prim_key_bytes = bincode::encode_to_vec(primary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

        composite_key.extend_from_slice(&prim_key_bytes);

        // Store with empty value (we only need the key for indexing)
        sec_tree.insert(&composite_key, &[]).await?;

        Ok(())
    }

    /// Remove a secondary key mapping from a secondary tree
    async fn remove_secondary_key(
        &self,
        secondary_key: &M::SecondaryKeys,
        primary_key: &M::PrimaryKey,
    ) -> Result<(), NetabaseError> {
        let sec_tree = self.open_secondary_tree("secondary");

        let mut composite_key = bincode::encode_to_vec(secondary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;
        let prim_key_bytes = bincode::encode_to_vec(primary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;
        composite_key.extend_from_slice(&prim_key_bytes);

        sec_tree.remove(&composite_key).await?;

        Ok(())
    }

    /// Find models by secondary key using the secondary tree index
    pub async fn get_by_secondary_key(
        &self,
        secondary_key: M::SecondaryKeys,
    ) -> Result<Vec<M>, NetabaseError>
    where
        M::PrimaryKey: bincode::Decode<()>,
    {
        let sec_tree = self.open_secondary_tree("secondary");

        let sec_key_bytes = bincode::encode_to_vec(&secondary_key, bincode::config::standard())
            .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

        let mut results = Vec::new();

        // Use the secondary tree to find matching primary keys
        for (composite_key, _) in sec_tree.scan_prefix(&sec_key_bytes).await? {
            let prim_key_start = sec_key_bytes.len();
            if composite_key.len() > prim_key_start {
                let (primary_key, _) = bincode::decode_from_slice::<M::PrimaryKey, _>(
                    &composite_key[prim_key_start..],
                    bincode::config::standard(),
                )
                .map_err(|e| crate::error::EncodingDecodingError::from(e))?;

                if let Some(model) = self.get(primary_key).await? {
                    results.push(model);
                }
            }
        }

        Ok(results)
    }
}

// ============================================================================
// NetabaseTreeAsync trait implementation for IndexedDBStoreTree
// ============================================================================

#[cfg(feature = "wasm")]
impl<D, M> NetabaseTreeAsync<D, M> for IndexedDBStoreTree<D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D> + TryFrom<D> + Into<D> + Clone,
    M::PrimaryKey: bincode::Encode + bincode::Decode<()> + Clone,
    M::SecondaryKeys: bincode::Encode + bincode::Decode<()>,
    D::Discriminant: std::fmt::Display + strum::IntoEnumIterator,
{
    type PrimaryKey = M::PrimaryKey;
    type SecondaryKeys = M::SecondaryKeys;

    fn put(&self, model: M) -> impl std::future::Future<Output = Result<(), NetabaseError>> + Send {
        async move { self.put(model).await }
    }

    fn get(
        &self,
        key: Self::PrimaryKey,
    ) -> impl std::future::Future<Output = Result<Option<M>, NetabaseError>> + Send {
        async move { self.get(key).await }
    }

    fn remove(
        &self,
        key: Self::PrimaryKey,
    ) -> impl std::future::Future<Output = Result<Option<M>, NetabaseError>> + Send {
        async move { self.remove(key).await }
    }

    fn get_by_secondary_key(
        &self,
        secondary_key: Self::SecondaryKeys,
    ) -> impl std::future::Future<Output = Result<Vec<M>, NetabaseError>> + Send {
        async move { self.get_by_secondary_key(secondary_key).await }
    }

    fn is_empty(&self) -> impl std::future::Future<Output = Result<bool, NetabaseError>> + Send {
        async move { self.is_empty().await }
    }

    fn len(&self) -> impl std::future::Future<Output = Result<usize, NetabaseError>> + Send {
        async move { self.len().await }
    }

    fn clear(&self) -> impl std::future::Future<Output = Result<(), NetabaseError>> + Send {
        async move { self.clear().await }
    }
}

/// Generic tree abstraction for IndexedDB that can be used for any key-value storage
/// This allows dynamic object store creation similar to sled::Tree
/// Uses discriminant-based naming for type safety
#[cfg(feature = "wasm")]
pub struct IndexedDBTree {
    db: std::sync::Arc<IdbDatabase>,
    tree_name: String,
}

#[cfg(feature = "wasm")]
impl IndexedDBTree {
    /// Create a new IndexedDBTree with shared database access
    pub fn new(db: std::sync::Arc<IdbDatabase>, tree_name: String) -> Self {
        Self { db, tree_name }
    }

    /// Insert a key-value pair
    pub async fn insert(&self, key: &[u8], value: &[u8]) -> Result<(), NetabaseError> {
        let key_js = js_sys::Uint8Array::from(key);
        let value_js = js_sys::Uint8Array::from(value);

        let tx = self
            .db
            .transaction_on_one_with_mode(&self.tree_name, IdbTransactionMode::Readwrite)
            .map_err(|e| {
                NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
            })?;

        let store = tx
            .object_store(&self.tree_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        store
            .put_key_val(&key_js, &value_js)
            .map_err(|e| NetabaseError::Storage(format!("Failed to put value: {:?}", e)))?;

        let _ = tx
            .await
            .into_result()
            .map_err(|e| NetabaseError::Storage(format!("Transaction failed: {:?}", e)))?;

        Ok(())
    }

    /// Get a value by key
    pub async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, NetabaseError> {
        let key_js = js_sys::Uint8Array::from(key);

        let tx = self.db.transaction_on_one(&self.tree_name).map_err(|e| {
            NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
        })?;

        let store = tx
            .object_store(&self.tree_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        let value = store
            .get(&key_js)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get value: {:?}", e)))?
            .await
            .map_err(|e| NetabaseError::Storage(format!("Get request failed: {:?}", e)))?;

        match value {
            Some(js_value) => {
                let uint8_array = js_sys::Uint8Array::new(&js_value);
                let mut bytes = vec![0u8; uint8_array.length() as usize];
                uint8_array.copy_to(&mut bytes);
                Ok(Some(bytes))
            }
            None => Ok(None),
        }
    }

    /// Remove a key-value pair
    pub async fn remove(&self, key: &[u8]) -> Result<Option<Vec<u8>>, NetabaseError> {
        let old_value = self.get(key).await?;

        let key_js = js_sys::Uint8Array::from(key);

        let tx = self
            .db
            .transaction_on_one_with_mode(&self.tree_name, IdbTransactionMode::Readwrite)
            .map_err(|e| {
                NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
            })?;

        let store = tx
            .object_store(&self.tree_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        store
            .delete(&key_js)
            .map_err(|e| NetabaseError::Storage(format!("Failed to delete value: {:?}", e)))?
            .await
            .map_err(|e| NetabaseError::Storage(format!("Delete request failed: {:?}", e)))?;

        let _ = tx
            .await
            .into_result()
            .map_err(|e| NetabaseError::Storage(format!("Transaction failed: {:?}", e)))?;

        Ok(old_value)
    }

    /// Scan with a key prefix (similar to sled's scan_prefix)
    pub async fn scan_prefix(&self, prefix: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, NetabaseError> {
        let tx = self.db.transaction_on_one(&self.tree_name).map_err(|e| {
            NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
        })?;

        let store = tx
            .object_store(&self.tree_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        let cursor_request = store
            .open_cursor()
            .map_err(|e| NetabaseError::Storage(format!("Failed to open cursor: {:?}", e)))?;

        let cursor = cursor_request
            .await
            .map_err(|e| NetabaseError::Storage(format!("Cursor request failed: {:?}", e)))?;

        let mut results = Vec::new();

        if let Some(mut cursor) = cursor {
            loop {
                let key_js = cursor.key().ok_or_else(|| {
                    NetabaseError::Storage("Failed to get cursor key".to_string())
                })?;
                let value_js = cursor.value();

                let key_array = js_sys::Uint8Array::new(&key_js);
                let mut key_bytes = vec![0u8; key_array.length() as usize];
                key_array.copy_to(&mut key_bytes);

                if key_bytes.starts_with(prefix) {
                    let value_array = js_sys::Uint8Array::new(&value_js);
                    let mut value_bytes = vec![0u8; value_array.length() as usize];
                    value_array.copy_to(&mut value_bytes);

                    results.push((key_bytes, value_bytes));
                }

                let continue_request = cursor.continue_cursor().map_err(|e| {
                    NetabaseError::Storage(format!("Failed to continue cursor: {:?}", e))
                })?;

                let has_next = continue_request.await.map_err(|e| {
                    NetabaseError::Storage(format!("Cursor continue failed: {:?}", e))
                })?;

                if !has_next {
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Clear all entries in the tree
    pub async fn clear(&self) -> Result<(), NetabaseError> {
        let tx = self
            .db
            .transaction_on_one_with_mode(&self.tree_name, IdbTransactionMode::Readwrite)
            .map_err(|e| {
                NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
            })?;

        let store = tx
            .object_store(&self.tree_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        store
            .clear()
            .map_err(|e| NetabaseError::Storage(format!("Failed to clear: {:?}", e)))?
            .await
            .map_err(|e| NetabaseError::Storage(format!("Clear request failed: {:?}", e)))?;

        let _ = tx
            .await
            .into_result()
            .map_err(|e| NetabaseError::Storage(format!("Transaction failed: {:?}", e)))?;

        Ok(())
    }

    /// Iterate over all key-value pairs
    pub async fn iter(&self) -> Result<Vec<(Vec<u8>, Vec<u8>)>, NetabaseError> {
        let tx = self.db.transaction_on_one(&self.tree_name).map_err(|e| {
            NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
        })?;

        let store = tx
            .object_store(&self.tree_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        let cursor_request = store
            .open_cursor()
            .map_err(|e| NetabaseError::Storage(format!("Failed to open cursor: {:?}", e)))?;

        let cursor = cursor_request
            .await
            .map_err(|e| NetabaseError::Storage(format!("Cursor request failed: {:?}", e)))?;

        let mut results = Vec::new();

        if let Some(mut cursor) = cursor {
            loop {
                let key_js = cursor.key().ok_or_else(|| {
                    NetabaseError::Storage("Failed to get cursor key".to_string())
                })?;
                let value_js = cursor.value();

                let key_array = js_sys::Uint8Array::new(&key_js);
                let mut key_bytes = vec![0u8; key_array.length() as usize];
                key_array.copy_to(&mut key_bytes);

                let value_array = js_sys::Uint8Array::new(&value_js);
                let mut value_bytes = vec![0u8; value_array.length() as usize];
                value_array.copy_to(&mut value_bytes);

                results.push((key_bytes, value_bytes));

                let continue_request = cursor.continue_cursor().map_err(|e| {
                    NetabaseError::Storage(format!("Failed to continue cursor: {:?}", e))
                })?;

                let has_next = continue_request.await.map_err(|e| {
                    NetabaseError::Storage(format!("Cursor continue failed: {:?}", e))
                })?;

                if !has_next {
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Get the number of entries in the tree
    pub async fn len(&self) -> Result<usize, NetabaseError> {
        let tx = self.db.transaction_on_one(&self.tree_name).map_err(|e| {
            NetabaseError::Storage(format!("Failed to create transaction: {:?}", e))
        })?;

        let store = tx
            .object_store(&self.tree_name)
            .map_err(|e| NetabaseError::Storage(format!("Failed to get object store: {:?}", e)))?;

        let count = store
            .count()
            .map_err(|e| NetabaseError::Storage(format!("Failed to count: {:?}", e)))?
            .await
            .map_err(|e| NetabaseError::Storage(format!("Count request failed: {:?}", e)))?;

        Ok(count as usize)
    }

    /// Check if the tree is empty
    pub async fn is_empty(&self) -> Result<bool, NetabaseError> {
        Ok(self.len().await? == 0)
    }
}
