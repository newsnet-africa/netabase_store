# RecordStore Implementation Changes

## Summary

The RecordStore implementation has been **completely rewritten** to eliminate ambiguity and enforce a strict storage guarantee: **all data stored is NetabaseModelTrait types, never NetabaseDefinitionTrait wrappers**.

## What Was Removed

### 1. Definition-Based Storage (Removed) ❌

**Old behavior (removed):**
```rust
// Value was wrapped in Definition
let definition: BlogSchema = BlogSchema::User(user);
let value_bytes = bincode::encode_to_vec(&definition, ...)?;
```

**New behavior (enforced):**
```rust
// Value is the model directly
let value_bytes = bincode::encode_to_vec(&user, ...)?;
```

### 2. Fallback Routing Logic (Removed) ❌

**Old behavior (removed):**
```rust
// Had to try all trees to find the key
for disc in D::Discriminant::iter() {
    if let Ok(tree) = self.db().open_tree(disc.to_string()) {
        if tree.contains_key(&key_bytes).unwrap_or(false) {
            return Ok(tree);
        }
    }
}
```

**New behavior (enforced):**
```rust
// Parse discriminant directly from key
let model_key = ModelRecordKey::from_record_key(key)?;
self.db().open_tree(model_key.tree_name())
```

### 3. Backward Compatibility Code (Removed) ❌

**Old behavior (removed):**
```rust
// Try ModelRecordKey first, fall back to Definition decoding
if let Ok(model_key) = ModelRecordKey::from_record_key(&record.key) {
    // Use model key
} else {
    // Fallback: decode value as Definition
    let (definition, _): (D, _) = bincode::decode_from_slice(...)?;
}
```

**New behavior (enforced):**
```rust
// ModelRecordKey format is REQUIRED
let model_key = ModelRecordKey::from_record_key(&record.key)?;
```

### 4. Ambiguous Documentation (Removed) ❌

Removed all references to:
- "Backward compatibility"
- "Legacy format support"
- "Fallback behavior"
- "Migration path"

## What Was Added

### 1. Strict Format Enforcement ✅

**ModelRecordKey format is now REQUIRED:**

```
Format: <discriminant>:<key_bytes>
Example: User:0x0100000000000001
```

Any key not in this format will result in `Error::MaxRecords`.

### 2. Clear Storage Guarantee ✅

**Documentation now explicitly states:**

> All records stored are guaranteed to be `NetabaseModelTrait` types, never wrapped in `NetabaseDefinitionTrait`.

### 3. Primary API Designation ✅

The `ModelRecordStore` trait is now documented as the **primary/recommended API**:

```rust
use netabase_store::databases::record_store::model_store::ModelRecordStore;

// Recommended - type-safe, enforces guarantee
store.put_model::<MyDefinition, _>(&user)?;

// Avoid - requires manual key construction
let record = model_store::utils::model_to_record(...)?;
store.put(record)?;
```

### 4. Comprehensive Documentation ✅

Added/updated:
- Module-level documentation with storage guarantee
- RecordStore impl documentation with format requirements
- ModelRecordStore trait documentation emphasizing guarantees
- STORAGE_GUARANTEE.md with implementation details

## Benefits of Removing Old Implementation

### 1. **No Ambiguity** 🎯

**Before:** Two possible storage formats
- ModelRecordKey format → stores model
- Legacy format → stores Definition

**After:** One format only
- ModelRecordKey format → stores model

### 2. **No Confusion** 🎯

**Before:** Unclear which format would be used
- "Will this store a model or Definition?"
- "How do I ensure I'm using the right format?"

**After:** Always stores models
- "Always stores models, no exceptions"
- "Use put_model() and it's guaranteed"

### 3. **Better Performance** 🎯

**Before:** Fallback logic slowed everything down
- Try ModelRecordKey parsing
- If failed, iterate all trees
- If not found, decode value as Definition

**After:** Direct routing
- Parse ModelRecordKey (one operation)
- Open tree directly
- Done

### 4. **Cleaner Code** 🎯

**Before:**
```rust
fn tree_for_key(&self, key: &Key) -> Result<sled::Tree> {
    if let Ok(model_key) = ModelRecordKey::from_record_key(key) {
        return self.db().open_tree(model_key.tree_name())
            .map_err(|_| Error::MaxRecords);
    }

    let key_bytes = utils::encode_key(key);
    for disc in D::Discriminant::iter() {
        if let Ok(tree) = self.db().open_tree(disc.to_string())
            && tree.contains_key(&key_bytes).unwrap_or(false)
        {
            return Ok(tree);
        }
    }

    let first_disc = D::Discriminant::iter().next()
        .ok_or(Error::MaxRecords)?;
    self.db().open_tree(first_disc.to_string())
        .map_err(|_| Error::MaxRecords)
}
```

**After:**
```rust
fn tree_for_key(&self, key: &Key) -> Result<sled::Tree> {
    let model_key = ModelRecordKey::from_record_key(key)?;
    self.db()
        .open_tree(model_key.tree_name())
        .map_err(|_| Error::MaxRecords)
}
```

### 5. **Paxos Integration Ready** 🎯

**Before:** Unclear how Paxos would interact
- "Do I propose the model or the Definition?"
- "What gets stored after consensus?"

**After:** Crystal clear
- Propose models directly
- Models get stored directly
- No conversion layer needed

```rust
// Paxos can work directly with models
let proposed_user = User { id: 1, name: "Bob" };
paxos.propose(proposed_user).await?;

// When committed, store directly
store.put_model::<BlogSchema, _>(&committed_user)?;

// ✅ Database contains: User { id: 1, name: "Bob" }
// ❌ NOT: BlogSchema::User(User { id: 1, name: "Bob" })
```

## Breaking Changes

### For New Code

✅ **No breaking changes** - new code should use `ModelRecordStore` trait which works correctly.

### For Existing Code Using Legacy Format

❌ **Breaking** - code that relied on Definition wrapping will fail:

**Will fail:**
```rust
// Old code that doesn't use ModelRecordKey format
let key = RecordKey::from(bincode::encode_to_vec(&id, ...)?);
let definition = BlogSchema::User(user);
let value = bincode::encode_to_vec(&definition, ...)?;
let record = Record { key, value, ... };
store.put(record)?; // ❌ Error: key not in ModelRecordKey format
```

**Must change to:**
```rust
// New code using ModelRecordStore
store.put_model::<BlogSchema, _>(&user)?; // ✅ Correct format
```

## Migration Guide

If you have existing code using the RecordStore, you need to:

1. **Replace raw RecordStore operations with ModelRecordStore:**

```rust
// OLD
let record = Record { ... };
store.put(record)?;

// NEW
store.put_model::<MyDefinition, _>(&model)?;
```

2. **Update key construction to use ModelRecordKey:**

```rust
// OLD
let key = RecordKey::from(key_bytes);

// NEW
let model_key = ModelRecordKey::from_model::<MyDefinition, _>(&model);
let key = model_key.to_record_key();
```

3. **Update decoding to expect models, not Definitions:**

```rust
// OLD
let (definition, _): (MyDefinition, _) = bincode::decode_from_slice(...)?;
let model: MyModel = definition.try_into()?;

// NEW
let (model, _): (MyModel, _) = bincode::decode_from_slice(...)?;
```

## Files Modified

### Core Implementation
- `netabase_store/src/databases/record_store/sled_impl.rs`
  - Removed Definition fallback in `tree_for_record()`
  - Removed tree iteration in `tree_for_key()`
  - Added strict ModelRecordKey enforcement
  - Added comprehensive documentation

### Model Store
- `netabase_store/src/databases/record_store/model_store.rs`
  - Updated `model_to_record()` to encode models directly
  - Updated `record_to_model()` to decode models directly
  - Removed Definition conversion logic
  - Emphasized as primary API

### Documentation
- `netabase_store/src/databases/record_store/mod.rs`
  - Added storage guarantee to module docs
  - Emphasized ModelRecordStore as recommended API

- `netabase_store/src/databases/record_store/STORAGE_GUARANTEE.md`
  - Removed backward compatibility section
  - Added strict format requirements
  - Clarified storage guarantees

## Testing

To verify the changes work correctly:

```rust
#[test]
fn test_model_storage_guarantee() {
    let store = SledStore::<BlogSchema>::temp()?;

    // Store a user
    let user = User { id: 1, name: "Alice".to_string() };
    store.put_model::<BlogSchema, _>(&user)?;

    // Verify key format
    let model_key = ModelRecordKey::from_model::<BlogSchema, _>(&user);
    let key = model_key.to_record_key();

    // Key should be "User:<id_bytes>"
    assert!(key.to_vec().starts_with(b"User:"));

    // Get the raw record
    let record = store.get(&key).unwrap();

    // Value should decode directly to User (not BlogSchema)
    let (decoded_user, _): (User, _) =
        bincode::decode_from_slice(&record.value, bincode::config::standard())?;

    assert_eq!(decoded_user.id, 1);
    assert_eq!(decoded_user.name, "Alice");

    // Should NOT decode as BlogSchema
    let result: Result<(BlogSchema, _), _> =
        bincode::decode_from_slice(&record.value, bincode::config::standard());
    assert!(result.is_err()); // ✅ Proves it's not a Definition
}
```

## Conclusion

The old implementation has been completely removed. The RecordStore now has a single, clear contract:

✅ **Models are stored directly** (never Definition wrappers)
✅ **ModelRecordKey format is required** (no exceptions)
✅ **No backward compatibility** (clean, unambiguous implementation)
✅ **Paxos-ready** (can work directly with models)

This eliminates confusion, improves performance, and provides a solid foundation for building consensus on top of Kademlia DHT.
