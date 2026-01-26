# Netabase Store: Comprehensive System Architecture

**Netabase Store** is a type-safe, high-performance embedded database library for Rust, designed specifically for decentralized and peer-to-peer (P2P) applications. It builds upon the [redb](https://github.com/cberner/redb) embedded database engine, adding a layer of schema enforcement, automatic indexing, relationship management, and P2P synchronization primitives.

## 1. Core Abstractions & Hierarchy

The architecture follows a strict hierarchy to ensure type safety and logical separation of concerns.

### Repository Level (`NetabaseRepository`)
The highest level of isolation. A **Repository** acts as a hard boundary for data visibility and relationship linking.
- **Purpose**: Security and Multi-tenancy. Models in different repositories cannot reference each other.
- **Mechanism**: The `InRepository<R>` marker trait seals a definition within a specific repository context.
- **Use Case**: A single application might have a `SecureRepo` for user credentials and a `PublicRepo` for blog posts.

### Definition Level (`NetabaseDefinition`)
A logical grouping of related models that share a schema lifecycle.
- **Purpose**: Schema management and organization.
- **Key Components**:
  - **`SubscriptionRegistry`**: Maps string-based "Topics" to specific Models (see Subscriptions below).
  - **`DefKeys`**: A unified enum wrapping the keys of all models in the definition, allowing polymorphic access.
  - **`TreeNames`**: A generated enum mapping every model and index to its underlying `redb` table name.

### Model Level (`NetabaseModel`)
The atomic unit of storage. Models are Rust structs backed by multiple `redb` tables.
- **Storage Strategy**: A single Model is not stored in just one table. It is "shredded" across several auxiliary tables.
- **Versioning**: Models support forward-only migration.

## 2. Data Shapes: The Four Pillars

To implement a networking layer effectively, one must understand the store's data shapes: **Physical**, **Network**, **Structural**, and **Semantic**.

### A. Physical Shape (The Bytes)
At the lowest level, `netabase_store` relies on `postcard` (a #![no_std] focused serializer) for efficient, deterministic serialization.

**Table "Shredding" Strategy:**
A single high-level `Model` is distributed across multiple specialized B-Tree tables:

| Table Type | Key Type | Value Type | Purpose |
| :--- | :--- | :--- | :--- |
| **Main Table** | `PK` | `SerializedModel` | Source of truth. |
| **Secondary** | `SecKey` | `PK` | **Reverse Index**. (Multimap) |
| **Relational** | `PK` | `RelatedKey` | **Graph Edges**. (Multimap) |
| **Blob** | `BlobKey` | `Chunk` | **Heavy Data**. Stores data >60KB in 60KB chunks. |
| **Subscription** | `Topic` | `SubscriptionValue` | **Sync Index**. (Fixed 64-byte width). |

**Blob Storage Note:**
Large fields marked `#[blob]` are replaced in the Main Table by a `BlobItem::Reference(Key)`. The actual data lives in the Blob table. This prevents heavy assets from clogging the main index and allows for partial downloads.

### B. Network Shape (Sync & Addressing)
This layer provides the primitives for P2P synchronization.

**1. Content Addressing & Envelopes**
For models marked `#[subscribe(immutable)]`:
- **Key:** `ModelHash` (SHA-256 of the content).
- **Value:** `Envelope { hash: ModelHash, inner: ModelData }`.
- **Implication:** Automatic deduplication across the network. If two peers generate the same data, they generate the same key.

**2. The Subscription Primitive**
The `Subscription` table is the primary interface for syncing.
- **Value Structure:** Fixed 64-byte width.
  - `[0..32]`: Primary Key Hash (Identity).
  - `[32..64]`: Model Content Hash (State).
- **Merkle Sync:** Because these values are fixed-width and sorted by Topic, you can trivially build a `SubscriptionMerkleTree`.
- **Protocol:** Exchange Merkle Roots -> Find Divergent Hash -> Request Model by Hash.

**3. Libp2p Metadata**
Models implementing `Libp2pModel` inject hidden metadata:
- **Publisher:** `PeerId` (Signer).
- **Expires:** `SystemTime` (TTL).
- **Signature:** (Planned) Authenticity proof.

### C. Structural Shape (The Graph)
Netabase is a graph database. Relationships are first-class citizens.
- **Edges:** Defined by `#[link(Definition, Model)]`.
- **Storage:** Persisted in `Relational` tables (`PK -> FK`).
- **Hydration:** The `RedbModelHydrator` recursively traverses edges.
- **Network Implication:** Syncing a model may require traversing its graph to sync dependencies ("Sync this post AND its author").

### D. Semantic Shape (The Schema)
Peers must agree on the "Shape of the World".
- **RepositorySchema:** A TOML structure describing the entire database.
- **Handshake:** Peers exchange Schema TOML.
- **Negotiation:** `SchemaComparisonResult` determines if peers are `Identical`, `Compatible` (one is strictly newer), or in `Conflict`.

## 3. Networking Implementation Guide

This section maps the abstract shapes to concrete API usage for building the networking layer.

### Step 1: Handshake & Schema Negotiation
Before exchanging data, peers must verify they speak the same language.

```rust
use netabase_store::traits::registery::definition::schema::RepositorySchema;

// 1. Export local schema
let local_schema = store.export_schema_toml()?;

// 2. Send to peer... (Network IO)
// 3. Receive peer schema
let peer_schema: RepositorySchema = toml::from_str(&received_toml)?;

// 4. Compare
let comparison = local_schema_struct.compare(&peer_schema);
match comparison {
    SchemaComparisonResult::Identical => println!("Fast sync enabled"),
    SchemaComparisonResult::Conflict { .. } => panic!("Incompatible peers!"),
    _ => println!("Migration or partial sync required"),
}
```

### Step 2: Topic Synchronization (Merkle Tree)
Efficiently find what data is missing without exchanging the data itself.

```rust
use netabase_store::subscription_hash::SubscriptionMerkleTree;

// 1. Get Merkle Root for a topic
let topic = "chat/general";
let hashes = store.query_by_subscription(topic)?; // Returns Vec<ModelHash>
let tree = SubscriptionMerkleTree::from_hashes(hashes);

// 2. Send Root to peer
send(tree.root_hex());

// 3. If roots differ, exchange full hash list (or sub-tree)
// 4. Calculate diff
let diff = local_tree.diff(&peer_tree);

// 5. Request missing items
for missing_hash in diff.missing_in_self {
    request_model(missing_hash);
}
```

### Step 3: Fetching Data (Content Addressed)
Once you know *what* hash is missing, fetch the actual bytes.

```rust
// Peer A (Responder)
fn handle_request(hash: ModelHash) -> Vec<u8> {
    // 1. Read raw bytes from store (bypassing deserialization for speed)
    let raw_bytes = store.read_raw_by_hash(hash)?;
    raw_bytes
}

// Peer B (Requester)
fn handle_response(data: Vec<u8>) {
    // 1. Verify Hash (Security)
    let calculated_hash = ModelHash::from_bytes(&data);
    assert_eq!(requested_hash, calculated_hash);

    // 2. Deserialize & Validate
    // Note: Use 'Libp2pModel' trait to check signatures if present
    
    // 3. Insert into Local Store
    store.insert_raw(data)?;
}
```

### Step 4: Handling Blobs (Chunking)
For models with `#[blob]` fields, the main sync only transfers the metadata. You must fetch blobs separately.

```rust
// 1. Read model and check for blob references
let model = store.read_entry(id)?;
if let BlobItem::Reference(blob_key) = model.large_file {
    // 2. Request blob chunks
    // The store automatically chunks data >60KB
    let chunks = store.read_blob_chunks(blob_key)?;
}
```

## 4. Configuration & Options (`CrudOptions`)

The `CrudOptions` struct controls how data is accessed, balancing performance against convenience.

- **Pagination**: `limit` / `offset` for UI lists.
- **Hydration**: `fetch_relations` to automatically load linked models (graph traversal).
- **Blob Control**: `strip_blobs` to load lightweight model skeletons without heavy assets.

## 5. Migration & Evolution

Netabase includes a robust forward-only migration system.
- **Runtime Check**: Verifies on-disk schema against compiled schema on startup.
- **Auto-Migration**: Uses `MigrateFrom` trait to lazy-migrate old records during reads.
