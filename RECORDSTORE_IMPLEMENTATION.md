# Implementing libp2p RecordStore Trait for Netabase Store

This document explains how to implement the libp2p `RecordStore` trait for any Netabase store using the Definition enum as the serialization layer.

## Core Design Philosophy

**Key Principle**: Models are stored directly in their native format. The `Definition` enum acts as the serialization/deserialization boundary between the network layer and storage layer.

### Data Flow

```
Network Layer (libp2p)
         ↓
    Record (key + value bytes)
         ↓
    Deserialize value → Definition enum
         ↓
    Extract inner model
         ↓
Storage Layer (Netabase) - Model stored directly
         ↑
    Fetch model → Wrap in Definition
         ↑
    Serialize Definition → Record value
         ↑
Network Layer (libp2p)
```

## Implementation Steps

### Step 1: Add Serialization to Definition Enum

Your Definition enum must support serialization:

```rust
#[derive(Debug, Clone, EnumDiscriminants, serde::Serialize, serde::Deserialize)]
#[strum_discriminants(name(DefinitionsDiscriminants))]
#[strum_discriminants(derive(EnumIter, AsRefStr, Hash))]
pub enum Definitions {
    User(User),
    Product(Product),
    Category(Category),
    Review(Review),
    Tag(Tag),
    ProductTag(ProductTag),
}
```

### Step 2: Implement RecordStoreDefinitionExt Trait

This trait provides the bridge between `Record` operations and model storage:

```rust
pub trait RecordStoreDefinitionExt: NetabaseDefinition + Clone {
    /// Store the inner model directly (no wrapper)
    fn put_inner_model<S>(&self, store: &S) -> Result<(), Box<dyn std::error::Error>>
    where
        S: StoreTrait<Self>;

    /// Extract primary key as bytes from this Definition variant
    fn primary_key_bytes(&self) -> Vec<u8>;

    /// Fetch model by key bytes and wrap in Definition enum
    fn get_by_key_bytes<S>(
        store: &S,
        key: &[u8],
    ) -> Result<Option<Self>, Box<dyn std::error::Error>>
    where
        S: StoreTrait<Self>;
}
```

### Step 3: Implement the Extension Trait

Example implementation for your specific Definition enum:

```rust
impl RecordStoreDefinitionExt for Definitions {
    fn put_inner_model<S>(&self, store: &S) -> Result<(), Box<dyn std::error::Error>>
    where
        S: StoreTrait<Self>,
    {
        // Match on Definition variant and store inner model directly
        match self {
            Definitions::User(model) => Ok(store.put_one(model.clone())?),
            Definitions::Product(model) => Ok(store.put_one(model.clone())?),
            Definitions::Category(model) => Ok(store.put_one(model.clone())?),
            Definitions::Review(model) => Ok(store.put_one(model.clone())?),
            Definitions::Tag(model) => Ok(store.put_one(model.clone())?),
            Definitions::ProductTag(model) => Ok(store.put_one(model.clone())?),
        }
    }

    fn primary_key_bytes(&self) -> Vec<u8> {
        // Extract primary key and serialize to bytes
        match self {
            Definitions::User(model) => {
                let key = model.primary_key();
                bincode::encode_to_vec(&key, bincode::config::standard())
                    .unwrap_or_default()
            },
            Definitions::Product(model) => {
                let key = model.primary_key();
                bincode::encode_to_vec(&key, bincode::config::standard())
                    .unwrap_or_default()
            },
            Definitions::Category(model) => {
                let key = model.primary_key();
                bincode::encode_to_vec(&key, bincode::config::standard())
                    .unwrap_or_default()
            },
            Definitions::Review(model) => {
                let key = model.primary_key();
                bincode::encode_to_vec(&key, bincode::config::standard())
                    .unwrap_or_default()
            },
            Definitions::Tag(model) => {
                let key = model.primary_key();
                bincode::encode_to_vec(&key, bincode::config::standard())
                    .unwrap_or_default()
            },
            Definitions::ProductTag(model) => {
                let key = model.primary_key();
                bincode::encode_to_vec(&key, bincode::config::standard())
                    .unwrap_or_default()
            },
        }
    }

    fn get_by_key_bytes<S>(
        store: &S,
        key_bytes: &[u8],
    ) -> Result<Option<Self>, Box<dyn std::error::Error>>
    where
        S: StoreTrait<Self>,
    {
        // Approach 1: Try each model type (simple but not optimal)
        // Try User
        if let Ok((user_key, _)) = bincode::decode_from_slice::<UserId>(
            key_bytes,
            bincode::config::standard(),
        ) {
            if let Ok(Some(user)) = store.get_one::<User>(user_key) {
                return Ok(Some(Definitions::User(user)));
            }
        }

        // Try Product
        if let Ok((product_key, _)) = bincode::decode_from_slice::<ProductId>(
            key_bytes,
            bincode::config::standard(),
        ) {
            if let Ok(Some(product)) = store.get_one::<Product>(product_key) {
                return Ok(Some(Definitions::Product(product)));
            }
        }

        // ... repeat for all other model types ...

        Ok(None)

        // Approach 2 (Better): Maintain a key->model_type index
        // Store a mapping of key -> model discriminant
        // Then directly fetch from the correct table
    }
}
```

### Step 4: Create the Adapter

```rust
pub struct NetabaseRecordStoreAdapter<D: NetabaseDefinition> {
    store: RedbStore<D>,
    config: RecordStoreConfig,
}

pub struct RecordStoreConfig {
    pub max_value_bytes: usize,
    pub max_records: usize,
}

impl<D> NetabaseRecordStoreAdapter<D>
where
    D: NetabaseDefinition + Clone + serde::Serialize + serde::de::DeserializeOwned,
{
    pub fn new(store: RedbStore<D>, config: RecordStoreConfig) -> Self {
        Self { store, config }
    }

    fn deserialize_record_value(&self, value: &[u8]) -> Result<D, Box<dyn std::error::Error>> {
        let config = bincode::config::standard();
        let (definition, _): (D, usize) = bincode::decode_from_slice(value, config)?;
        Ok(definition)
    }

    fn serialize_definition(&self, definition: &D) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let config = bincode::config::standard();
        let bytes = bincode::encode_to_vec(definition, config)?;
        Ok(bytes)
    }
}
```

### Step 5: Implement RecordStore Trait

```rust
impl<D> RecordStore for NetabaseRecordStoreAdapter<D>
where
    D: NetabaseDefinition + Clone + serde::Serialize + serde::de::DeserializeOwned + RecordStoreDefinitionExt,
{
    type RecordsIter<'a> = RecordsIterator where Self: 'a;
    type ProvidedIter<'a> = ProvidedIterator where Self: 'a;

    /// GET FLOW:
    /// 1. Receive key bytes from network
    /// 2. Fetch model from store using key
    /// 3. Wrap model in Definition enum
    /// 4. Serialize Definition to bytes
    /// 5. Return as Record
    fn get(&self, k: &kad::Key) -> Option<Cow<'_, kad::Record>> {
        let key_bytes = k.to_vec();

        // Fetch model and wrap in Definition
        let definition = D::get_by_key_bytes(&self.store, &key_bytes).ok()??;

        // Serialize Definition
        let value = self.serialize_definition(&definition).ok()?;

        // Create Record
        let record = kad::Record {
            key: k.clone(),
            value,
            publisher: None,
            expires: None,
        };

        Some(Cow::Owned(record))
    }

    /// PUT FLOW:
    /// 1. Receive Record from network
    /// 2. Deserialize Record.value → Definition enum
    /// 3. Match on Definition to extract inner model
    /// 4. Store model directly in correct tree (no wrapper!)
    fn put(&mut self, r: kad::Record) -> kad::Result<()> {
        // Validate size
        if r.value.len() >= self.config.max_value_bytes {
            return Err(kad::Error::ValueTooLarge);
        }

        // Deserialize to Definition enum
        let definition = self
            .deserialize_record_value(&r.value)
            .map_err(|_| kad::Error::MaxRecords)?;

        // Store inner model directly (Definition enum handles the dispatch)
        definition
            .put_inner_model(&self.store)
            .map_err(|_| kad::Error::MaxRecords)?;

        Ok(())
    }

    fn remove(&mut self, k: &kad::Key) {
        let key_bytes = k.to_vec();
        // Fetch to determine model type, then delete
        if let Ok(Some(definition)) = D::get_by_key_bytes(&self.store, &key_bytes) {
            // In practice, you'd need a delete_inner_model method
            // similar to put_inner_model
        }
    }

    /// ITERATOR FLOW:
    /// a) Iterate over all models from all trees
    /// b) Wrap each model in Definition enum
    /// c) Serialize Definition to create Record.value
    /// d) Return as Cow::Owned(Record)
    fn records(&self) -> Self::RecordsIter<'_> {
        let mut records = Vec::new();

        // Use the DefinitionStoreExt::iter_all_models() we created
        if let Ok(iter) = self.store.iter_all_models() {
            for model_result in iter {
                if let Ok(definition_cow) = model_result {
                    let definition = definition_cow.into_owned();

                    // Get primary key for this model
                    let key_bytes = definition.primary_key_bytes();

                    // Serialize Definition
                    if let Ok(value) = self.serialize_definition(&definition) {
                        let record = kad::Record {
                            key: kad::Key::new(key_bytes),
                            value,
                            publisher: None,
                            expires: None,
                        };
                        records.push(record);
                    }
                }
            }
        }

        RecordsIterator::new(records)
    }

    // Provider methods would follow similar patterns
    fn add_provider(&mut self, _record: kad::ProviderRecord) -> kad::Result<()> {
        Ok(())
    }

    fn providers(&self, _key: &kad::Key) -> Vec<kad::ProviderRecord> {
        vec![]
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        ProvidedIterator::new(vec![])
    }

    fn remove_provider(&mut self, _key: &kad::Key, _provider: &[u8]) {}
}
```

## Complete Example

```rust
use netabase_store::RedbStore;
use libp2p::kad::{Kademlia, KademliaConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create store
    let store = RedbStore::<Definitions>::new("./kad.db")?;

    // Create adapter
    let adapter = NetabaseRecordStoreAdapter::new(
        store,
        RecordStoreConfig::default(),
    );

    // Use with libp2p
    let local_peer_id = PeerId::random();
    let mut kad = Kademlia::with_config(
        local_peer_id,
        adapter,
        KademliaConfig::default(),
    );

    // Network receives a PUT command
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
    };

    // Wrap in Definition and serialize
    let definition = Definitions::User(user);
    let value = bincode::encode_to_vec(&definition, bincode::config::standard())?;

    let record = Record::new(
        Key::new(bincode::encode_to_vec(&UserId(1), bincode::config::standard())?),
        value,
    );

    // Put stores the User model directly (not wrapped)
    kad.put_record(record, Quorum::One)?;

    // Get fetches User, wraps in Definition, serializes, returns as Record
    let key = Key::new(bincode::encode_to_vec(&UserId(1), bincode::config::standard())?);
    if let Some(record) = kad.get_record(&key) {
        // Deserialize to get the Definition
        let (definition, _) = bincode::decode_from_slice::<Definitions>(
            &record.value,
            bincode::config::standard(),
        )?;

        // Extract the User
        if let Definitions::User(user) = definition {
            println!("Retrieved user: {}", user.name);
        }
    }

    Ok(())
}
```

## Iterator Implementation Details

The `records()` iterator follows this flow:

```rust
fn records(&self) -> Self::RecordsIter<'_> {
    let mut records = Vec::new();

    // Step a) Iterate over all models from all trees
    if let Ok(all_models_iter) = self.store.iter_all_models() {
        for model_cow in all_models_iter {
            if let Ok(definition_cow) = model_cow {
                // Step b) Already wrapped in Definition enum
                let definition = definition_cow.into_owned();

                // Step c) Convert to Record
                let key_bytes = definition.primary_key_bytes();
                if let Ok(value_bytes) = self.serialize_definition(&definition) {
                    let record = kad::Record {
                        key: kad::Key::new(key_bytes),
                        value: value_bytes,
                        publisher: None,
                        expires: None,
                    };

                    // Step d) Will be returned as Cow::Owned
                    records.push(record);
                }
            }
        }
    }

    RecordsIterator::new(records)
}
```

## Key Optimizations

### 1. Key-to-Model-Type Index

For faster lookups in `get_by_key_bytes`, maintain an index:

```rust
// Add to your Definitions enum
#[derive(Debug, Clone, Encode, Decode)]
pub struct KeyIndex {
    key_hash: [u8; 32],
    model_discriminant: DefinitionsDiscriminants,
}

// When putting a model, also index the key
impl RecordStoreDefinitionExt for Definitions {
    fn put_inner_model<S>(&self, store: &S) -> Result<(), Box<dyn std::error::Error>> {
        // Store the model
        match self {
            Definitions::User(model) => store.put_one(model.clone())?,
            // ...
        }

        // Index the key
        let key_bytes = self.primary_key_bytes();
        let key_hash = blake3::hash(&key_bytes);
        let index = KeyIndex {
            key_hash: *key_hash.as_bytes(),
            model_discriminant: self.into(),
        };
        store.put_one(index)?;

        Ok(())
    }

    fn get_by_key_bytes<S>(store: &S, key_bytes: &[u8]) -> Result<Option<Self>> {
        // Look up model type from index
        let key_hash = blake3::hash(key_bytes);
        let index_key = KeyIndexId(*key_hash.as_bytes());

        if let Ok(Some(index)) = store.get_one::<KeyIndex>(index_key) {
            // Now we know which model type to fetch
            match index.model_discriminant {
                DefinitionsDiscriminants::User => {
                    let (key, _) = bincode::decode_from_slice::<UserId>(key_bytes, ...)?;
                    Ok(store.get_one::<User>(key)?.map(Definitions::User))
                },
                // ... other discriminants
            }
        } else {
            Ok(None)
        }
    }
}
```

### 2. Lazy Iterator

For large datasets, implement a lazy iterator that doesn't load all records into memory:

```rust
pub struct LazyRecordsIterator<'a, D: NetabaseDefinition> {
    adapter: &'a NetabaseRecordStoreAdapter<D>,
    model_type_index: usize,
    current_model_keys: Vec<Vec<u8>>,
    current_key_index: usize,
}

// Implement Iterator that fetches one model at a time
```

## Summary

This implementation achieves:

1. ✅ **Models stored directly** - No wrapper types, native storage format
2. ✅ **Definition enum as serialization boundary** - Clean separation of concerns
3. ✅ **Type-safe operations** - Rust's type system ensures correctness
4. ✅ **Network compatibility** - Works seamlessly with libp2p RecordStore
5. ✅ **Efficient storage** - No duplication, optimal database usage

The key insight is that `Definition` enum serves dual purposes:
- **At rest**: Enum variants specify which tree to store in
- **In transit**: Serialized enum is the Record.value on the network

This creates a clean, efficient, and maintainable implementation.
