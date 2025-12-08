# Netabase Store Refactor Plan

## 1. File Reorganization (Rust Best Practices)

### Current Structure Issues
- Transaction file is too large (~600 lines)
- Mixed concerns in single files
- No clear separation between trait definitions and implementations

### New Structure
```
src/
├── lib.rs                          # Main exports and module declarations
├── error.rs                        # Error types
│
├── traits/                         # All trait definitions
│   ├── mod.rs
│   ├── definition/
│   │   ├── mod.rs                  # NetabaseDefinition trait
│   │   ├── key.rs                  # DefinitionKey trait
│   │   └── discriminant.rs         # DiscriminantName trait
│   ├── model/
│   │   ├── mod.rs                  # NetabaseModelTrait
│   │   ├── key.rs                  # ModelKeyTrait
│   │   ├── relational.rs           # RelationalLink
│   │   └── redb.rs                 # RedbNetabaseModelTrait
│   ├── store/
│   │   ├── mod.rs                  # StoreTrait
│   │   ├── transaction.rs          # ReadTransaction, WriteTransaction
│   │   ├── tree_manager.rs         # TreeManager
│   │   └── record_store.rs         # RecordStore adapter traits
│   └── network/
│       ├── mod.rs
│       ├── record.rs               # Network record wrapper
│       └── provider.rs             # Provider record handling
│
├── types/                          # Concrete types
│   ├── mod.rs
│   ├── record.rs                   # Record, ProviderRecord wrappers
│   └── iterator.rs                 # ChainedRecordIterator
│
└── databases/                      # Backend implementations
    ├── mod.rs
    ├── redb_store/
    │   ├── mod.rs                  # RedbStore struct
    │   ├── transaction/
    │   │   ├── mod.rs
    │   │   ├── read.rs             # RedbReadTransaction
    │   │   ├── write.rs            # RedbWriteTransaction
    │   │   └── queue.rs            # QueueOperation
    │   ├── tables.rs               # Table definitions helper
    │   ├── record_store.rs         # RecordStore implementation
    │   └── provider_store.rs       # ProviderRecord multimap
    └── memory_store/               # Future: in-memory impl
```

## 2. AsRef<str> Refactor

### Changes Required

#### Remove AsRef<str> from main enums:
- ❌ `UserSecondaryKeys: AsRef<str>`
- ❌ `UserRelationalKeys: AsRef<str>`
- ❌ `ProductSecondaryKeys: AsRef<str>`
- etc.

#### Keep AsRef<str> only on discriminants:
- ✅ `UserSecondaryKeysDiscriminants: AsRef<str>`
- ✅ `UserRelationalKeysDiscriminants: AsRef<str>`
- etc.

#### Create tree access enums (no inner types):
```rust
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

## 3. Expanded Boilerplate Example

### New Entities
1. **User** (existing, enhanced)
2. **Product** (existing, enhanced)
3. **Category** (new) - Products belong to categories
4. **Review** (new) - Users review products
5. **Tag** (new) - Products can have multiple tags (many-to-many)
6. **Order** (new) - Users place orders for products

### Relationship Types to Test
- **One-to-Many**: User → Products, Category → Products
- **Many-to-One**: Product → User (creator), Product → Category
- **Many-to-Many**: Product ↔ Tag (via junction)
- **Self-referential**: User → User (followers)
- **Multiple relationships**: Product → User (creator), Product → Review → User (reviewer)

## 4. Proper Redb Implementation

### QueueOperation Improvements
```rust
pub enum QueueOperation<D: NetabaseDefinition> {
    MainTreeInsert {
        table_def: TableDefinition<'static, ...>,
        model_data: D::ModelAssociatedTypes,
        primary_key: D::ModelAssociatedTypes,
    },
    SecondaryKeyInsert {
        table_def: TableDefinition<'static, ...>,
        key_data: D::ModelAssociatedTypes,
        primary_key_ref: D::ModelAssociatedTypes,
    },
    // ... etc
}
```

### Table Definition Management
- Create `TableDefRegistry` to manage all table definitions
- Use discriminants to look up table definitions
- Type-safe table access

## 5. RecordStore Implementation

### Network Record Wrapper
```rust
pub struct NetworkRecord<D: NetabaseDefinition> {
    key: Vec<u8>,                    // Serialized key
    value: D::ModelAssociatedTypes,  // Typed value
    publisher: Option<PeerId>,
    expires: Option<Instant>,
}

impl<D: NetabaseDefinition> NetworkRecord<D> {
    pub fn to_libp2p_record(&self) -> Result<Record>;
    pub fn from_libp2p_record(record: Record) -> Result<Self>;
}
```

### Chained Iterator
```rust
pub struct ChainedRecordIterator<D: NetabaseDefinition> {
    // Iterator that chains all model tables
    model_iterators: Vec<Box<dyn Iterator<Item = NetworkRecord<D>>>>,
    current: usize,
}
```

### Provider Record Multimap
```rust
pub struct ProviderRecordStore {
    // redb multimap: Key -> Set<(PeerId, Instant)>
    db: Database,
}

impl ProviderRecordStore {
    pub fn add_provider(&mut self, key: &Key, provider: ProviderRecord) -> Result<()>;
    pub fn providers(&self, key: &Key) -> Result<Vec<ProviderRecord>>;
    pub fn remove_provider(&mut self, key: &Key, provider: &PeerId) -> Result<()>;
}
```

### Integration
```rust
impl<D: NetabaseDefinition> RecordStore for RedbStore<D> {
    type RecordsIter<'a> = ChainedRecordIterator<D>;
    type ProvidedIter<'a> = ProviderRecordIterator;

    fn get(&self, k: &Key) -> Option<Cow<'_, Record>> {
        // Deserialize key, determine model type, fetch from appropriate table
        // Serialize model back to Record
    }

    fn put(&mut self, r: Record) -> Result<()> {
        // Deserialize Record to determine model type
        // Use ModelAssociatedTypes to route to correct table
        // Use existing put infrastructure
    }

    // ... other methods
}
```

## Implementation Order

1. ✅ Research RecordStore trait
2. Create new file structure
3. Refactor AsRef<str> and create tree access enums
4. Expand boilerplate example
5. Implement proper QueueOperation with table definitions
6. Add NetworkRecord wrapper type
7. Implement ChainedRecordIterator
8. Implement ProviderRecordStore with multimap
9. Implement RecordStore trait for RedbStore
10. Test everything

## Testing Strategy

1. Unit tests for each component
2. Integration test for RecordStore
3. Benchmark for iterator performance
4. Stress test for provider multimap
5. Network simulation test

## Performance Considerations

- Use zero-copy where possible (Cow)
- Lazy deserialization (only when accessed)
- Efficient iteration (streaming, not collecting)
- Multimap index for fast provider lookups

## Type Safety Goals

- No Vec<u8> in public API
- Discriminants for all enum types
- Compile-time table name verification
- Type-safe record routing
