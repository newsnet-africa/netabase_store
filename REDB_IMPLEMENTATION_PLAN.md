# Redb Implementation Plan

## Overview

This document outlines the implementation plan for completing the redb database integration into the NetabaseStore system. The core abstraction layer is complete, and basic CRUD operations work. This plan details the remaining work to make the system production-ready.

---

## Current Status

### ✅ **Completed**

1. **Core Type System**
   - All traits defined (NetabaseDefinition, NetabaseModel, NBTransaction)
   - Key type hierarchy (Primary, Secondary, Relational, Subscription)
   - RelationalLink with 4-variant lifecycle management
   - Permission system (3-level: Definition, Model, Runtime)
   - Table naming and discriminant systems

2. **Redb Integration - Basic CRUD**
   - RedbStore: Database creation and initialization
   - RedbTransaction: Transaction management (read/write)
   - RedbModelCrud: Automatic CRUD for all models
   - ModelOpenTables: Type-safe table access
   - All 4 table types: Main, Secondary, Relational, Subscription

3. **Working Features**
   - ✅ Create models with all key types
   - ✅ Read models by primary key
   - ✅ Update models (with key index updates)
   - ✅ Delete models (with cascade cleanup)
   - ✅ Secondary key indexing
   - ✅ Relational key indexing
   - ✅ Subscription key indexing
   - ✅ Multi-model relationships

4. **Test Coverage**
   - Functional test suite covering all CRUD operations
   - Boilerplate example with User, Post, Category models
   - Cross-definition relationships tested

### 🚧 **In Progress / Incomplete**

1. **NBTransaction Trait Methods** (`src/traits/database/transaction/mod.rs`)
   - Most methods marked as `todo!()`
   - Only basic CRUD implemented via RedbModelCrud

2. **Permission Enforcement**
   - TODO comments in CRUD for relational permission checks
   - Cross-definition permission checks not fully wired

3. **Lifetime Management**
   - Warning about lifetime elision in `begin_transaction`
   - Need to properly handle database/transaction lifetimes

4. **Error Handling**
   - Basic error mapping exists but could be improved
   - Need more specific error types for different failure modes

---

## Implementation Plan

## Phase 1: Complete NBTransaction Implementation

**Priority:** High
**Effort:** Medium

### Files to Modify:
- `src/databases/redb/transaction.rs`
- `src/traits/database/transaction/mod.rs`

### Tasks:

#### 1.1: Implement Conditional Read Operations
```rust
// read_if<M>(&self, predicate: Fn(&M) -> bool) -> NetabaseResult<Vec<M>>
```
- Scan main table for model M
- Apply predicate to filter results
- Return matching models
- Consider performance implications for large datasets

#### 1.2: Implement Range Queries
```rust
// read_range<M>(&self, start: M::Primary, end: M::Primary) -> NetabaseResult<Vec<M>>
```
- Use redb's range query capabilities
- Return all models with primary keys in range [start, end)
- Ensure proper ordering

#### 1.3: Implement Secondary Key Queries
```rust
// read_by_secondary<M, S>(&self, key: S) -> NetabaseResult<Vec<M>>
// where S: NetabaseModelSecondaryKey
```
- Open secondary multimap table for discriminant
- Get all primary keys for secondary key
- Read models from main table
- Return results

#### 1.4: Implement Relational Queries
```rust
// read_related<M, R>(&self, relation_key: R) -> NetabaseResult<Vec<M>>
// where R: NetabaseModelRelationalKey
```
- Open relational multimap table
- Get all primary keys related to relation_key
- Read models from main table
- Check permissions for relational access
- Support both same-definition and cross-definition queries

#### 1.5: Implement Subscription Queries
```rust
// get_subscribers<M>(&self, topic: SubscriptionDiscriminant) -> NetabaseResult<Vec<M::Primary>>
```
- Open subscription multimap table for topic
- Return all subscribed primary keys
- Use SUBSCRIPTION_REGISTRY for validation

#### 1.6: Implement Batch Operations
```rust
// create_batch<M>(&mut self, models: &[M]) -> NetabaseResult<()>
// update_batch<M>(&mut self, models: &[M]) -> NetabaseResult<()>
// delete_batch<M>(&mut self, keys: &[M::Primary]) -> NetabaseResult<()>
```
- Optimize for bulk operations
- Consider transaction size limits
- Maintain atomicity

---

## Phase 2: Permission System Completion

**Priority:** High
**Effort:** Medium

### Files to Modify:
- `src/databases/redb/transaction.rs`
- `src/traits/permissions/model.rs`
- `src/traits/permissions/definition.rs`

### Tasks:

#### 2.1: Relational Permission Enforcement
**Problem:** TODOs in create/update/delete for relational permissions

```rust
// In create_entry, update_entry, delete_entry:
for rel_key in model.get_relational_keys() {
    // TODO: Need mapping from RelationalKeyDiscriminant to target model discriminant
    // Then check permissions.can_access_model(target_discriminant, AccessType::Read)
}
```

**Solution:**
1. Add method to `NetabaseModelRelationalKey`:
   ```rust
   fn target_model_discriminant() -> D::Discriminant;
   ```
2. Implement in each RelationalKey enum variant
3. Use in permission checks before inserting relational entries

#### 2.2: Cross-Definition Permission Checks
**Current:** `can_access_cross_definition` exists but not enforced

**Implementation:**
1. In `open_model_tables`: Check definition-level permissions
2. In relational queries: Validate cross-definition access
3. Add runtime permission ticket validation

#### 2.3: Subscription Permission Model
**Question:** Should subscriptions have separate permissions?

**Recommendation:**
- Subscriptions should check inbound permissions
- Topic publishers should check outbound permissions
- Add `SubscriptionAccessLevel` to permission model

#### 2.4: Permission-Based Table Filtering
**Current:** TODO in open_model_tables to filter relational tables

```rust
// TODO: Filter relational tables based on permissions
// Only open tables for relations we have permission to access
```

**Implementation:**
1. Check `Model::PERMISSIONS.outbound` for each relational discriminant
2. Only create TableDefinitions for permitted relations
3. Return filtered ModelOpenTables

---

## Phase 3: Query Optimization & Indexing

**Priority:** Medium
**Effort:** High

### Tasks:

#### 3.1: Compound Secondary Keys
**Current:** Each secondary key is indexed separately

**Enhancement:**
- Support composite secondary keys (e.g., `Name+Age`)
- Add `NetabaseModelCompoundSecondaryKey` trait
- Generate compound index tables

#### 3.2: Query Planning
**Goal:** Optimize complex queries with multiple conditions

**Implementation:**
1. Add query builder API:
   ```rust
   QueryBuilder::new()
       .model::<User>()
       .where_secondary(UserSecondaryKeys::Age(30))
       .where_related(UserRelationalKeys::Category(cat_id))
       .execute(txn)
   ```
2. Analyze which index to use (secondary vs relational)
3. Implement index intersection/union for AND/OR queries

#### 3.3: Caching Layer
**Goal:** Cache frequently accessed models

**Options:**
1. LRU cache at RedbStore level
2. Cache ModelOpenTables to avoid repeated table opens
3. Cache deserialized models

---

## Phase 4: Cross-Definition Operations

**Priority:** Medium
**Effort:** High

### Tasks:

#### 4.1: Cross-Definition Transactions
**Current:** Transactions are scoped to single definition

**Enhancement:**
1. Add `MultiDefinitionTransaction` type
2. Support opening tables across definitions
3. Maintain separate permission contexts per definition

#### 4.2: RelationalLink Hydration
**Current:** Commented as TODO in RelationalLink

```rust
pub fn hydrate(&self, txn: &impl NBTransaction) -> NetabaseResult<RelationalLink<Hydrated>>
```

**Implementation:**
1. For same-definition: Use txn.read(primary_key)
2. For cross-definition: Need txn.read_cross_definition()
3. Handle permission checks
4. Return Borrowed variant with txn lifetime

#### 4.3: Global Definition Registry
**Goal:** Runtime lookup of definitions by GlobalDefinitionKeys

**Implementation:**
1. Create `DefinitionRegistry` singleton
2. Register all definitions at startup
3. Use for dynamic cross-definition access

---

## Phase 5: Advanced Features

**Priority:** Low
**Effort:** Variable

### 5.1: Subscription System Activation
**Current:** Infrastructure exists but not active

**Implementation:**
1. Add publish/subscribe API:
   ```rust
   txn.publish(topic, data)?;
   txn.get_subscribers(topic)?;
   ```
2. Trigger mechanisms (on create/update/delete)
3. Event queue for async processing

### 5.2: Migration System
**Goal:** Handle schema changes over time

**Features:**
1. Version tracking in database
2. Migration scripts
3. Backwards compatibility checks
4. Safe rollback mechanisms

### 5.3: Backup & Restore
**Goal:** Data export/import

**Implementation:**
1. Export to JSON/CBOR format
2. Import with validation
3. Incremental backup support

### 5.4: Replication
**Goal:** Multi-database sync

**Options:**
1. Master-slave replication
2. Event-sourcing based sync
3. CRDT for conflict resolution

---

## Phase 6: Performance & Production Readiness

**Priority:** High (before production)
**Effort:** Medium

### 6.1: Benchmark Suite
**Create benchmarks for:**
1. CRUD operations (single & batch)
2. Index queries (secondary, relational, subscription)
3. Large dataset performance (1M+ records)
4. Concurrent access patterns

### 6.2: Memory Management
**Optimize:**
1. Reduce allocations in hot paths
2. Use zero-copy where possible (Borrowed variant)
3. Pool frequently used structures

### 6.3: Concurrency
**Current:** Single-threaded via Arc<Database>

**Enhancements:**
1. Read transaction pooling
2. Write transaction queuing
3. Lock-free read paths where possible

### 6.4: Error Handling Improvements
**Add specific error types:**
```rust
pub enum NetabaseError {
    ModelNotFound { model: String, key: String },
    PermissionDenied { operation: String, model: String },
    ConstraintViolation { constraint: String },
    TransactionConflict,
    // ...
}
```

### 6.5: Logging & Observability
**Implement:**
1. Structured logging with tracing
2. Operation metrics (count, latency)
3. Debug logging for permission checks
4. Transaction trace IDs

---

## Phase 7: API Refinement

**Priority:** Medium
**Effort:** Low-Medium

### 7.1: Lifetime Simplification
**Current:** Warning about lifetime elision

**Fix:**
```rust
// Before:
pub fn begin_transaction(&self) -> NetabaseResult<RedbTransaction<D>>

// After:
pub fn begin_transaction(&self) -> NetabaseResult<RedbTransaction<'_, D>>
```

### 7.2: Builder Patterns
**Goal:** Ergonomic APIs

**Examples:**
```rust
// Store builder
RedbStore::builder()
    .path("/tmp/db.redb")
    .permissions(permissions)
    .cache_size(1000)
    .build()?;

// Model builder (for complex models)
User::builder()
    .id(user_id)
    .name("Alice")
    .age(30)
    .partner(partner_link)
    .build()?;
```

### 7.3: Error Context
**Add context to errors:**
```rust
txn.create_redb(&user)
    .context(format!("Creating user with ID: {}", user.id))?;
```

---

## Phase 8: Documentation

**Priority:** High
**Effort:** Medium

### 8.1: API Documentation
- Comprehensive rustdoc for all public APIs
- Examples for common operations
- Link to implementation details

### 8.2: Guides
1. Getting Started Guide
2. Model Definition Guide
3. Permission System Guide
4. Cross-Definition Relationships Guide
5. Performance Tuning Guide

### 8.3: Architecture Documentation
1. Design decisions rationale
2. Type system explanation
3. Lifetime management guide
4. Contribution guidelines

---

## Technical Debt Items

### High Priority
1. Remove `Clone` panic implementations in `RedbStorePermissions`
   - **Solution:** Wrap Database in Arc at construction time
2. Fix unused import warnings
3. Fix dead code warnings for permission fields
4. Implement Default for all TreeNames enums programmatically

### Medium Priority
1. Replace string-based model names in `NetabasePermissions` with type-safe approach
2. Add proper error handling instead of `unwrap()` in example code
3. Refactor ModelOpenTables to avoid tuple-heavy API

### Low Priority
1. Consider const generics for table name arrays
2. Explore procedural macros for boilerplate reduction
3. Evaluate alternative serialization formats (protobuf, capnp)

---

## Testing Strategy

### Unit Tests
- [ ] Test each NBTransaction method individually
- [ ] Permission system unit tests
- [ ] RelationalLink variant conversions
- [ ] Key extraction from models

### Integration Tests
- [x] Basic CRUD (functional_test.rs)
- [ ] Concurrent access patterns
- [ ] Transaction rollback scenarios
- [ ] Permission denial scenarios
- [ ] Cross-definition operations

### Property-Based Tests
- [ ] Model round-trip (create -> read -> verify)
- [ ] Index consistency (secondary/relational match main table)
- [ ] Permission invariants

### Performance Tests
- [ ] Benchmark suite (criterion)
- [ ] Large dataset tests (1M+ records)
- [ ] Memory profiling
- [ ] Concurrency stress tests

---

## Risk Assessment

### High Risk
1. **Permission System Complexity**
   - **Risk:** Easy to introduce security holes
   - **Mitigation:** Extensive testing, security audit

2. **Lifetime Management**
   - **Risk:** Borrow checker issues at scale
   - **Mitigation:** Prototype complex scenarios early

3. **Cross-Definition Access**
   - **Risk:** Type safety violations
   - **Mitigation:** Careful trait design, runtime checks

### Medium Risk
1. **Performance at Scale**
   - **Risk:** Slow queries on large datasets
   - **Mitigation:** Early benchmarking, query optimization

2. **Migration Complexity**
   - **Risk:** Hard to evolve schema
   - **Mitigation:** Design migration system early

### Low Risk
1. **API Ergonomics**
   - **Risk:** Difficult to use
   - **Mitigation:** Gather user feedback, iterate

---

## Timeline Estimate

**Assuming 1 full-time developer:**

- Phase 1 (NBTransaction): **2-3 weeks**
- Phase 2 (Permissions): **2 weeks**
- Phase 3 (Query Optimization): **3-4 weeks**
- Phase 4 (Cross-Definition): **2-3 weeks**
- Phase 5 (Advanced Features): **4-6 weeks** (optional, can be deferred)
- Phase 6 (Production Readiness): **2-3 weeks**
- Phase 7 (API Refinement): **1 week**
- Phase 8 (Documentation): **2 weeks**

**Total Core Features (Phases 1-4, 6-8):** ~14-18 weeks
**With Advanced Features (Phase 5):** ~18-24 weeks

---

## Success Criteria

### MVP (Minimum Viable Product)
- [x] Basic CRUD works
- [ ] All NBTransaction methods implemented
- [ ] Permission system fully enforced
- [ ] Cross-definition relationships work
- [ ] Performance acceptable for 100K records
- [ ] Documentation complete

### Production Ready
- [ ] All phases complete except advanced features
- [ ] Comprehensive test coverage (>80%)
- [ ] Benchmarks show acceptable performance
- [ ] Security audit passed
- [ ] Zero known critical bugs

### Future Goals
- [ ] Subscription system active
- [ ] Migration system working
- [ ] Replication support
- [ ] Multi-database deployments in production

---

## Appendix: Key Design Decisions

### Why 4 Table Types?
- **Main:** Fast O(1) primary key lookups
- **Secondary:** Indexed non-unique fields
- **Relational:** Graph-like queries
- **Subscription:** Pub/sub patterns

### Why RelationalLink with 4 Variants?
- **Dehydrated:** Minimal storage overhead
- **Owned:** Transfer across threads
- **Hydrated:** Application-managed lifetimes
- **Borrowed:** Zero-copy from database

### Why 3-Level Permissions?
- **Definition:** Coarse-grained access control
- **Model:** Fine-grained relation permissions
- **Runtime:** Dynamic, context-dependent checks

### Why Const-based Configuration?
- Compile-time validation
- Zero runtime overhead
- Type-safe table names

---

## Next Steps

1. **Immediate (This Week):**
   - ✅ Run functional tests to validate current implementation
   - ✅ Document current state in this plan
   - Start Phase 1.1: Implement read_if

2. **Short-term (Next 2 Weeks):**
   - Complete Phase 1: NBTransaction implementation
   - Start Phase 2: Permission enforcement

3. **Medium-term (Next Month):**
   - Complete Phases 2-3
   - Begin Phase 6: Benchmarking

4. **Long-term (Next Quarter):**
   - Complete all core phases
   - Evaluate advanced features priority
   - Production deployment planning
