# Implementation Plan for TODO Items

**Date**: 2026-01-07
**Status**: Ready to implement
**Total Tasks**: 5 major items

---

## Task 1: Optimize Relational Queries with Multimap (4-6 hours)

### Current State
- Relational queries use O(n) table scan
- Each query iterates through entire relational index table

### Target State
- Use multimap pattern: `RelationalKey -> Vec<PrimaryKey>`
- O(log n) lookups similar to secondary keys
- Support `Vec<RelationalLink<T>>` patterns

### Implementation Steps

## Task 2: Complete Merkle Proof Verification (1-2 hours)

### Current State
- Merkle tree construction works ✅
- Proof generation works ✅
- Proof verification commented out (signature issue)

### Target State
- Working `verify_proof()` method
- Integration with P2P sync methods

### Implementation Steps

#### 2.1 Fix Merkle Proof Verification
**File**: `tests/comprehensive_table_tests.rs` (line 376)

**Current**:
```rust
// assert!(tree.verify_proof(&hash, &proof, tree.len()));
```

**Investigation needed**:
- Check `rs_merkle` crate documentation
- Verify correct signature for `verify_proof()`
- May need proof index instead of tree length

**Likely fix**:
```rust
let proof_index = hashes.iter().position(|h| h == &hash).unwrap();
assert!(tree.verify_proof(&hash, &proof, proof_index));
```

#### 2.2 Add to Public API
**File**: `src/subscription_hash.rs` or new `src/merkle_sync.rs`

```rust
pub fn verify_merkle_proof(
    hash: &ModelHash,
    proof: &MerkleProof,
    root: &[u8],
    index: usize,
) -> bool {
    // Wrapper around rs_merkle verification
}
```

---

## Task 3: Add Selective Subscription Insertion (3-4 hours)

### Current State
- All instances subscribe to ALL model-level topics
- No way to opt-out or selectively subscribe

### Target State
- Default: Subscribe to all model topics
- Optional: Provide list of specific topics for this instance
- API: `create_with_subscriptions(model, topics: Option<Vec<TopicKey>>)`

### Implementation Steps

#### 3.1 Add Create Variant
**File**: `src/databases/redb/transaction/mod.rs`

```rust
pub fn create_with_subscriptions<M>(
    &self,
    model: &M,
    topics: Option<Vec<D::SubscriptionKeys>>,
) -> NetabaseResult<()>
where M: RedbModelCrud<'db, D> + ...,
{
    // If topics is None, use all model subscriptions (default behavior)
    // If topics is Some(...), only subscribe to those specific topics
    
    let subscription_topics = match topics {
        None => model.get_subscription_keys(), // All topics
        Some(custom) => custom, // Specific topics
    };
    
    // Pass subscription_topics to create logic
    M::create_with_subscriptions(model, &tables, subscription_topics)?;
}
```

#### 3.2 Update Model CRUD Trait
**File**: Model boilerplate generation

```rust
fn create_with_subscriptions(
    model: &Self,
    tables: &ModelTables,
    topics: Vec<SubscriptionKey>,
) -> NetabaseResult<()> {
    // Insert to primary table
    // Insert to secondary indexes
    // Insert to relational indexes
    // Only insert to subscription tables in `topics` list
    for topic in topics {
        if let Some(table) = get_subscription_table(topic) {
            table.insert(pk, hash)?;
        }
    }
}
```

#### 3.3 Builder Pattern Integration
**File**: `src/query/mod.rs` or new `src/builder/create.rs`

Optional: Use `derive_builder` or similar

```rust
CreateBuilder::new(user)
    .subscribe_to(vec![Topic1, Topic3])
    .execute(&txn)?;
```

#### 3.4 Testing
- Test default behavior (all topics)
- Test selective subscription
- Test empty subscription list
- Test update/delete with selective subscriptions

---

## Task 4: Definition-Level Subscription Queries (6-8 hours)

### Current State
- Model-level queries work: `query_by_subscription::<User>(&Topic1)`
- No cross-model aggregation

### Target State
- Definition-level queries: `Definition::query_subscription(&txn, &Topic1)`
- Returns iterator that chains all model table iterators
- Returns `Vec<(Definition, ModelHash)>` with models wrapped in Definition enum

### Implementation Steps

#### 4.1 Track Subscriptions in Definition
**File**: `netabase_macros/src/generators/definition.rs`

Generate metadata about which models subscribe to which topics:

```rust
impl Definition {
    pub const fn subscription_models(
        topic: &DefinitionSubscriptions
    ) -> &'static [&'static str] {
        match topic {
            DefinitionSubscriptions::Topic1 => &["User", "Post"],
            DefinitionSubscriptions::Topic2 => &["User"],
            DefinitionSubscriptions::Topic3 => &["Post", "Comment"],
            DefinitionSubscriptions::Topic4 => &["Comment"],
        }
    }
}
```

#### 4.2 Create Chained Iterator
**File**: `src/databases/redb/subscription_iter.rs` (new file)

```rust
pub struct SubscriptionIterator<'a, D: NetabaseDefinition> {
    model_iters: Vec<Box<dyn Iterator<Item = (D, ModelHash)> + 'a>>,
    current_index: usize,
}

impl<'a, D> Iterator for SubscriptionIterator<'a, D> {
    type Item = (D, ModelHash);
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.current_index < self.model_iters.len() {
            if let Some(item) = self.model_iters[self.current_index].next() {
                return Some(item);
            }
            self.current_index += 1;
        }
        None
    }
}
```

#### 4.3 Generate Definition Query Method
**File**: `netabase_macros/src/generators/definition.rs`

```rust
impl Definition {
    pub fn query_subscription<'db>(
        txn: &'db ReadTransaction<Definition>,
        topic: &DefinitionSubscriptions,
    ) -> NetabaseResult<SubscriptionIterator<'db, Definition>> {
        let mut iters = Vec::new();
        
        // For each model subscribed to this topic
        match topic {
            DefinitionSubscriptions::Topic1 => {
                // Query User subscription table
                let user_results = txn.query_by_subscription::<User, _>(topic)?;
                iters.push(Box::new(
                    user_results.into_iter().map(|(u, h)| (Definition::User(u), h))
                ));
                
                // Query Post subscription table
                let post_results = txn.query_by_subscription::<Post, _>(topic)?;
                iters.push(Box::new(
                    post_results.into_iter().map(|(p, h)| (Definition::Post(p), h))
                ));
            }
            // ... other topics
        }
        
        Ok(SubscriptionIterator {
            model_iters: iters,
            current_index: 0,
        })
    }
}
```

#### 4.4 Optimize with Chained Redb Iterators
Use redb's `Range<'a, K, V>` directly instead of collecting:

```rust
pub enum DefinitionSubscriptionIter<'a> {
    Topic1 {
        user_iter: Range<'a, UserID, ModelHash>,
        post_iter: Range<'a, PostID, ModelHash>,
        state: TopicIterState,
    },
    // ... other variants
}

enum TopicIterState {
    User,
    Post,
    Done,
}
```

This avoids collecting all results into memory.

#### 4.5 Testing
- Test cross-model iteration
- Verify deterministic ordering
- Test with large datasets
- Verify lazy evaluation (doesn't load all at once)

---

## Task 5: P2P Merkle Synchronization (8-12 hours)

### Current State
- Merkle tree construction works
- Model hashing works
- No sync protocol

### Target State
- Compare merkle roots between peers
- Identify missing/different items
- Request specific models by hash

### Implementation Steps

#### 5.1 Create Sync Protocol Module
**File**: `src/sync/protocol.rs` (new)

```rust
pub struct SyncProtocol<'db, D: NetabaseDefinition> {
    txn: &'db ReadTransaction<'db, D>,
}

impl<'db, D: NetabaseDefinition> SyncProtocol<'db, D> {
    /// Compare local and peer merkle roots
    pub fn compare_roots(
        &self,
        local_root: &[u8],
        peer_root: &[u8],
    ) -> SyncStatus {
        if local_root == peer_root {
            SyncStatus::InSync
        } else {
            SyncStatus::OutOfSync
        }
    }
    
    /// Get missing items by comparing trees
    pub fn get_missing_items(
        &self,
        local_tree: &MerkleTree,
        peer_tree: &MerkleTree,
    ) -> Vec<ModelHash> {
        // Compare branches, identify differences
        // Return hashes that peer has but we don't
        todo!()
    }
    
    /// Build request for missing models
    pub fn build_sync_request(
        &self,
        missing_hashes: Vec<ModelHash>,
    ) -> SyncRequest {
        SyncRequest {
            hashes: missing_hashes,
        }
    }
}

pub enum SyncStatus {
    InSync,
    OutOfSync,
}

pub struct SyncRequest {
    pub hashes: Vec<ModelHash>,
}
```

#### 5.2 Integrate Proof Verification
Use verified merkle proofs to validate peer data:

```rust
impl<'db, D> SyncProtocol<'db, D> {
    /// Verify a model matches claimed hash and proof
    pub fn verify_model(
        &self,
        model: &D,
        hash: &ModelHash,
        proof: &MerkleProof,
        root: &[u8],
    ) -> bool {
        // 1. Hash the model
        let computed_hash = model.compute_hash();
        if &computed_hash != hash {
            return false;
        }
        
        // 2. Verify merkle proof
        verify_merkle_proof(hash, proof, root, /* index */)
    }
}
```

#### 5.3 Subscription-Specific Sync
**File**: `src/sync/subscription.rs` (new)

```rust
impl Definition {
    /// Sync a specific subscription topic with peer
    pub fn sync_subscription<'db>(
        txn: &'db ReadTransaction<Definition>,
        topic: &DefinitionSubscriptions,
        peer_root: &[u8],
    ) -> NetabaseResult<SyncPlan> {
        // 1. Build local merkle tree for topic
        let local_tree = Self::subscription_merkle_tree(txn, topic)?;
        
        // 2. Compare roots
        if local_tree.root() == peer_root {
            return Ok(SyncPlan::InSync);
        }
        
        // 3. Determine what we need
        // (In real implementation, would exchange proofs with peer)
        Ok(SyncPlan::NeedSync {
            missing: vec![],
            different: vec![],
        })
    }
}

pub enum SyncPlan {
    InSync,
    NeedSync {
        missing: Vec<ModelHash>,
        different: Vec<ModelHash>,
    },
}
```

#### 5.4 Testing
- Test root comparison
- Test hash diff calculation
- Test sync request building
- Integration test with two databases
- Benchmark sync performance

---

## Implementation Order

1. **Task 2** (Merkle Proof) - 1-2 hours - Prerequisite for Task 5
2. **Task 1** (Relational Multimap) - 4-6 hours - Independent, high value
3. **Task 3** (Selective Subscriptions) - 3-4 hours - Independent, useful feature
4. **Task 4** (Definition Queries) - 6-8 hours - Prerequisite for Task 5
5. **Task 5** (P2P Sync) - 8-12 hours - Capstone feature

**Total Estimated Time**: 22-32 hours

---

## Notes

- All tasks are backward compatible
- Existing tests should continue passing
- Each task can be implemented incrementally
- Task 5 (Transaction-level convenience) is removed as unnecessary per user feedback

---

## Success Criteria

✅ All 206 tests still passing
✅ New tests for each feature
✅ Performance improvements measurable
✅ Documentation updated
✅ Examples added

