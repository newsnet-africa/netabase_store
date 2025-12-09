# Netabase Store Refactoring Plan

## Executive Summary

The codebase analysis revealed significant backend leakage where `redb`-specific types and traits have contaminated the generic trait layer. This prevents implementing alternative backends (sled, IndexedDB) without major modifications.

## Completed Work

### ✅ Phase 1: Backend Abstraction Layer (COMPLETED)

Created a comprehensive backend abstraction in `src/backend/`:

1. **`backend/error.rs`**: `BackendError` trait for backend-agnostic error handling
2. **`backend/traits.rs`**: Complete set of backend traits:
   - `BackendKey`: Trait for serializable, orderable keys
   - `BackendValue`: Trait for serializable values
   - `BackendReadableTable`: Read-only table operations
   - `BackendWritableTable`: Mutable table operations
   - `BackendReadTransaction`: Read transaction interface
   - `BackendWriteTransaction`: Write transaction interface
   - `BackendStore`: Top-level store interface

**Benefits:**
- Any KV store with tree/table structure can implement these traits
- Type-safe, zero-cost abstraction
- Enables sled, IndexedDB, and other backends
- Maintains transaction semantics across backends

## Remaining Work

### Phase 2: Redb Backend Adapter

**Objective**: Wrap redb to implement the backend traits, isolating redb-specific code.

**Tasks**:
1. Create `src/databases/redb_store/backend_impl.rs`
2. Implement `BackendError` for `redb::Error` types
3. Implement `BackendKey` for redb key types (using bincode/serde)
4. Implement `BackendValue` for redb value types
5. Create wrapper types: `RedbReadableTable`, `RedbWritableTable`
6. Implement `BackendReadTransaction` for redb transactions
7. Implement `BackendWriteTransaction` for redb transactions
8. Implement `BackendStore` for `RedbStore`

**Files to create/modify**:
- New: `src/databases/redb_store/backend_impl.rs` (~500 lines)
- New: `src/databases/redb_store/key_value_impls.rs` (~200 lines)
- Modify: `src/databases/redb_store/mod.rs`

### Phase 3: Error Handling Refactoring

**Objective**: Make `NetabaseError` backend-agnostic.

**Current Problem**:
```rust
// src/error.rs
pub enum NetabaseError {
    RedbError(#[from] RedbError), // HARDCODED TO REDB!
    ...
}
```

**Solution**:
```rust
pub enum NetabaseError {
    BackendError(Box<dyn BackendError>), // GENERIC!
    ...
}
```

**Tasks**:
1. Replace `RedbError` variant with `BackendError`
2. Update all error conversions
3. Add `From<Box<dyn BackendError>>` implementation
4. Update error tests

**Files to modify**:
- `src/error.rs` (~50 line changes)
- `src/databases/redb_store/transaction.rs` (error conversions)
- `src/databases/redb_store/mod.rs` (error conversions)

### Phase 4: Trait Layer Cleanup

**Objective**: Remove all `redb` imports and types from `src/traits/`.

**Current Contamination**:
```rust
// src/traits/model/mod.rs
use redb::{Key, TableDefinition, Value}; // MUST BE REMOVED

pub trait RedbNetabaseModelTrait<D>: NetabaseModelTrait<D>
where
    Self: Value + 'static,  // REDB-SPECIFIC
    Self::PrimaryKey: Key + 'static,  // REDB-SPECIFIC
{
    fn definition<'a>(db: &RedbStore<D>)
        -> TableDefinition<'a, Self::PrimaryKey, Self>;  // REDB TYPE
}
```

**Solution**:

1. **Remove `RedbNetabaseModelTrait` from traits layer**
   - Move to `src/databases/redb_store/model_trait.rs`
   - It's a backend-specific extension, not core trait

2. **Remove `redb::Key` and `redb::Value` bounds**
   - Replace with `BackendKey` and `BackendValue` where needed
   - Most bounds can be removed entirely

3. **Update transaction traits**
   - `ReadTransaction::get()` should not require `redb::Key` bounds
   - `WriteTransaction::put()` should not require `redb::Value` bounds

**Files to modify**:
- `src/traits/model/mod.rs` (remove redb imports, move RedbNetabaseModelTrait)
- `src/traits/store/store/mod.rs` (remove redb imports and bounds)
- `src/traits/store/transaction.rs` (remove redb imports and bounds)
- Create: `src/databases/redb_store/model_trait.rs` (backend-specific trait)

**Estimated changes**: ~300 lines across 4 files

### Phase 5: Comprehensive Testing

**Objective**: Add thorough unit and integration tests for all functionality.

**Test Structure**:
```
tests/
├── unit/
│   ├── backend_traits.rs      # Test backend trait contracts
│   ├── model_trait.rs          # Test model trait implementations
│   ├── definition_trait.rs     # Test definition traits
│   ├── tree_manager.rs         # Test tree management
│   └── subscriptions.rs        # Test subscription features
├── integration/
│   ├── crud_operations.rs      # Test full CRUD workflows
│   ├── secondary_keys.rs       # Test secondary index operations
│   ├── relational_keys.rs      # Test relationship operations
│   ├── subscriptions.rs        # Test subscription sync
│   ├── transactions.rs         # Test transaction semantics
│   └── batch_operations.rs     # Test batch operations
├── backend_impl/
│   ├── redb_backend.rs         # Test redb backend implementation
│   └── mock_backend.rs         # Mock backend for testing abstraction
└── property_based/
    └── model_roundtrip.rs      # Property-based tests with proptest
```

**Test Coverage Goals**:
- **Unit tests**: 90%+ coverage of trait implementations
- **Integration tests**: All example workflows from boilerplate.rs
- **Backend tests**: Verify redb adapter implements all contracts
- **Mock backend**: Verify abstraction works with alternative impl

**Estimated tests**: ~2000-3000 lines

### Phase 6: Backend Abstraction Verification

**Objective**: Prove the abstraction works by implementing a mock/memory backend.

**Create**: `src/databases/memory_store/`
- Simple HashMap-based implementation
- Implements all `Backend*` traits
- Used for testing abstraction
- Demonstrates multi-backend support

**Benefits**:
- Validates abstraction design
- Fast test execution (in-memory)
- Reference implementation for sled/IndexedDB
- Proves backend swappability

**Estimated work**: ~800 lines

## Migration Strategy

### Option A: Big Bang Refactor (Recommended for New Project)
- Complete all phases in one PR
- Breaking changes to API
- Clean slate for backend abstraction

### Option B: Incremental Refactor (Safer for Production)
1. Add backend traits alongside existing code
2. Create redb adapter implementing backend traits
3. Add new API using backend traits
4. Gradually migrate existing code
5. Remove old API in major version bump

### Option C: Parallel Implementation
- Keep existing redb-specific API in `v1` module
- Build new backend-agnostic API in `v2` module
- Users can choose which to use during transition

## Breaking Changes

### API Changes

**Before**:
```rust
use netabase_store::RedbStore;
let store = RedbStore::<Definitions>::new("db.redb")?;
```

**After** (Option 1 - Generic):
```rust
use netabase_store::databases::redb_store::RedbStore;
let store = RedbStore::<Definitions>::new("db.redb")?;
```

**After** (Option 2 - Trait-based):
```rust
use netabase_store::{Store, backends::redb::RedbBackend};
let store = Store::<Definitions, RedbBackend>::new("db.redb")?;
```

### Trait Changes

**Before**:
```rust
impl RedbNetabaseModelTrait<Definitions> for User {
    fn definition(...) -> TableDefinition<...> { ... }
}
```

**After**:
```rust
// Core trait stays the same
impl NetabaseModelTrait<Definitions> for User { ... }

// Backend-specific trait moved to backend module
impl redb_store::ModelSerialization<Definitions> for User { ... }
```

## Estimated Effort

| Phase | Lines of Code | Estimated Time |
|-------|---------------|----------------|
| 1. Backend Abstraction | 300 (✅ Done) | - |
| 2. Redb Adapter | 700 | 2-3 days |
| 3. Error Refactoring | 150 | 1 day |
| 4. Trait Layer Cleanup | 300 | 2-3 days |
| 5. Testing | 2500 | 4-5 days |
| 6. Verification | 800 | 2-3 days |
| **Total** | **~4750 lines** | **~2 weeks** |

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking existing code | HIGH | Comprehensive tests, migration guide |
| Performance regression | MEDIUM | Benchmarks before/after |
| Incomplete abstraction | HIGH | Mock backend implementation |
| Over-engineering | MEDIUM | Start with redb only, expand as needed |

## Recommendations

1. **Complete the refactoring**: The backend leakage is severe enough that incremental fixes will be difficult

2. **Start with Phase 2-4**: Get the redb adapter working with the new abstractions

3. **Add tests continuously**: Don't wait until the end

4. **Document decisions**: Keep architecture decision records (ADRs)

5. **Benchmark**: Measure performance impact of abstraction layer

6. **Version carefully**: This is a breaking change - use semantic versioning

## Success Criteria

- ✅ Zero `redb` imports in `src/traits/`
- ✅ All tests passing with backend abstraction
- ✅ Mock backend successfully implements all traits
- ✅ No performance regression (within 5%)
- ✅ Boilerplate example works unchanged (or with minimal changes)
- ✅ Documentation updated
- ✅ Migration guide written

## Next Steps

1. **Immediate**: Implement redb backend adapter (Phase 2)
2. **Short-term**: Refactor errors and clean trait layer (Phase 3-4)
3. **Medium-term**: Add comprehensive tests (Phase 5)
4. **Long-term**: Implement sled backend as proof of abstraction

## Questions to Consider

1. **API surface**: Keep current API or redesign around backend abstraction?
2. **Serialization**: Use `bincode` for all backends or per-backend choice?
3. **Error handling**: Dynamic dispatch (Box<dyn BackendError>) or static (generic)?
4. **Performance**: Are trait objects acceptable or need zero-cost abstraction?
5. **Versioning**: Major version bump or new crate?

---

**Status**: Phase 1 complete, ready to begin Phase 2
**Last Updated**: 2025-12-09
