# Blob Storage Refactoring Plan

## Current Implementation Issues

### Problem
The current blob storage implementation is inefficient in its use of redb's `MultimapTable`:

**Current approach:**
```rust
// Key includes owner but each chunk is a separate key-value pair
UserBlobKeys::Bio { owner: UserID("user1") } → UserBlobItem(chunk0_data)
UserBlobKeys::Bio { owner: UserID("user1") } → UserBlobItem(chunk1_data)  
UserBlobKeys::Bio { owner: UserID("user1") } → UserBlobItem(chunk2_data)
```

Problems:
1. Each chunk creates a new multimap entry with duplicate keys
2. The `UserBlobItem` stores the full serialized blob data without chunk index
3. No explicit chunk ordering - relies on insertion order
4. Less efficient than redb's intended multimap usage

### Correct Implementation

**Desired approach:**
```rust
// Single key with multiple indexed values
UserID("user1") → (chunk0_bytes, index=0)
UserID("user1") → (chunk1_bytes, index=1)
UserID("user1") → (chunk2_bytes, index=2)
```

Benefits:
1. More efficient multimap usage - single key, multiple values
2. Explicit chunk indexing in the value
3. Simpler key structure (just the primary key)
4. Easier to query all chunks for a model
5. Natural ordering by index

## Required Changes

### 1. Blob Key Structure

**Current:**
```rust
pub enum UserBlobKeys {
    Bio { owner: UserID },      // Separate variant per field
    Another { owner: UserID },
}
```

**Proposed:**
```rust
// Option A: Field discriminant in key
pub enum UserBlobKeys {
    Bio(UserID),           // Just the owner ID
    Another(UserID),
}

// Option B: Separate table per field (cleaner)
// Bio blob table: UserID → (Vec<u8>, u32)
// Another blob table: UserID → (Vec<u8>, u32)
```

**Recommendation: Option B** - Each blob field gets its own multimap table.
- Simpler keys (just the primary key type)
- Better type safety
- Easier to manage separately

### 2. Blob Value Structure

**Current:**
```rust
UserBlobItem(Vec<u8>)  // Just the serialized data
```

**Proposed:**
```rust
(Vec<u8>, u32)  // (chunk_bytes, chunk_index)
// or
struct BlobChunk {
    data: Vec<u8>,
    index: u32,
}
```

**Recommendation:** Use tuple `(Vec<u8>, u32)` for simplicity and efficiency.

### 3. Table Naming

**Current:**
```rust
"User__blob__bio"      // Single blob table per field
"User__blob__another"
```

**Proposed (same):**
```rust
"User__blob__bio"      // MultimapTable<UserID, (Vec<u8>, u32)>
"User__blob__another"  // MultimapTable<UserID, (Vec<u8>, u32)>
```

Keep the same naming - each field gets its own multimap table.

### 4. CRUD Operations

**Insert:**
```rust
// Current
for (key, item) in blob_entries {
    table.insert(key, item)?;  // key = UserBlobKeys::Bio{owner}, item = UserBlobItem
}

// Proposed
for (chunk_data, chunk_index) in blob_chunks {
    table.insert(primary_key.clone(), (chunk_data, chunk_index))?;
}
```

**Read:**
```rust
// Current
let items: Vec<UserBlobItem> = table.get(key)?;

// Proposed  
let mut chunks: Vec<(Vec<u8>, u32)> = table.get(&primary_key)?.collect();
chunks.sort_by_key(|(_, idx)| *idx);  // Sort by index
let data: Vec<u8> = chunks.into_iter()
    .flat_map(|(bytes, _)| bytes)
    .collect();
```

**Update:**
```rust
// Current
// Remove all old entries, insert all new entries

// Proposed (same, but more efficient)
table.remove_all(&primary_key)?;  // Clear all chunks for this key
for (chunk_data, chunk_index) in new_blob_chunks {
    table.insert(primary_key.clone(), (chunk_data, chunk_index))?;
}
```

**Delete:**
```rust
// Current
for (key, item) in blob_entries {
    table.remove(key, item)?;  // Requires exact key-value match
}

// Proposed
table.remove_all(&primary_key)?;  // Remove all values for key
```

## Implementation Steps

### Phase 1: Update Traits
- [ ] Modify `NetabaseBlobItem` trait to return `Vec<(Vec<u8>, u32)>` from chunking
- [ ] Update `split_into_blobs()` to include chunk indices
- [ ] Update `reconstruct_from_blobs()` to accept `Vec<(Vec<u8>, u32)>`

### Phase 2: Update Macros
- [ ] Change blob key enum to use simpler structure (field discriminant + owner)
- [ ] Update `get_blob_entries()` to return proper format
- [ ] Ensure each field has separate multimap table

### Phase 3: Update CRUD Operations  
- [ ] Modify insertion to use `(Vec<u8>, u32)` values
- [ ] Update read operations to reconstruct from indexed chunks
- [ ] Fix update operations to use `remove_all()`
- [ ] Simplify delete operations

### Phase 4: Update Tests
- [ ] Test blob chunking with new format
- [ ] Test reconstruction with indices
- [ ] Test update operations
- [ ] Test deletion

### Phase 5: Update Documentation
- [ ] Update blob.rs documentation
- [ ] Update ARCHITECTURE.md blob section
- [ ] Update GUIDE.md blob examples
- [ ] Add migration notes if needed

## Benefits

1. **Performance**: More efficient multimap usage
2. **Correctness**: Explicit chunk ordering via indices
3. **Simplicity**: Cleaner key structure, simpler queries
4. **Maintainability**: Each field has dedicated table
5. **Flexibility**: Easier to implement partial updates later

## Backward Compatibility

This is a **breaking change** for existing databases. Options:

1. **Migration Path**: Provide automatic migration from old to new format
2. **Version Bump**: Increment schema version, run migration on first access
3. **Fresh Start**: Document as pre-1.0, acceptable to break compatibility

**Recommendation:** Since crate is at 0.1.0, this is acceptable as a breaking change before 1.0 release.

## Testing Strategy

1. Create test with large blob (>200KB)
2. Verify chunking creates proper indices
3. Verify reconstruction maintains order
4. Verify updates replace all chunks
5. Verify deletes remove all chunks
6. Benchmark performance vs current implementation

## Timeline

- Analysis: ✅ Complete
- Design: ✅ Complete  
- Implementation: ~4-6 hours
- Testing: ~2 hours
- Documentation: ~1 hour

**Total estimate:** 7-9 hours of focused work

## Notes

This refactoring aligns with redb's multimap design and will provide:
- Better performance (fewer key comparisons)
- Clearer semantics (one model = one key)
- Easier debugging (all chunks grouped together)
- Foundation for future enhancements (partial updates, streaming, etc.)
