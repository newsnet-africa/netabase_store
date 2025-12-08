# Netabase Store Refactor Status

## Completed Work

### 1. ✅ AsRef<str> Refactor
**Status: Complete and Tested**

#### Changes Made:
- Removed `impl AsRef<str>` from all main secondary and relational key enums
- Only discriminants now implement `AsRef<str>` (via strum's `#[derive(AsRefStr)]`)
- Updated trait bounds in `NetabaseModelKeyTrait` to remove `AsRef<str>` requirement
- Updated `SecondaryKeyTrees` and `RelationalKeyTrees` to only require `DiscriminantName` on discriminants
- Added `DiscriminantName` bounds to `ModelTrees` where clause for proper constraint propagation

#### Files Modified:
- `examples/boilerplate.rs` - Removed 4 `AsRef<str>` implementations
- `src/traits/model/key.rs` - Removed `AsRef<str>` from trait bounds
- `src/traits/store/tree_manager.rs` - Updated struct constraints

#### Result:
✅ Code compiles successfully with only warnings (unused imports)
✅ Tree naming now uses only discriminants (type-safe)
✅ No AsRef<str> on data-containing enums

### 2. ✅ libp2p::kad::RecordStore Research
**Status: Complete**

#### Findings:
The RecordStore trait requires implementation of:
- `get(&self, k: &Key) -> Option<Cow<'_, Record>>`
- `put(&mut self, r: Record) -> Result<()>`
- `remove(&mut self, k: &Key)`
- `records(&self) -> Self::RecordsIter<'_>` - Iterator over all records
- `add_provider(&mut self, record: ProviderRecord)` - Multimap storage
- `providers(&self, key: &Key) -> Vec<ProviderRecord>`
- `provided(&self) -> Self::ProvidedIter<'_>` - Iterator over provided records
- `remove_provider(&mut self, k: &Key, p: &PeerId)`

#### Associated Types:
- `RecordsIter<'a>`: Iterator yielding `Cow<'a, Record>`
- `ProvidedIter<'a>`: Iterator yielding `Cow<'a, ProviderRecord>`

#### Key Requirements for Implementation:
1. **Serialization Layer**: Need to serialize/deserialize models to libp2p Record format
2. **Chained Iterator**: Must iterate over ALL model tables (User, Product, etc.)
3. **Multimap for Providers**: redb multimap for Key -> Set<(PeerId, Instant)>
4. **Type Routing**: Discriminant-based routing to determine which table to query

#### Documentation Sources:
- [RecordStore Trait](https://docs.rs/libp2p/0.12.0/libp2p/kad/record/store/trait.RecordStore.html)
- [Record Module](https://docs.rs/libp2p-kad/0.16.2/libp2p_kad/record/index.html)
- [GitHub Discussion](https://github.com/libp2p/rust-libp2p/discussions/2402)

### 3. ✅ Comprehensive Refactor Plan
**Status: Complete**

Created `REFACTOR_PLAN.md` with:
- Detailed file reorganization structure
- Implementation strategy for all remaining tasks
- Testing strategy
- Performance considerations
- Type safety goals

## Remaining Work

### High Priority

#### 1. Create Tree Access Enums (No Inner Types)
**Purpose**: Separate tree identification from data storage

```rust
// For User model
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum UserSecondaryTreeNames {
    Email,
    Name,
    Age,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum UserRelationalTreeNames {
    CreatedProducts,
}
```

**Benefits**:
- Type-safe tree access without data
- Efficient (Copy instead of Clone)
- Clear separation of concerns
- Can be used in table registries

#### 2. Expand Boilerplate Example
**Add These Entities**:

```rust
// Category - Products belong to categories
pub struct Category {
    pub id: u64,
    pub name: String,
    pub description: String,
}

// Review - Users review products
pub struct Review {
    pub id: u64,
    pub product_id: u128,  // FK to Product
    pub user_id: u64,      // FK to User
    pub rating: u8,
    pub comment: String,
    pub created_at: u64,   // Unix timestamp
}

// Tag - For many-to-many with Product
pub struct Tag {
    pub id: u64,
    pub name: String,
}

// ProductTag - Junction table
pub struct ProductTag {
    pub product_id: u128,
    pub tag_id: u64,
}
```

**Relationships to Test**:
- One-to-Many: Category → Products, User → Reviews
- Many-to-One: Product → Category, Review → User
- Many-to-Many: Product ↔ Tag (via ProductTag junction)
- Multiple relationships: Review → Product, Review → User

#### 3. Fix QueueOperation and Table Definitions
**Current Issue**: Operations use generic types, table definitions not included properly

**Solution**:
```rust
pub enum ConcreteOperation<D: NetabaseDefinition> {
    MainTreeInsert {
        model_discriminant: D::Discriminant,
        model_data: D::ModelAssociatedTypes,
        primary_key: D::ModelAssociatedTypes,
        // Use discriminant to look up table definition
    },
    SecondaryKeyInsert {
        model_discriminant: D::Discriminant,
        key_discriminant: /* Secondary key discriminant */,
        key_data: D::ModelAssociatedTypes,
        primary_key_ref: D::ModelAssociatedTypes,
    },
    // ... etc
}

// Table definition registry
pub struct TableDefRegistry<D: NetabaseDefinition> {
    // Map discriminants to factory functions for table defs
    factories: HashMap<D::Discriminant, Box<dyn Fn() -> TableDefinition<...>>>,
}
```

#### 4. File Reorganization
**New Structure**:
```
src/
├── traits/
│   ├── definition/
│   │   └── discriminant.rs (new) - DiscriminantName trait
│   ├── model/
│   │   └── redb.rs (new) - RedbNetabaseModelTrait
│   ├── store/
│   │   └── record_store.rs (new) - RecordStore adapter
│   └── network/ (new)
│       ├── record.rs - Network record wrapper
│       └── provider.rs - Provider record handling
├── types/ (new)
│   ├── record.rs - Record wrapper
│   └── iterator.rs - ChainedRecordIterator
└── databases/
    └── redb_store/
        ├── transaction/
        │   ├── read.rs (new) - Split from transaction.rs
        │   ├── write.rs (new) - Split from transaction.rs
        │   └── queue.rs (new) - QueueOperation
        ├── tables.rs (new) - Table definition management
        ├── record_store.rs (new) - RecordStore impl
        └── provider_store.rs (new) - ProviderRecord multimap
```

### Medium Priority

#### 5. NetworkRecord Wrapper
```rust
use libp2p::kad::{Record, Key as KadKey};

pub struct NetworkRecord<D: NetabaseDefinition> {
    key: Vec<u8>,                    // Serialized (model discriminant + primary key)
    value: D::ModelAssociatedTypes,  // Typed model
    publisher: Option<PeerId>,
    expires: Option<Instant>,
}

impl<D: NetabaseDefinition> NetworkRecord<D> {
    /// Serialize model to libp2p Record
    pub fn to_libp2p_record(&self) -> NetabaseResult<Record> {
        // Serialize value to bytes
        // Create Record with key, value, publisher, expires
    }

    /// Deserialize libp2p Record to typed model
    pub fn from_libp2p_record(record: Record) -> NetabaseResult<Self> {
        // Extract discriminant from key
        // Deserialize value based on discriminant
        // Route to correct ModelAssociatedTypes variant
    }
}
```

#### 6. Chained Record Iterator
```rust
pub struct ChainedRecordIterator<'a, D: NetabaseDefinition> {
    store: &'a RedbStore<D>,
    model_iterators: Vec<Box<dyn Iterator<Item = NetworkRecord<D>> + 'a>>,
    current_index: usize,
}

impl<'a, D: NetabaseDefinition> Iterator for ChainedRecordIterator<'a, D> {
    type Item = Cow<'a, Record>;

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate through all model tables
        // Serialize each model to Record
        // Return as Cow::Owned
    }
}
```

#### 7. ProviderRecord Multimap Store
```rust
use libp2p::kad::ProviderRecord;
use redb::{Database, MultimapTableDefinition};

/// Stores provider records using redb multimap
/// Key -> Set<(PeerId, Instant, addresses)>
pub struct ProviderRecordStore {
    db: Database,
}

// Use redb's multimap API
const PROVIDERS_TABLE: MultimapTableDefinition<&[u8], &[u8]> =
    MultimapTableDefinition::new("providers");

impl ProviderRecordStore {
    pub fn add_provider(&mut self, key: &KadKey, record: ProviderRecord) -> NetabaseResult<()> {
        // Serialize ProviderRecord
        // Insert into multimap
    }

    pub fn providers(&self, key: &KadKey) -> NetabaseResult<Vec<ProviderRecord>> {
        // Query multimap by key
        // Deserialize all values
    }

    pub fn remove_provider(&mut self, key: &KadKey, peer: &PeerId) -> NetabaseResult<()> {
        // Find and remove specific provider entry
    }

    pub fn provided(&self) -> impl Iterator<Item = ProviderRecord> + '_ {
        // Iterate over all provider records where this node is the provider
    }
}
```

### Low Priority

#### 8. RecordStore Implementation for RedbStore
```rust
impl<D: NetabaseDefinition> RecordStore for RedbStore<D> {
    type RecordsIter<'a> = ChainedRecordIterator<'a, D>;
    type ProvidedIter<'a> = ProviderRecordIterator<'a>;

    fn get(&self, k: &KadKey) -> Option<Cow<'_, Record>> {
        // 1. Deserialize key to extract model discriminant
        // 2. Route to appropriate model table
        // 3. Fetch model
        // 4. Serialize to Record
    }

    fn put(&mut self, r: Record) -> Result<()> {
        // 1. Deserialize record to determine model type
        // 2. Route to ModelAssociatedTypes
        // 3. Use existing put_one infrastructure
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        ChainedRecordIterator::new(self)
    }

    // ... implement remaining methods
}
```

## Testing Strategy

### Unit Tests
- [ ] Test tree access enums
- [ ] Test NetworkRecord serialization
- [ ] Test ProviderRecordStore multimap operations
- [ ] Test ChainedRecordIterator

### Integration Tests
- [ ] Test RecordStore get/put round-trip
- [ ] Test provider record add/retrieve/remove
- [ ] Test iteration over all records
- [ ] Test with expanded boilerplate entities

### Performance Tests
- [ ] Benchmark iterator performance
- [ ] Benchmark provider lookup
- [ ] Test with large datasets (10k+ records)

## Implementation Order

1. ✅ Remove AsRef<str> from main enums
2. ✅ Research RecordStore requirements
3. ✅ Create refactor plan
4. ⏳ Create tree access enums
5. ⏳ Expand boilerplate example
6. ⏳ Reorganize file structure
7. ⏳ Fix QueueOperation with table definitions
8. ⏳ Implement NetworkRecord wrapper
9. ⏳ Implement ChainedRecordIterator
10. ⏳ Implement ProviderRecordStore
11. ⏳ Implement RecordStore trait
12. ⏳ Test everything

## Key Design Decisions

### 1. Discriminant-Based Routing
All operations use discriminants to determine which table to access:
```rust
match model_discriminant {
    DefinitionsDiscriminants::User => /* access User table */,
    DefinitionsDiscriminants::Product => /* access Product table */,
    // ...
}
```

### 2. Type-Safe Serialization
No `Vec<u8>` in internal APIs. Use `ModelAssociatedTypes` wrapper:
```rust
pub enum DefinitionModelAssociatedTypes {
    UserModel(User),
    ProductModel(Product),
    // ... never just Vec<u8>
}
```

### 3. Lazy Deserialization
Records are stored as bytes but only deserialized when accessed:
```rust
pub enum RecordValue<D: NetabaseDefinition> {
    Serialized(Vec<u8>, D::Discriminant),  // Not yet deserialized
    Deserialized(D::ModelAssociatedTypes),  // Fully typed
}
```

### 4. Zero-Copy Where Possible
Use `Cow<'_, T>` for borrowed vs owned data:
```rust
fn get(&self, k: &Key) -> Option<Cow<'_, Record>> {
    // Can return Cow::Borrowed if data is already in memory
    // Or Cow::Owned if deserialized from disk
}
```

## Performance Considerations

### Iterator Efficiency
- Stream records, don't collect all at once
- Lazy deserialization
- Reuse allocations where possible

### Provider Multimap
- Index by key for O(1) lookup
- Prune expired entries periodically
- Limit provider count per key (replication factor)

### Caching Strategy
- Consider caching frequently accessed records
- LRU cache for deserialized models
- Invalidate on write operations

## Migration Path

For existing users:
1. No breaking changes to core API (get/put still work)
2. RecordStore is opt-in (feature flag?)
3. Provide migration guide for custom types
4. Document new patterns

## Next Steps for Implementation

To continue this refactor:

1. **Create tree access enums** in boilerplate.rs
2. **Add Category and Review entities** with relationships
3. **Split transaction.rs** into read/write/queue modules
4. **Create NetworkRecord** type in new types/ directory
5. **Implement basic ProviderRecordStore** with redb multimap
6. **Add RecordStore trait** to store traits
7. **Implement ChainedRecordIterator**
8. **Wire everything together** in RecordStore impl

Each step builds on the previous, maintaining compilation throughout.
