# Unified Constructor API Design Recommendations

## Executive Summary

Current implementation has inconsistent constructors across backends. This document proposes a unified API where **all backends share the same method signatures**, with backend-specific behavior handled through configuration types.

---

## 1. Current Inconsistencies

### Problem: Different Methods for Different Backends

```rust
// Different constructor methods - confusing API
SledStore::new(path)
SledStore::temp()

RedbStore::new(path)
RedbStore::open(path)

RedbStoreZeroCopy::new(path)
RedbStoreZeroCopy::open(path)

MemoryStore::new()

IndexedDBStore::new(db_name)
IndexedDBStore::new_with_version(name, version)
```

**Pain Points**:
1. Users must memorize which method is available for each backend
2. Can't write generic code that works across backends
3. Migration between backends requires code changes
4. Testing with different backends requires conditional compilation

---

## 2. Proposed Unified API

### Core Principle: Same method signatures for all backends

```rust
// All backends implement this trait:
pub trait BackendStore<D>: BackendFor<D> + Sized {
    type Config: Default;
    
    // Core methods - all backends support these
    fn new(config: Self::Config) -> Result<Self, NetabaseError>;
    fn open(config: Self::Config) -> Result<Self, NetabaseError>;
    fn temp() -> Result<Self, NetabaseError>;
    
    // Optional transaction API (only for transaction-supporting backends)
    // Discussed separately - may not apply to all
}
```

### Configuration Types

Each backend has a config type capturing its parameters:

```rust
// File-based backends
pub struct FileConfig {
    pub path: PathBuf,
}

// In-memory backends
pub struct MemoryConfig;

// IndexedDB backends
pub struct IndexedDBConfig {
    pub db_name: String,
    pub version: u32,
}

// All backends implement Default with sensible values
impl Default for FileConfig {
    fn default() -> Self {
        Self { path: PathBuf::from("./data") }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self { Self }
}

impl Default for IndexedDBConfig {
    fn default() -> Self {
        Self {
            db_name: "netabase".to_string(),
            version: 1,
        }
    }
}
```

---

## 3. Implementation by Backend

### 3.1 SledStore (File-based)

```rust
impl<D> BackendStore<D> for SledStore<D>
where
    D: NetabaseDefinitionTrait + ToIVec,
    <D as IntoDiscriminant>::Discriminant: NetabaseDiscriminant,
{
    type Config = FileConfig;
    
    fn new(config: Self::Config) -> Result<Self, NetabaseError> {
        // Current behavior: SledStore::new() creates or opens
        let db = sled::open(&config.path)?;
        Ok(Self {
            db,
            trees: D::Discriminant::iter().collect(),
        })
    }
    
    fn open(config: Self::Config) -> Result<Self, NetabaseError> {
        // Same as new() - sled doesn't distinguish
        Self::new(config)
    }
    
    fn temp() -> Result<Self, NetabaseError> {
        // Current behavior: SledStore::temp()
        let config = sled::Config::new().temporary(true);
        let db = config.open()?;
        Ok(Self {
            db,
            trees: D::Discriminant::iter().collect(),
        })
    }
}
```

**Usage**:
```rust
// Create with defaults
let store = SledStore::<MyDef>::new(FileConfig::default())?;

// Create with custom path
let store = SledStore::<MyDef>::new(FileConfig {
    path: PathBuf::from("./custom_path")
})?;

// Temp for testing
let store = SledStore::<MyDef>::temp()?;

// Same as new() for sled
let store = SledStore::<MyDef>::open(FileConfig::default())?;
```

### 3.2 RedbStore (File-based, separate new/open)

```rust
impl<D> BackendStore<D> for RedbStore<D>
where
    D: NetabaseDefinitionTrait + ToIVec,
    <D as IntoDiscriminant>::Discriminant: NetabaseDiscriminant,
{
    type Config = FileConfig;
    
    fn new(config: Self::Config) -> Result<Self, NetabaseError> {
        // Creates fresh database
        let db = Database::create(&config.path)?;
        Ok(Self {
            db: Arc::new(db),
            #[cfg(feature = "redb")]
            tables: D::tables(),
            trees: D::Discriminant::iter().collect(),
        })
    }
    
    fn open(config: Self::Config) -> Result<Self, NetabaseError> {
        // Opens existing or creates
        let db = Database::open(&config.path)?;
        Ok(Self {
            db: Arc::new(db),
            #[cfg(feature = "redb")]
            tables: D::tables(),
            trees: D::Discriminant::iter().collect(),
        })
    }
    
    fn temp() -> Result<Self, NetabaseError> {
        // Future enhancement: in-memory database support
        // Currently not supported by redb
        Err(NetabaseError::Storage(
            "RedbStore does not support temporary databases yet".to_string()
        ))
    }
}
```

**Usage**:
```rust
// Create fresh database
let store = RedbStore::<MyDef>::new(FileConfig::default())?;

// Open existing or create
let store = RedbStore::<MyDef>::open(FileConfig::default())?;

// Temp not supported (for now)
let store = RedbStore::<MyDef>::temp()?;  // Error
```

### 3.3 RedbStoreZeroCopy (File-based with transactions)

```rust
impl<D> BackendStore<D> for RedbStoreZeroCopy<D>
where
    D: NetabaseDefinitionTrait,
{
    type Config = FileConfig;
    
    fn new(config: Self::Config) -> Result<Self, NetabaseError> {
        // Remove existing for clean start
        let _ = std::fs::remove_file(&config.path);
        let db = Database::create(&config.path)?;
        Ok(Self {
            db: Arc::new(db),
            _phantom: PhantomData,
        })
    }
    
    fn open(config: Self::Config) -> Result<Self, NetabaseError> {
        // FIX: This currently has a bug (uses create instead of open)
        let db = Database::open(&config.path)?;  // FIXED
        Ok(Self {
            db: Arc::new(db),
            _phantom: PhantomData,
        })
    }
    
    fn temp() -> Result<Self, NetabaseError> {
        // Future: Could support in-memory database
        Err(NetabaseError::Storage(
            "RedbStoreZeroCopy does not support temporary databases yet".to_string()
        ))
    }
}
```

**Usage**:
```rust
// Create fresh database
let store = RedbStoreZeroCopy::<MyDef>::new(FileConfig::default())?;

// Open existing database (FIXED BUG)
let store = RedbStoreZeroCopy::<MyDef>::open(FileConfig::default())?;

// Transaction API still explicit
let mut txn = store.begin_write()?;
let mut tree = txn.open_tree::<User>()?;
// ... operations ...
txn.commit()?;
```

### 3.4 MemoryStore (In-memory)

```rust
impl<D> BackendStore<D> for MemoryStore<D>
where
    D: NetabaseDefinitionTrait + ToIVec,
    <D as IntoDiscriminant>::Discriminant: NetabaseDiscriminant,
{
    type Config = MemoryConfig;
    
    fn new(_config: Self::Config) -> Result<Self, NetabaseError> {
        // Config ignored - always creates in-memory
        Ok(Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            trees: D::Discriminant::iter().collect(),
        })
    }
    
    fn open(_config: Self::Config) -> Result<Self, NetabaseError> {
        // Same as new() - always creates fresh (no persistence)
        Self::new(_config)
    }
    
    fn temp() -> Result<Self, NetabaseError> {
        // Same as new() - already temporary by nature
        Self::new(MemoryConfig::default())
    }
}
```

**Usage**:
```rust
// Create in-memory store
let store = MemoryStore::<MyDef>::new(MemoryConfig::default())?;

// open() is same as new() for memory
let store = MemoryStore::<MyDef>::open(MemoryConfig::default())?;

// temp() is same as new()
let store = MemoryStore::<MyDef>::temp()?;
```

### 3.5 IndexedDBStore (Browser-based)

```rust
impl<D> BackendStore<D> for IndexedDBStore<D>
where
    D: NetabaseDefinitionTrait + ToIVec,
    D::Discriminant: DiscriminantBounds,
{
    type Config = IndexedDBConfig;
    
    async fn new_async(config: Self::Config) -> Result<Self, NetabaseError> {
        let mut db_req = IdbDatabase::open_u32(&config.db_name, config.version)?;
        
        db_req.set_on_upgrade_needed(Some(|evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
            // Create object stores for each model type
            let db = evt.db();
            for disc in D::Discriminant::iter() {
                let store_name: String = disc.as_ref().to_string();
                if !db.object_store_names().any(|n| n == store_name) {
                    db.create_object_store(&store_name)?;
                }
                let sec_store = format!("{}_secondary", store_name);
                if !db.object_store_names().any(|n| n == sec_store) {
                    db.create_object_store(&sec_store)?;
                }
            }
            Ok(())
        }));
        
        let db = db_req.await?;
        Ok(Self {
            db: Arc::new(db),
            db_name: config.db_name,
            trees: D::Discriminant::iter().collect(),
            _phantom: PhantomData,
        })
    }
    
    async fn open_async(config: Self::Config) -> Result<Self, NetabaseError> {
        // IndexedDB always creates if not exists
        Self::new_async(config).await
    }
    
    async fn temp_async() -> Result<Self, NetabaseError> {
        // Browser context doesn't really have "temp" - everything persists
        // Use a special database name to indicate temporary intent
        Self::new_async(IndexedDBConfig {
            db_name: "__temp__netabase".to_string(),
            version: 1,
        }).await
    }
}
```

**Usage**:
```rust
// Create with defaults
let store = IndexedDBStore::<MyDef>::new(IndexedDBConfig::default()).await?;

// Create with custom config
let store = IndexedDBStore::<MyDef>::new(IndexedDBConfig {
    db_name: "my_app_db".to_string(),
    version: 2,
}).await?;

// Open is same as new() (browser doesn't distinguish)
let store = IndexedDBStore::<MyDef>::open(IndexedDBConfig::default()).await?;

// Temp uses special naming
let store = IndexedDBStore::<MyDef>::temp().await?;
```

---

## 4. Migration Path

### Phase 1: Add New Trait (Backwards compatible)

Keep existing constructors, add new `BackendStore` trait alongside.

```rust
// Existing (keep for compatibility)
impl<D> SledStore<D> {
    pub fn new(path: P) -> Result<Self> { ... }
    pub fn temp() -> Result<Self> { ... }
}

// New (alongside existing)
impl<D> BackendStore<D> for SledStore<D> {
    type Config = FileConfig;
    fn new(config: Self::Config) -> Result<Self> { ... }
    // etc.
}
```

### Phase 2: Deprecate Old Methods

```rust
#[deprecated(since = "0.2.0", note = "use BackendStore::new() instead")]
pub fn new(path: P) -> Result<Self> { ... }
```

### Phase 3: Remove Old Methods (Major version)

Clean up in 1.0 release.

---

## 5. Generic Code Examples

With unified API, users can write generic code:

### Example 1: Backend-agnostic store creation

```rust
// Old approach - backend-specific
#[cfg(feature = "sled")]
fn create_store() -> Result<SledStore<MyDef>, NetabaseError> {
    SledStore::new("./data")
}

#[cfg(feature = "redb")]
fn create_store() -> Result<RedbStore<MyDef>, NetabaseError> {
    RedbStore::open("./data.redb")
}


// New approach - unified trait
fn create_store<B: BackendStore<MyDef>>(
    config: B::Config
) -> Result<B, NetabaseError> {
    B::open(config)  // Same method for all!
}

// Usage:
let sled_store = create_store::<SledStore<_>>(FileConfig::default())?;
let redb_store = create_store::<RedbStore<_>>(FileConfig::default())?;
let mem_store = create_store::<MemoryStore<_>>(MemoryConfig::default())?;
```

### Example 2: Environment-based backend selection

```rust
fn create_backend(backend_type: &str, path: &str) 
    -> Result<Box<dyn Any>, NetabaseError> 
{
    match backend_type {
        "sled" => {
            let config = FileConfig { path: PathBuf::from(path) };
            Ok(Box::new(SledStore::<MyDef>::new(config)?) as Box<dyn Any>)
        }
        "redb" => {
            let config = FileConfig { path: PathBuf::from(path) };
            Ok(Box::new(RedbStore::<MyDef>::open(config)?) as Box<dyn Any>)
        }
        "memory" => {
            Ok(Box::new(MemoryStore::<MyDef>::temp()?) as Box<dyn Any>)
        }
        _ => Err(NetabaseError::Storage("Unknown backend".to_string()))
    }
}
```

### Example 3: Testing with multiple backends

```rust
#[test_case(MemoryConfig::default())]
#[test_case(FileConfig { path: PathBuf::from("./test_sled") })]
fn test_store_with_backend<C: Default + 'static>(config: C) {
    // Single test code works with any compatible backend
}
```

---

## 6. Transaction API Consideration

The current RedbStoreZeroCopy transaction API is good and shouldn't change:

```rust
// Keep this as-is for RedbStoreZeroCopy
let mut txn = store.begin_write()?;
let mut tree = txn.open_tree::<User>()?;
tree.put(user)?;
txn.commit()?;
```

Other backends can optionally implement similar traits without breaking the unified constructor API.

---

## 7. Implementation Checklist

- [ ] Define `BackendStore` trait with Config pattern
- [ ] Create `FileConfig`, `MemoryConfig`, `IndexedDBConfig` types
- [ ] Implement `BackendStore` for all 5 backends
- [ ] Maintain backwards compatibility with old methods
- [ ] Add deprecation notices to old methods
- [ ] Update documentation with unified examples
- [ ] Provide migration guide for users
- [ ] Add integration tests using generic code
- [ ] Remove old methods in major version bump

---

## 8. Benefits of This Approach

1. **Maximum API overlap** - All backends use same method signatures
2. **Easy migration** - Switch backends by changing type parameter
3. **Generic code** - Write backend-agnostic functions
4. **Clear semantics** - `new()` always creates, `open()` always opens
5. **Extensible** - Add new backends without breaking API
6. **Backwards compatible** - Existing code continues to work
7. **Configuration-driven** - Flexible parameter passing
8. **Type-safe** - Each backend has appropriate Config type

---

## 9. Code Duplication Management

Each backend's `BackendStore` impl follows same pattern but can't be fully DRY. Use helper functions:

```rust
// Helper for file-based backends
fn open_file_db(path: &Path) -> Result<sled::Db, NetabaseError> {
    Ok(sled::open(path)?)
}

// Then use in impl:
impl<D> BackendStore<D> for SledStore<D> {
    type Config = FileConfig;
    fn new(config: Self::Config) -> Result<Self, NetabaseError> {
        let db = open_file_db(&config.path)?;
        Ok(Self {
            db,
            trees: D::Discriminant::iter().collect(),
        })
    }
    // ...
}
```

