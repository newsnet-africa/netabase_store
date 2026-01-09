# API Reference - New Features

This document provides a complete API reference for the newly implemented features in Netabase Store.

## Feature Flags

Netabase Store provides granular feature flags to minimize dependencies and optimize compilation:

```toml
[dependencies]
netabase_store = { version = "0.1", default-features = false, features = ["secondary_keys"] }
```

**Available Features**:
- `secondary_keys` - Enable secondary index support on models
- `relational_keys` - Enable foreign key relationships between models
- `blobs` - Enable blob field support for large binary data
- `repository` - Enable repository pattern API
- `migration` - Enable schema migration system (requires `toml` dependency)
- `libp2p` - Enable libp2p integration for P2P networking (requires `libp2p` dependency)

**Default Features**: All features are enabled by default for convenience.

**Minimal Configuration**: For smallest binary size and fastest compilation, disable default features and only enable what you need.

## Table of Contents

1. [Feature Flags](#feature-flags)
2. [Subscription System](#subscription-system)
3. [Selective Subscription Control](#selective-subscription-control)
4. [Merkle Tree API](#merkle-tree-api)
5. [Content Hashing](#content-hashing)

---

## Subscription System

### Declaring Subscriptions

#### Definition-Level Topics

```rust
#[netabase_macros::netabase_definition(
    MyApp, 
    subscriptions(Topic1, Topic2, Topic3)
)]
mod my_app { ... }
```

**Parameters**:
- `MyApp` - Definition name
- `subscriptions(...)` - List of topic names for this definition

**Generated Types**:
- `MyAppSubscriptions` enum with variants for each topic
- Subscription tables for each (model, topic) pair

---

#### Model-Level Subscriptions

```rust
#[derive(NetabaseModel, ...)]
#[subscribe(Topic1, Topic2)]
pub struct User {
    #[primary_key]
    pub id: String,
    // ... other fields
}
```

**Parameters**:
- `#[subscribe(...)]` - List of topics this model type subscribes to
- Topics must be declared at definition level

**Behavior**:
- ALL instances of the model subscribe to specified topics by default
- Can be overridden per-instance using `create_with_subscriptions()`

---

### Querying Subscriptions

#### `query_by_subscription<M, K>()`

Query all models of a specific type subscribed to a topic.

```rust
pub fn query_by_subscription<M, K>(
    &self,
    subscription_key: &K,
) -> NetabaseResult<Vec<ModelHash>>
where
    M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D>,
    K: Into<D::SubscriptionKeys> + Clone,
```

**Parameters**:
- `M` - Model type to query
- `subscription_key` - Topic to query (e.g., `&MyAppSubscriptions::Topic1`)

**Returns**:
- `Vec<ModelHash>` - List of content hashes
- Hashes are sorted for deterministic ordering

**Example**:
```rust
let txn = store.begin_read()?;
let results = txn.query_by_subscription::<User, _>(
    &MyAppSubscriptions::Topic1
)?;

for hash in results {
    println!("User hash: {}", hash.to_hex());
}
```

**Performance**: O(n) where n is the number of models in the topic (not total models)

---

## Selective Subscription Control

### `create_with_subscriptions()`

Create a model with fine-grained control over which topics it subscribes to.

```rust
pub fn create_with_subscriptions<M>(
    &self,
    model: &M,
    subscription_topics: Option<Vec<D::SubscriptionKeys>>,
) -> NetabaseResult<()>
where
    M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
    // ... trait bounds
```

**Parameters**:
- `model` - Model instance to create
- `subscription_topics` - Optional subscription control:
  - `None` - Subscribe to all model-level topics (default behavior)
  - `Some(vec![...])` - Subscribe to specific topics only
  - `Some(vec![])` - Subscribe to no topics

**Returns**:
- `Ok(())` on success
- `Err(NetabaseError)` on failure

**Examples**:

```rust
// Subscribe to all topics (same as create())
txn.create_with_subscriptions(&user, None)?;

// Subscribe to specific topics
let topics = vec![
    MyAppSubscriptions::Topic1,
    MyAppSubscriptions::Topic3,
];
txn.create_with_subscriptions(&user, Some(topics))?;

// Subscribe to no topics
txn.create_with_subscriptions(&user, Some(vec![]))?;
```

**Use Cases**:
- Privacy control (public vs private users)
- Feature flags (beta access)
- Sharding (different instances sync different topics)
- Access control (role-based topics)

**Behavior**:
- Model is still created in primary table regardless of subscriptions
- Can still be queried by primary key
- Only appears in subscription queries for subscribed topics
- Secondary keys and relational links work normally

---

## Merkle Tree API

### `SubscriptionMerkleTree`

Merkle tree for efficient content verification and synchronization.

**Module**: `netabase_store::subscription_hash`

---

#### `from_hashes()`

Build a Merkle tree from a list of model hashes.

```rust
pub fn from_hashes(hashes: Vec<ModelHash>) -> Self
```

**Parameters**:
- `hashes` - Vector of model content hashes

**Returns**:
- `SubscriptionMerkleTree` instance

**Behavior**:
- Hashes are sorted internally for deterministic tree construction
- Tree is built using SHA-256

**Example**:
```rust
use netabase_store::subscription_hash::SubscriptionMerkleTree;

let results = txn.query_by_subscription::<User, _>(&topic)?;
let hashes: Vec<_> = results.iter().map(|(_, hash)| *hash).collect();

let tree = SubscriptionMerkleTree::from_hashes(hashes);
```

---

#### `root()`

Get the Merkle root hash.

```rust
pub fn root(&self) -> Option<[u8; 32]>
```

**Returns**:
- `Some([u8; 32])` - Root hash if tree is non-empty
- `None` - If tree is empty

**Example**:
```rust
let root = tree.root().unwrap();
println!("Root: {}", hex::encode(root));

// Compare roots for quick sync check
if local_tree.root() == peer_tree.root() {
    println!("Trees are identical");
}
```

---

#### `root_hex()`

Get the Merkle root as a hex string.

```rust
pub fn root_hex(&self) -> Option<String>
```

**Returns**:
- `Some(String)` - Hex-encoded root hash
- `None` - If tree is empty

---

#### `len()` / `is_empty()`

Get tree size.

```rust
pub fn len(&self) -> usize
pub fn is_empty(&self) -> bool
```

**Returns**:
- Number of leaves in the tree

---

#### `hashes()`

Get all hashes in the tree.

```rust
pub fn hashes(&self) -> &[ModelHash]
```

**Returns**:
- Slice of all model hashes (sorted)

---

#### `proof()`

Generate a Merkle proof for a specific hash.

```rust
pub fn proof(&self, hash: &ModelHash) -> Option<MerkleProof<Sha256Hasher>>
```

**Parameters**:
- `hash` - Hash to generate proof for

**Returns**:
- `Some(MerkleProof)` - If hash is in tree
- `None` - If hash is not in tree

**Example**:
```rust
let hash = hashes[0];
let proof = tree.proof(&hash).expect("Hash should be in tree");

// Send proof to peer for verification
```

**Complexity**: O(log n)

---

#### `verify_proof()`

Verify a Merkle proof.

```rust
pub fn verify_proof(
    &self, 
    hash: &ModelHash, 
    proof: &MerkleProof<Sha256Hasher>
) -> bool
```

**Parameters**:
- `hash` - Hash being verified
- `proof` - Merkle proof to verify

**Returns**:
- `true` - Proof is valid
- `false` - Proof is invalid or hash not in tree

**Example**:
```rust
// Verify before accepting data from peer
if peer_tree.verify_proof(&hash, &proof) {
    // Safe to accept this model
    txn.create(&model)?;
} else {
    // Reject invalid data
    eprintln!("Invalid proof!");
}
```

**Complexity**: O(log n)

---

#### `diff()`

Compare two Merkle trees to find differences.

```rust
pub fn diff(&self, other: &SubscriptionMerkleTree) -> SubscriptionDiff
```

**Parameters**:
- `other` - Tree to compare against

**Returns**:
- `SubscriptionDiff` containing missing hashes in each direction

**Example**:
```rust
let diff = local_tree.diff(&peer_tree);

if diff.has_differences() {
    println!("Missing in peer: {:?}", diff.missing_in_other);
    println!("Missing locally: {:?}", diff.missing_in_self);
}
```

**Complexity**: O(n) where n is total unique hashes

---

### `SubscriptionDiff`

Result of comparing two Merkle trees.

**Fields**:
```rust
pub struct SubscriptionDiff {
    pub missing_in_other: Vec<ModelHash>,  // In self, not in other
    pub missing_in_self: Vec<ModelHash>,   // In other, not in self
}
```

**Methods**:

#### `has_differences()`

```rust
pub fn has_differences(&self) -> bool
```

Check if there are any differences between trees.

#### `diff_count()`

```rust
pub fn diff_count(&self) -> usize
```

Total number of differences.

---

## Content Hashing

### `ModelHash`

SHA-256 hash of model content.

**Module**: `netabase_store::subscription_hash`

---

#### `new()`

Create hash from raw bytes.

```rust
pub fn new(bytes: [u8; 32]) -> Self
```

---

#### `from_data<T>()`

Create hash from serializable data.

```rust
pub fn from_data<T: Serialize>(data: &T) -> Result<Self, Box<dyn std::error::Error>>
```

**Example**:
```rust
let hash = ModelHash::from_data(&user)?;
```

---

#### `to_hex()` / `from_hex()`

Convert to/from hex strings.

```rust
pub fn to_hex(&self) -> String
pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError>
```

**Example**:
```rust
let hex = hash.to_hex();
println!("Hash: {}", hex);

let parsed = ModelHash::from_hex(&hex)?;
assert_eq!(hash, parsed);
```

---

#### `as_bytes()`

Get raw hash bytes.

```rust
pub fn as_bytes(&self) -> &[u8; 32]
```

---

## Integration with Existing APIs

### Automatic Hash Management

Hashes are computed and maintained automatically during CRUD operations:

```rust
// Hash computed and stored in subscription tables
txn.create(&user)?;

// Hash recomputed when model changes
txn.update(&user)?;

// Hash removed from subscription tables
txn.delete(&user_id)?;
```

### Usage Patterns

- `create()` still works exactly as before (subscribes to all topics)
- `create_with_subscriptions()` is a new optional method
- Subscription queries return hashes for efficient sync:
  ```rust
  let hashes = txn.query_by_subscription::<User, _>(&topic)?;
  
  for hash in hashes {
      println!("Hash: {}", hash.to_hex());
  }
  ```

---

## Complete Examples

### P2P Sync Example

```rust
use netabase_store::subscription_hash::{SubscriptionMerkleTree, ModelHash};

// Build local tree from subscription
let local_hashes = local_txn.query_by_subscription::<User, _>(&topic)?;
let local_tree = SubscriptionMerkleTree::from_hashes(local_hashes.clone());

// Get peer tree (from network)
let peer_tree = receive_tree_from_peer();

// Quick sync check
if local_tree.root() == peer_tree.root() {
    println!("✓ Already in sync");
    return Ok(());
}

// Find differences
let diff = local_tree.diff(&peer_tree);

// Request missing items
for hash in diff.missing_in_self {
    // Peer sends: (serialized_model, proof)
    let (model_bytes, proof) = request_from_peer(hash);
    
    // Verify before accepting
    if peer_tree.verify_proof(&hash, &proof) {
        let model: User = postcard::from_bytes(&model_bytes)?;
        local_txn.create(&model)?;
    }
}

// Send our items to peer
for hash in diff.missing_in_other {
    let proof = local_tree.proof(&hash).unwrap();
    
    // Check if we have this hash (it's in our local_hashes list)
    if local_hashes.contains(&hash) {
        // Need to retrieve the model data. 
        // Note: For content-addressed models, the hash IS the key.
        // For regular models, you'd need a way to look up by hash (e.g. secondary index)
        // or scan.
        let model = find_model_by_hash(&local_txn, hash)?;
        let model_bytes = postcard::to_allocvec(&model)?;
        send_to_peer(model_bytes, proof);
    }
}
```

### Selective Subscription Example

```rust
// User roles determine topic access
enum UserRole {
    Admin,
    Premium,
    Free,
}

fn create_user_with_role(
    txn: &WriteTransaction<MyApp>,
    user: User,
    role: UserRole,
) -> NetabaseResult<()> {
    let topics = match role {
        UserRole::Admin => vec![
            MyAppSubscriptions::Public,
            MyAppSubscriptions::Premium,
            MyAppSubscriptions::Admin,
            MyAppSubscriptions::Beta,
        ],
        UserRole::Premium => vec![
            MyAppSubscriptions::Public,
            MyAppSubscriptions::Premium,
        ],
        UserRole::Free => vec![
            MyAppSubscriptions::Public,
        ],
    };
    
    txn.create_with_subscriptions(&user, Some(topics))
}

// Query role-specific content
let admin_users = txn.query_by_subscription::<User, _>(
    &MyAppSubscriptions::Admin
)?;
```

---

## Testing

Comprehensive tests are available in:

- `tests/selective_subscriptions.rs` - Selective subscription API
- `tests/comprehensive_table_tests.rs::test_merkle_tree_construction` - Merkle trees
- `src/subscription_hash.rs` - Unit tests for all hash/tree operations

Run tests:
```bash
cargo test --lib subscription_hash
cargo test --test selective_subscriptions
cargo test --test comprehensive_table_tests
```

---

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| `create_with_subscriptions()` | O(t) | t = number of topics |
| `query_by_subscription()` | O(n) | n = models in topic |
| `SubscriptionMerkleTree::from_hashes()` | O(n log n) | Due to sorting |
| `tree.proof()` | O(log n) | Logarithmic tree traversal |
| `tree.verify_proof()` | O(log n) | Logarithmic verification |
| `tree.diff()` | O(n) | Linear hash comparison |
| `tree.root()` | O(1) | Cached result |

---

## Version History

- **v0.1.0** (2026-01-07):
  - ✅ Merkle proof verification fixed and tested
  - ✅ Selective subscription control added
  - ✅ Complete API documentation
  - ✅ Comprehensive test coverage
