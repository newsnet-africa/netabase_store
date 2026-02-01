# In-Memory Backend Architecture

## Overview

Creating mock backends serves multiple purposes:
1. **Testing** - Fast, deterministic tests without disk I/O
2. **Decoupling** - Proves trait system is backend-agnostic  
3. **Development** - Rapid prototyping without redb
4. **Documentation** - Reference implementation
5. **Future-proofing** - Path to IndexedDB, other backends

## Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│  User Code (Models, Definitions, Queries)              │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│  Core Traits (NBStore, Transaction, ModelCrud)         │
│  - Backend agnostic                                     │
│  - Generic over storage implementation                  │
└─────────────────────────────────────────────────────────┘
                        │
           ┌────────────┼────────────┐
           ▼            ▼            ▼
    ┌───────────┐ ┌──────────┐ ┌──────────┐
    │   Redb    │ │  Memory  │ │ IndexedDB│
    │  Backend  │ │  Backend │ │  Backend │
    └───────────┘ └──────────┘ └──────────┘
```

## Proposed Structure

```
src/store/backends/
├── mod.rs
├── redb/                   # Existing redb implementation
│   └── ...
├── memory/                 # New in-memory backend
│   ├── mod.rs
│   ├── store.rs            # MemoryStore<D>
│   ├── transaction/
│   │   ├── mod.rs
│   │   ├── read.rs
│   │   ├── write.rs
│   │   └── crud.rs
│   └── storage/
│       ├── typed.rs        # TypedHashMapBackend (like redb)
│       └── bytes.rs        # ByteVecBackend (generic)
└── indexeddb/              # Future: browser support
    └── mod.rs
```

## Implementation Design

### Abstraction Layer: Storage Backend Trait

```rust
// src/store/backends/mod.rs

/// Core storage operations abstracted from redb
pub trait StorageBackend {
    type ReadTransaction<'db>: ReadTransaction;
    type WriteTransaction<'db>: WriteTransaction;
    
    fn begin_read(&self) -> Result<Self::ReadTransaction<'_>>;
    fn begin_write(&self) -> Result<Self::WriteTransaction<'_>>;
}

/// Read-only table access
pub trait ReadTransaction {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn range(&self, table: &str, start: &[u8], end: &[u8]) -> Result<RangeIter>;
    fn scan(&self, table: &str) -> Result<ScanIter>;
}

/// Read-write table access
pub trait WriteTransaction: ReadTransaction {
    fn insert(&mut self, table: &str, key: &[u8], value: &[u8]) -> Result<()>;
    fn remove(&mut self, table: &str, key: &[u8]) -> Result<()>;
    fn commit(self) -> Result<()>;
    fn abort(self) -> Result<()>;
}

/// Iterator over storage entries
pub trait StorageIterator: Iterator<Item = Result<(Vec<u8>, Vec<u8>)>> {}
```

### Memory Backend: Byte Vector Implementation

```rust
// src/store/backends/memory/storage/bytes.rs

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

/// Pure byte-vector storage (most generic)
#[derive(Clone)]
pub struct ByteVecBackend {
    tables: Arc<RwLock<BTreeMap<String, BTreeMap<Vec<u8>, Vec<u8>>>>>,
}

impl ByteVecBackend {
    pub fn new() -> Self {
        Self {
            tables: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
    
    fn ensure_table(&self, table: &str) {
        let mut tables = self.tables.write().unwrap();
        tables.entry(table.to_string()).or_insert_with(BTreeMap::new);
    }
}

pub struct ByteVecReadTxn {
    snapshot: BTreeMap<String, BTreeMap<Vec<u8>, Vec<u8>>>,
}

impl ReadTransaction for ByteVecReadTxn {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.snapshot
            .get(table)
            .and_then(|t| t.get(key))
            .cloned())
    }
    
    fn range(&self, table: &str, start: &[u8], end: &[u8]) -> Result<RangeIter> {
        let entries = self.snapshot
            .get(table)
            .map(|t| {
                t.range(start.to_vec()..end.to_vec())
                    .map(|(k, v)| Ok((k.clone(), v.clone())))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        
        Ok(RangeIter { entries, index: 0 })
    }
    
    fn scan(&self, table: &str) -> Result<ScanIter> {
        let entries = self.snapshot
            .get(table)
            .map(|t| {
                t.iter()
                    .map(|(k, v)| Ok((k.clone(), v.clone())))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        
        Ok(ScanIter { entries, index: 0 })
    }
}

pub struct ByteVecWriteTxn {
    backend: ByteVecBackend,
    snapshot: BTreeMap<String, BTreeMap<Vec<u8>, Vec<u8>>>,
    mutations: BTreeMap<String, Vec<Mutation>>,
    committed: bool,
}

#[derive(Clone)]
enum Mutation {
    Insert(Vec<u8>, Vec<u8>),
    Remove(Vec<u8>),
}

impl WriteTransaction for ByteVecWriteTxn {
    fn insert(&mut self, table: &str, key: &[u8], value: &[u8]) -> Result<()> {
        self.mutations
            .entry(table.to_string())
            .or_insert_with(Vec::new)
            .push(Mutation::Insert(key.to_vec(), value.to_vec()));
        Ok(())
    }
    
    fn remove(&mut self, table: &str, key: &[u8]) -> Result<()> {
        self.mutations
            .entry(table.to_string())
            .or_insert_with(Vec::new)
            .push(Mutation::Remove(key.to_vec()));
        Ok(())
    }
    
    fn commit(mut self) -> Result<()> {
        let mut tables = self.backend.tables.write().unwrap();
        
        for (table_name, mutations) in &self.mutations {
            let table = tables.entry(table_name.clone()).or_insert_with(BTreeMap::new);
            
            for mutation in mutations {
                match mutation {
                    Mutation::Insert(k, v) => {
                        table.insert(k.clone(), v.clone());
                    }
                    Mutation::Remove(k) => {
                        table.remove(k);
                    }
                }
            }
        }
        
        self.committed = true;
        Ok(())
    }
    
    fn abort(mut self) -> Result<()> {
        self.committed = true; // Mark as handled
        Ok(())
    }
}

impl Drop for ByteVecWriteTxn {
    fn drop(&mut self) {
        if !self.committed {
            eprintln!("Warning: Write transaction dropped without commit or abort");
        }
    }
}

impl StorageBackend for ByteVecBackend {
    type ReadTransaction<'db> = ByteVecReadTxn;
    type WriteTransaction<'db> = ByteVecWriteTxn;
    
    fn begin_read(&self) -> Result<Self::ReadTransaction<'_>> {
        let snapshot = self.tables.read().unwrap().clone();
        Ok(ByteVecReadTxn { snapshot })
    }
    
    fn begin_write(&self) -> Result<Self::WriteTransaction<'_>> {
        let snapshot = self.tables.read().unwrap().clone();
        Ok(ByteVecWriteTxn {
            backend: self.clone(),
            snapshot,
            mutations: BTreeMap::new(),
            committed: false,
        })
    }
}
```

### Memory Backend: Typed HashMap (Redb-like)

```rust
// src/store/backends/memory/storage/typed.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::any::{Any, TypeId};

/// Typed storage similar to redb's type safety
#[derive(Clone)]
pub struct TypedHashMapBackend {
    // Map of (table_name, TypeId) -> type-erased HashMap
    tables: Arc<RwLock<HashMap<(String, TypeId), Box<dyn Any + Send + Sync>>>>,
}

impl TypedHashMapBackend {
    pub fn new() -> Self {
        Self {
            tables: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn open_table<K, V>(&self, name: &str) -> TypedTable<K, V>
    where
        K: redb::Key + 'static,
        V: redb::Value + 'static,
    {
        let key = (name.to_string(), TypeId::of::<(K, V)>());
        
        let mut tables = self.tables.write().unwrap();
        if !tables.contains_key(&key) {
            let map: HashMap<K::SelfType<'static>, V::SelfType<'static>> = HashMap::new();
            tables.insert(key.clone(), Box::new(map));
        }
        
        TypedTable {
            backend: self.clone(),
            name: name.to_string(),
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct TypedTable<K, V> {
    backend: TypedHashMapBackend,
    name: String,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> TypedTable<K, V>
where
    K: redb::Key + 'static,
    V: redb::Value + 'static,
{
    pub fn get(&self, key: &K::SelfType<'_>) -> Option<V::SelfType<'static>> {
        let tables = self.backend.tables.read().unwrap();
        let key_id = (self.name.clone(), TypeId::of::<(K, V)>());
        
        tables
            .get(&key_id)
            .and_then(|table| {
                table
                    .downcast_ref::<HashMap<K::SelfType<'static>, V::SelfType<'static>>>()
                    .and_then(|map| map.get(key).cloned())
            })
    }
    
    pub fn insert(&mut self, key: K::SelfType<'static>, value: V::SelfType<'static>) {
        let mut tables = self.backend.tables.write().unwrap();
        let key_id = (self.name.clone(), TypeId::of::<(K, V)>());
        
        if let Some(table) = tables.get_mut(&key_id) {
            if let Some(map) = table.downcast_mut::<HashMap<K::SelfType<'static>, V::SelfType<'static>>>() {
                map.insert(key, value);
            }
        }
    }
}
```

### Adapter Layer: Bridge to Existing Traits

```rust
// src/store/backends/memory/mod.rs

use super::StorageBackend;
use crate::traits::database::store::NBStore;
use crate::traits::registery::definition::NetabaseDefinition;

/// Memory-backed store implementing NBStore
pub struct MemoryStore<D> {
    backend: ByteVecBackend,
    _phantom: std::marker::PhantomData<D>,
}

impl<D> MemoryStore<D>
where
    D: NetabaseDefinition,
{
    pub fn new() -> Self {
        Self {
            backend: ByteVecBackend::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<D> NBStore<D> for MemoryStore<D>
where
    D: NetabaseDefinition + Clone,
{
    type ReadTransaction<'db> = MemoryReadTransaction<'db, D>;
    type WriteTransaction<'db> = MemoryWriteTransaction<'db, D>;
    
    fn begin_read(&self) -> Result<Self::ReadTransaction<'_>> {
        let txn = self.backend.begin_read()?;
        Ok(MemoryReadTransaction {
            inner: txn,
            _phantom: std::marker::PhantomData,
        })
    }
    
    fn begin_write(&self) -> Result<Self::WriteTransaction<'_>> {
        let txn = self.backend.begin_write()?;
        Ok(MemoryWriteTransaction {
            inner: txn,
            _phantom: std::marker::PhantomData,
        })
    }
}

pub struct MemoryReadTransaction<'db, D> {
    inner: ByteVecReadTxn,
    _phantom: std::marker::PhantomData<&'db D>,
}

pub struct MemoryWriteTransaction<'db, D> {
    inner: ByteVecWriteTxn,
    _phantom: std::marker::PhantomData<&'db D>,
}

// Implement CRUD traits similar to RedbModelCrud
impl<'db, D, M> ModelCrud<M> for MemoryReadTransaction<'db, D>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
{
    fn read(&self, key: &M::PrimaryKey) -> Result<Option<M>> {
        let table_name = M::table_name();
        let key_bytes = postcard::to_stdvec(key)?;
        
        match self.inner.get(table_name, &key_bytes)? {
            Some(value_bytes) => {
                let model = postcard::from_bytes(&value_bytes)?;
                Ok(Some(model))
            }
            None => Ok(None),
        }
    }
    
    // ... other CRUD operations
}
```

## Migration Path

### Phase 1: Extract Backend Trait

1. Create `StorageBackend` trait
2. Implement for redb (wrapper around existing code)
3. No changes to public API

### Phase 2: Implement Memory Backend

1. Create `ByteVecBackend`
2. Create `MemoryStore<D>`
3. Implement all CRUD traits
4. Add tests

### Phase 3: Extract Common Logic

1. Move serialization to shared module
2. Move query logic to shared module
3. Backends only handle raw storage

### Phase 4: Add More Backends

1. `TypedHashMapBackend` for type-safe in-memory
2. `IndexedDBBackend` for browsers
3. `SqliteBackend` for SQL compatibility

## Benefits

### Testing
```rust
#[test]
fn test_with_memory_backend() {
    // Fast, no disk I/O
    let store = MemoryStore::<MyDef>::new();
    // ... test
}

#[test]
fn test_with_redb_backend() {
    // Slower, but tests real persistence
    let (store, _temp) = RedbStore::<MyDef>::new_temporary().unwrap();
    // ... test
}
```

### Decoupling Proof
```rust
// Generic over backend
fn my_algorithm<S, D>(store: &S) -> Result<()>
where
    S: NBStore<D>,
    D: NetabaseDefinition,
{
    let txn = store.begin_read()?;
    // Algorithm works with any backend
    Ok(())
}
```

### Development Speed
```rust
// Instant startup for development
let store = MemoryStore::<MyDef>::new();

// vs redb which needs disk
let (store, _temp) = RedbStore::<MyDef>::new_temporary()?;
```

## Recommendation

**Yes, implement memory backend!**

1. **Short-term**: ByteVec backend for tests
2. **Medium-term**: Extract StorageBackend trait
3. **Long-term**: TypedHashMap for type safety, IndexedDB for browsers

**Priority: High** - Validates architecture, speeds up tests, enables future backends.
