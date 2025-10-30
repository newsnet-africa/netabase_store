# RecordStore Storage Guarantee

## Overview

The Netabase RecordStore implementation has been updated to **guarantee** that data stored in the database is always a `NetabaseModelTrait` type, never wrapped in the `NetabaseDefinitionTrait` enum.

## The Problem

Previously, the RecordStore implementation had several issues:

1. **Type Wrapping**: Models were wrapped in the Definition enum before storage
2. **Inefficient Routing**: Had to decode record values to determine which tree they belonged to
3. **Broken Abstraction**: RecordStore should work with opaque bytes, but we were decoding typed data
4. **Paxos Integration**: Made it difficult to use Paxos consensus on top of RecordStore

## The Solution

### ModelRecordKey Format

Records now use a key format that includes the model discriminant:

```
<discriminant_name>:<key_bytes>
```

Example:
```
User:0x0100000000000001  // User with id=1
Post:0x0100000000000042  // Post with id=66
```

This allows:
- **Efficient routing** to the correct tree without decoding the value
- **Type safety** by embedding model type information in the key
- **Direct storage** of model types without Definition wrapper

### ModelRecordStore Trait

A new extension trait provides typed operations:

```rust
use netabase_store::databases::record_store::model_store::ModelRecordStore;

// Put a model (stores the model directly, not wrapped)
let user = User { id: 1, name: "Alice" };
store.put_model::<MyDefinition, _>(&user)?;

// Get a model (returns the model directly)
let user: User = store.get_model::<MyDefinition, User, _>(&1)?;

// Remove a model
store.remove_model::<MyDefinition, User, _>(&1);
```

## Storage Guarantee

### What is Stored

When using `put_model()`:

```rust
// Model type
struct User {
    id: u64,
    name: String,
}

// What gets stored in the database:
// Key:   "User:0x0100000000000001"
// Value: <bincode encoded User struct>
//        NOT: <bincode encoded MyDefinition::User(User)>
```

The value is the **raw model bytes**, not wrapped in any enum.

### Why This Matters

1. **Type Safety**: You always know you're working with model types
2. **Performance**: No extra wrapping/unwrapping layer
3. **Paxos Integration**: Can implement consensus directly on models
4. **Consistency**: Aligns with how the rest of Netabase store works

## API Usage

### Recommended (Type-Safe)

```rust
use netabase_store::databases::record_store::model_store::ModelRecordStore;

// Store a model
let user = User { id: 1, name: "Alice".to_string() };
store.put_model::<BlogSchema, _>(&user)?;

// Retrieve by key
let retrieved: User = store
    .get_model::<BlogSchema, User, _>(&1)
    .ok_or("User not found")?;

assert_eq!(retrieved.name, "Alice");
```

### Low-Level (libp2p compatibility)

For libp2p interop, you can use the raw RecordStore trait, but **you must use ModelRecordKey format**:

```rust
use libp2p::kad::store::RecordStore;
use netabase_store::databases::record_store::model_store;

// Create a record with ModelRecordKey format (required!)
let record = model_store::utils::model_to_record::<BlogSchema, User>(&user)?;

// Store via RecordStore trait
store.put(record)?;

// ❌ WRONG - this will fail:
// let key = RecordKey::from(bincode::encode_to_vec(&user.id, ...)?);
// let record = Record { key, value: bincode::encode_to_vec(&definition, ...)?, ... };
// store.put(record)?; // Error: key not in ModelRecordKey format!
```

## Required Format

All records MUST use the ModelRecordKey format:

1. **Keys**: Must be in `<discriminant>:<key_bytes>` format
2. **Values**: Must be serialized model types (not Definition wrappers)
3. **No Fallback**: Records not in this format will return errors

This strict enforcement ensures:
- Type safety at the storage layer
- Efficient routing without value decoding
- Clear contract for all RecordStore operations

## Implementation Details

### Key Structure

```rust
pub struct ModelRecordKey {
    /// The model discriminant as a string (e.g., "User", "Post")
    pub discriminant: String,
    /// The serialized primary key bytes
    pub key_bytes: Vec<u8>,
}
```

### Encoding Flow

```rust
// 1. Create ModelRecordKey from model
let model_key = ModelRecordKey::from_model::<Definition, Model>(&model);

// 2. Convert to libp2p RecordKey
let record_key = model_key.to_record_key(); // "User:0x01..."

// 3. Encode model DIRECTLY (not wrapped)
let value_bytes = bincode::encode_to_vec(&model, config)?;

// 4. Create Record
let record = Record {
    key: record_key,
    value: value_bytes,  // Raw model bytes!
    publisher: None,
    expires: None,
};
```

### Decoding Flow

```rust
// 1. Parse ModelRecordKey from libp2p RecordKey
let model_key = ModelRecordKey::from_record_key(&record.key)?;

// 2. Get discriminant to know model type
let discriminant = model_key.tree_name(); // "User"

// 3. Decode DIRECTLY to model type (not Definition)
let model: User = bincode::decode_from_slice(&record.value, config)?;
```

## Benefits for Paxos Integration

The new design makes Paxos consensus integration much cleaner:

```rust
// Old way - had to unwrap Definition
let record = store.get(&key)?;
let (definition, _) = bincode::decode_from_slice(&record.value, config)?;
let user: User = definition.try_into()?; // Unwrap from Definition

// New way - direct model access
let user: User = store.get_model::<BlogSchema, User, _>(&key)?;

// Paxos can now work directly with models
let proposed_user = User { id: 1, name: "Bob".to_string() };
paxos.propose(proposed_user).await?;

// When committed, store directly
store.put_model::<BlogSchema, _>(&proposed_user)?;
```

## Summary

- ✅ **Guaranteed**: RecordStore stores NetabaseModelTrait types directly (never Definition wrappers)
- ✅ **Efficient**: Routing via key discriminant, zero value decoding needed
- ✅ **Type-Safe**: Strong typing through ModelRecordStore trait
- ✅ **Required Format**: All records MUST use ModelRecordKey format
- ✅ **Paxos-Ready**: Clean integration with consensus layer
- ✅ **No Ambiguity**: Single, clear storage format - no legacy compatibility

**Always use** `put_model()`, `get_model()`, and `remove_model()` to ensure correct ModelRecordKey format and model-type storage.
