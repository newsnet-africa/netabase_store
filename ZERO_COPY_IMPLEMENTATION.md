# Zero-Copy Redb Backend Implementation Plan

**Status**: In Progress  
**Created**: 2025-11-11  
**Goal**: Implement full zero-copy, transaction-scoped redb backend with direct Key/Value trait implementations

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Implementation Phases](#implementation-phases)
4. [API Design](#api-design)
5. [Testing Strategy](#testing-strategy)
6. [Migration Guide](#migration-guide)
7. [Performance Goals](#performance-goals)

## Overview

### Current State

The existing `redb_store.rs` implementation uses:
- BincodeWrapper for Key/Value trait implementations
- One transaction per operation (high overhead)
- Always clones data on reads (no zero-copy)
- Implicit transaction management

### Target State

The new `redb_zerocopy.rs` will provide:
- Direct Key/Value trait implementations on models
- Transaction-scoped API with proper lifetime management
- Zero-copy reads via `AccessGuard<M>`
- Explicit transaction control
- Bulk operations (`put_many`, `remove_many`)
- Both convenience and power-user APIs

### Key Benefits

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| Read performance | Bincode deserialize | Zero-copy reference | ~50-70% faster |
| Transaction overhead | Per-operation | Per-batch | ~10x fewer transactions |
| Memory usage | Clone on every read | Borrow from page | ~60% less allocation |
| API complexity | Simple but limited | Flexible with helpers | More powerful |

## Architecture

### Lifetime Chain

```
RedbStoreZeroCopy<D>                    ('static or app lifetime)
  ↓ begin_write() / begin_read()
RedbWriteTransactionZC<'db, D>          (borrows 'db from store)
RedbReadTransactionZC<'db, D>           (borrows 'db from store)
  ↓ open_tree<M>()
RedbTreeMut<'txn, 'db, D, M>            (borrows 'txn from transaction)
RedbTree<'txn, 'db, D, M>               (borrows 'txn from transaction)
  ↓ get_borrowed()
AccessGuard<'tree, M>                   (borrows from tree/transaction)
  ↓ value()
&M or M::BorrowedType<'tree>            (borrowed reference into database page)
```

### Type System

```rust
// Core database handle
pub struct RedbStoreZeroCopy<D> {
    db: Arc<Database>,
    tables: D::Tables,
}

// Write transaction
pub struct RedbWriteTransactionZC<'db, D> {
    inner: WriteTransaction,
    _phantom: PhantomData<&'db D>,
}

// Read transaction
pub struct RedbReadTransactionZC<'db, D> {
    inner: ReadTransaction,
    _phantom: PhantomData<&'db D>,
}

// Mutable tree (write operations)
pub struct RedbTreeMut<'txn, 'db, D, M> {
    txn: &'txn mut WriteTransaction,
    discriminant: D::Discriminant,
    table_name: &'static str,
    secondary_table_name: &'static str,
}

// Immutable tree (read operations)
pub struct RedbTree<'txn, 'db, D, M> {
    txn: &'txn ReadTransaction,
    discriminant: D::Discriminant,
    table_name: &'static str,
    secondary_table_name: &'static str,
}
```

## Implementation Phases

### Phase 1: Plan Document ✓

Create this comprehensive plan document.

**Status**: Complete

### Phase 2: Derive Macro Enhancements

#### 2.1 Create Type Analysis Utilities

**File**: `netabase_macros/src/generators/type_utils.rs`

**Purpose**: Centralized type analysis for width calculation and borrowing

**Functions to implement**:

```rust
/// Calculate total fixed width if all fields are fixed-width
pub fn calculate_fixed_width(fields: &syn::FieldsNamed) -> Option<usize> {
    let mut total = 0;
    for field in &fields.named {
        total += get_type_width(&field.ty)?;
    }
    Some(total)
}

/// Get size in bytes for a type, None if variable-width
pub fn get_type_width(ty: &Type) -> Option<usize> {
    match ty {
        Type::Path(tp) => {
            let segment = tp.path.segments.last()?;
            match segment.ident.to_string().as_str() {
                "u8" | "i8" | "bool" => Some(1),
                "u16" | "i16" => Some(2),
                "u32" | "i32" | "f32" => Some(4),
                "u64" | "i64" | "f64" => Some(8),
                "u128" | "i128" => Some(16),
                "String" | "Vec" => None, // Variable
                "Option" => {
                    // Could calculate for Option<fixed> but complex
                    None
                }
                _ => None,
            }
        }
        Type::Array(arr) => {
            let elem_width = get_type_width(&arr.elem)?;
            let len = get_array_len(&arr.len)?;
            Some(elem_width * len)
        }
        _ => None,
    }
}

/// Map owned type to borrowed type
pub fn map_to_borrowed_type(ty: &Type) -> TokenStream {
    // String -> &'a str
    // Vec<u8> -> &'a [u8]
    // primitives -> unchanged
    // Option<String> -> Option<&'a str>
}

/// Check if type can be borrowed (contains String, Vec<u8>, etc.)
pub fn is_borrowable_type(ty: &Type) -> bool {
    is_string_type(ty) || is_vec_u8_type(ty) || is_option_borrowable(ty)
}
```

**Integration points**:
- Used by zerocopy.rs for borrowed type generation
- Used by model_key.rs for fixed_width() optimization
- Shared utilities reduce duplication

#### 2.2 Enhance Zero-Copy Generator

**File**: `netabase_macros/src/generators/zerocopy.rs`

**Changes**:

1. **Import type_utils** at top of file
2. **Modify `generate_value_impl()`** (currently lines 388-464):
   ```rust
   pub fn generate_value_impl(model: &ItemStruct) -> TokenStream {
       let model_name = &model.ident;
       let borrowed_name = format!("{}Ref", model_name);
       
       // Use type_utils instead of delegating to tuple
       let fixed_width = if let Fields::Named(fields) = &model.fields {
           type_utils::calculate_fixed_width(fields)
       } else {
           None
       };
       
       let fixed_width_expr = match fixed_width {
           Some(width) => quote! { Some(#width) },
           None => quote! { None },
       };
       
       // ... rest of implementation
       quote! {
           impl ::redb::Value for #model_name {
               type SelfType<'a> = #borrowed_name<'a> where Self: 'a;
               
               fn fixed_width() -> Option<usize> {
                   #fixed_width_expr
               }
               
               // ... from_bytes, as_bytes, type_name
           }
       }
   }
   ```

3. **Fix `generate_from_borrowed()`** (currently lines 355-383):
   ```rust
   // Add the _borrowed_ref field initialization
   let field_conversions: Vec<TokenStream> = fields
       .named
       .iter()
       .map(|field| {
           let field_name = &field.ident;
           // Skip _borrowed_ref field
           if field_name.as_ref().unwrap() == "_borrowed_ref" {
               return quote! {};
           }
           let conversion = generate_to_owned_conversion(field_name.as_ref().unwrap(), &field.ty);
           quote! { #field_name: #conversion }
       })
       .collect();
   
   quote! {
       impl<'a> From<#borrowed_name<'a>> for #model_name {
           fn from(r: #borrowed_name<'a>) -> Self {
               #model_name {
                   #(#field_conversions,)*
                   #[cfg(feature = "redb-zerocopy")]
                   _borrowed_ref: Default::default(),
               }
           }
       }
   }
   ```

4. **Add Key trait for borrowed types**:
   ```rust
   pub fn generate_borrowed_key_impl(model: &ItemStruct) -> TokenStream {
       let borrowed_name = format!("{}Ref", model_name);
       
       quote! {
           impl<'a> ::redb::Key for #borrowed_name<'a> {
               fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                   let t1 = Self::from_bytes(data1);
                   let t2 = Self::from_bytes(data2);
                   t1.cmp(&t2)
               }
           }
       }
   }
   ```

#### 2.3 Update Model Key Generator

**File**: `netabase_macros/src/generators/model_key.rs`

**Changes**:

1. **Add import**: `use crate::generators::type_utils;`

2. **Enhance bincode `fixed_width()`** (lines 226-255):
   ```rust
   // In the redb::Value impl for the model
   fn fixed_width() -> Option<usize> {
       // Even with bincode, knowing fixed width helps redb optimize
       #fixed_width_calculation
   }
   ```

3. **Add `from_borrowed` method generation**:
   ```rust
   // In generate_model_trait_impl(), add after other impls:
   let from_borrowed_impl = quote! {
       #[cfg(all(feature = "redb", feature = "redb-zerocopy"))]
       impl #model_name {
           pub fn from_borrowed(borrowed: &#borrowed_name<'_>) -> Self {
               Self::from(borrowed.clone())
           }
       }
   };
   ```

#### 2.4 Module Updates

**File**: `netabase_macros/src/generators/mod.rs`

Add line:
```rust
pub mod type_utils;
```

### Phase 3: Runtime Implementation

#### 3.1 Core Transaction Types

**File**: `src/databases/redb_zerocopy.rs` (complete rewrite)

**Structure**:

```rust
// Module documentation (200+ lines of examples and explanation)

use crate::error::NetabaseError;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::{NetabaseModelTrait, NetabaseModelTraitKey};
// ... other imports

/// Main store handle
pub struct RedbStoreZeroCopy<D> { /* ... */ }

impl<D> RedbStoreZeroCopy<D> {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> { /* ... */ }
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> { /* ... */ }
    pub fn begin_write(&self) -> Result<RedbWriteTransactionZC<'_, D>, NetabaseError> { /* ... */ }
    pub fn begin_read(&self) -> Result<RedbReadTransactionZC<'_, D>, NetabaseError> { /* ... */ }
}

/// Write transaction
pub struct RedbWriteTransactionZC<'db, D> { /* ... */ }

impl<'db, D> RedbWriteTransactionZC<'db, D> {
    pub fn open_tree<M>(&mut self) -> Result<RedbTreeMut<'_, 'db, D, M>, NetabaseError> { /* ... */ }
    pub fn commit(self) -> Result<(), NetabaseError> { /* ... */ }
    pub fn abort(self) -> Result<(), NetabaseError> { /* ... */ }
}

/// Read transaction
pub struct RedbReadTransactionZC<'db, D> { /* ... */ }

impl<'db, D> RedbReadTransactionZC<'db, D> {
    pub fn open_tree<M>(&self) -> Result<RedbTree<'_, 'db, D, M>, NetabaseError> { /* ... */ }
}

/// Mutable tree
pub struct RedbTreeMut<'txn, 'db, D, M> { /* ... */ }

impl<'txn, 'db, D, M> RedbTreeMut<'txn, 'db, D, M> {
    pub fn put(&mut self, model: M) -> Result<(), NetabaseError> { /* ... */ }
    pub fn put_many(&mut self, models: Vec<M>) -> Result<(), NetabaseError> { /* ... */ }
    pub fn get(&self, key: &PrimaryKey) -> Result<Option<M>, NetabaseError> { /* ... */ }
    pub fn remove(&mut self, key: PrimaryKey) -> Result<Option<M>, NetabaseError> { /* ... */ }
    pub fn remove_many(&mut self, keys: Vec<PrimaryKey>) -> Result<Vec<Option<M>>, NetabaseError> { /* ... */ }
    pub fn len(&self) -> Result<usize, NetabaseError> { /* ... */ }
    pub fn is_empty(&self) -> Result<bool, NetabaseError> { /* ... */ }
    pub fn clear(&mut self) -> Result<(), NetabaseError> { /* ... */ }
}

/// Immutable tree
pub struct RedbTree<'txn, 'db, D, M> { /* ... */ }

impl<'txn, 'db, D, M> RedbTree<'txn, 'db, D, M> {
    pub fn get(&self, key: &PrimaryKey) -> Result<Option<M>, NetabaseError> { /* ... */ }
    pub fn get_borrowed(&self, key: &PrimaryKey) -> Result<Option<AccessGuard<'_, M>>, NetabaseError> { /* ... */ }
    pub fn iter(&self) -> Result<RedbIterator<M>, NetabaseError> { /* ... */ }
    pub fn iter_borrowed(&self) -> Result<RedbBorrowedIterator<'_, M>, NetabaseError> { /* ... */ }
    pub fn get_by_secondary_key(&self, key: &SecondaryKey) -> Result<Vec<M>, NetabaseError> { /* ... */ }
    pub fn get_by_secondary_key_borrowed(&self, key: &SecondaryKey) -> Result<Vec<AccessGuard<'_, M>>, NetabaseError> { /* ... */ }
    pub fn len(&self) -> Result<usize, NetabaseError> { /* ... */ }
    pub fn is_empty(&self) -> Result<bool, NetabaseError> { /* ... */ }
}
```

**Implementation notes**:
- Use table definitions from `D::Tables`
- Leak table name strings once per tree creation
- Secondary indexes use `MultimapTable<SecondaryKey, PrimaryKey>`
- All operations within transaction boundaries

#### 3.2 Convenience Wrappers

**Add to redb_zerocopy.rs**:

```rust
/// Tree that auto-commits on drop
pub struct AutoCommitTree<'db, D, M> {
    transaction: Option<RedbWriteTransactionZC<'db, D>>,
    tree: Option<RedbTreeMut<'db, 'db, D, M>>,
}

impl<'db, D, M> AutoCommitTree<'db, D, M> {
    pub fn put(&mut self, model: M) -> Result<(), NetabaseError> {
        self.tree.as_mut().unwrap().put(model)
    }
    
    pub fn commit(mut self) -> Result<(), NetabaseError> {
        drop(self.tree.take());
        self.transaction.take().unwrap().commit()
    }
}

impl<'db, D, M> Drop for AutoCommitTree<'db, D, M> {
    fn drop(&mut self) {
        if let (Some(tree), Some(txn)) = (self.tree.take(), self.transaction.take()) {
            drop(tree);
            let _ = txn.commit(); // Best effort
        }
    }
}

// Quick methods
impl<D> RedbStoreZeroCopy<D> {
    /// Insert a single model with auto-commit
    pub fn quick_put<M>(&self, model: M) -> Result<(), NetabaseError>
    where
        M: NetabaseModelTrait<D>,
    {
        let mut txn = self.begin_write()?;
        let mut tree = txn.open_tree::<M>()?;
        tree.put(model)?;
        drop(tree);
        txn.commit()
    }
    
    /// Get a single model (cloned)
    pub fn quick_get<M>(&self, key: &PrimaryKey) -> Result<Option<M>, NetabaseError>
    where
        M: NetabaseModelTrait<D>,
    {
        let txn = self.begin_read()?;
        let tree = txn.open_tree::<M>()?;
        tree.get(key)
    }
    
    /// Open tree with auto-commit behavior
    pub fn open_tree_auto<M>(&self) -> Result<AutoCommitTree<'_, D, M>, NetabaseError>
    where
        M: NetabaseModelTrait<D>,
    {
        let mut txn = self.begin_write()?;
        let tree = txn.open_tree::<M>()?;
        Ok(AutoCommitTree {
            transaction: Some(txn),
            tree: Some(tree),
        })
    }
}

// Transaction scope helper
pub fn with_write_transaction<D, F, R>(
    store: &RedbStoreZeroCopy<D>,
    f: F,
) -> Result<R, NetabaseError>
where
    F: FnOnce(&mut RedbWriteTransactionZC<D>) -> Result<R, NetabaseError>,
{
    let mut txn = store.begin_write()?;
    let result = f(&mut txn)?;
    txn.commit()?;
    Ok(result)
}
```

#### 3.3 Secondary Index Support

**Implementation in RedbTreeMut**:

```rust
impl<'txn, 'db, D, M> RedbTreeMut<'txn, 'db, D, M> {
    fn secondary_table_def(&self) -> MultimapTableDefinition<'static, SecondaryKey, PrimaryKey> {
        MultimapTableDefinition::new(self.secondary_table_name)
    }
    
    pub fn put(&mut self, model: M) -> Result<(), NetabaseError> {
        let primary_key = model.primary_key();
        let secondary_keys = model.secondary_keys();
        
        // Insert into primary table
        let mut table = self.txn.open_table(self.table_def())?;
        table.insert(M::Keys::from(primary_key.clone()), model)?;
        
        // Insert into secondary indexes
        if !secondary_keys.is_empty() {
            let mut sec_table = self.txn.open_multimap_table(self.secondary_table_def())?;
            for sec_key in secondary_keys {
                sec_table.insert(sec_key, primary_key.clone())?;
            }
        }
        
        Ok(())
    }
}
```

**Implementation in RedbTree**:

```rust
impl<'txn, 'db, D, M> RedbTree<'txn, 'db, D, M> {
    pub fn get_by_secondary_key(&self, sec_key: &SecondaryKey) -> Result<Vec<M>, NetabaseError> {
        let sec_table = self.txn.open_multimap_table(self.secondary_table_def())?;
        let mut results = Vec::new();
        
        // Get all primary keys for this secondary key
        for item in sec_table.get(sec_key)? {
            let primary_key = item?.value();
            if let Some(model) = self.get(&primary_key)? {
                results.push(model);
            }
        }
        
        Ok(results)
    }
    
    pub fn get_by_secondary_key_borrowed(&self, sec_key: &SecondaryKey) -> Result<Vec<AccessGuard<'_, M>>, NetabaseError> {
        // Similar but returns guards
    }
}
```

### Phase 4: Trait Updates

#### 4.1 Update NetabaseModelTrait

**File**: `src/traits/model.rs`

**Add method** (redb feature only):

```rust
#[cfg(feature = "redb")]
pub trait NetabaseModelTrait<D: NetabaseDefinitionTrait>: /* existing bounds */ {
    // ... existing methods ...
    
    /// Convert from borrowed type to owned type
    /// This enables cloning data from zero-copy reads when needed
    fn from_borrowed(borrowed: &Self::BorrowedType<'_>) -> Self;
}
```

### Phase 5: Testing

#### 5.1 Unit Tests

**File**: `src/databases/redb_zerocopy.rs` (inline `#[cfg(test)] mod tests`)

**Test structure**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Define test models
    #[netabase_definition_module(TestDef, TestKeys)]
    mod test_models {
        #[derive(NetabaseModel, Clone, Debug, PartialEq, Eq,
                 bincode::Encode, bincode::Decode,
                 serde::Serialize, serde::Deserialize)]
        #[netabase(TestDef)]
        pub struct User {
            #[primary_key]
            pub id: u64,
            pub name: String,
            #[secondary_key]
            pub email: String,
        }
    }
    
    #[test]
    fn test_basic_crud() {
        let store = RedbStoreZeroCopy::<TestDef>::new("test_basic.redb").unwrap();
        
        // Write
        let mut txn = store.begin_write().unwrap();
        let mut tree = txn.open_tree::<User>().unwrap();
        tree.put(User { id: 1, name: "Alice".into(), email: "alice@example.com".into() }).unwrap();
        drop(tree);
        txn.commit().unwrap();
        
        // Read (owned)
        let txn = store.begin_read().unwrap();
        let tree = txn.open_tree::<User>().unwrap();
        let user = tree.get(&1).unwrap().unwrap();
        assert_eq!(user.name, "Alice");
        
        // Read (borrowed) - zero-copy
        let guard = tree.get_borrowed(&1).unwrap().unwrap();
        let user_ref = guard.value();
        assert_eq!(user_ref.name, "Alice");
    }
    
    #[test]
    fn test_put_many() {
        let store = RedbStoreZeroCopy::<TestDef>::new("test_bulk.redb").unwrap();
        
        let users = (0..1000)
            .map(|i| User {
                id: i,
                name: format!("User{}", i),
                email: format!("user{}@example.com", i),
            })
            .collect();
        
        let mut txn = store.begin_write().unwrap();
        let mut tree = txn.open_tree::<User>().unwrap();
        tree.put_many(users).unwrap();
        drop(tree);
        txn.commit().unwrap();
        
        // Verify count
        let txn = store.begin_read().unwrap();
        let tree = txn.open_tree::<User>().unwrap();
        assert_eq!(tree.len().unwrap(), 1000);
    }
    
    #[test]
    fn test_transaction_isolation() {
        let store = RedbStoreZeroCopy::<TestDef>::new("test_isolation.redb").unwrap();
        
        // Start read transaction
        let read_txn = store.begin_read().unwrap();
        let read_tree = read_txn.open_tree::<User>().unwrap();
        assert_eq!(read_tree.len().unwrap(), 0);
        
        // Write in separate transaction
        let mut write_txn = store.begin_write().unwrap();
        let mut write_tree = write_txn.open_tree::<User>().unwrap();
        write_tree.put(User { id: 1, name: "Alice".into(), email: "alice@example.com".into() }).unwrap();
        drop(write_tree);
        write_txn.commit().unwrap();
        
        // Read transaction still sees old state (MVCC)
        assert_eq!(read_tree.len().unwrap(), 0);
        
        // New read transaction sees new state
        drop(read_tree);
        drop(read_txn);
        let new_txn = store.begin_read().unwrap();
        let new_tree = new_txn.open_tree::<User>().unwrap();
        assert_eq!(new_tree.len().unwrap(), 1);
    }
    
    #[test]
    fn test_secondary_index() {
        let store = RedbStoreZeroCopy::<TestDef>::new("test_secondary.redb").unwrap();
        
        let mut txn = store.begin_write().unwrap();
        let mut tree = txn.open_tree::<User>().unwrap();
        tree.put(User { id: 1, name: "Alice".into(), email: "alice@example.com".into() }).unwrap();
        tree.put(User { id: 2, name: "Bob".into(), email: "bob@example.com".into() }).unwrap();
        drop(tree);
        txn.commit().unwrap();
        
        // Query by secondary key
        let txn = store.begin_read().unwrap();
        let tree = txn.open_tree::<User>().unwrap();
        let users = tree.get_by_secondary_key(&"alice@example.com".to_string()).unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].name, "Alice");
    }
    
    // More tests...
}
```

#### 5.2 Integration Tests

**File**: `tests/redb_zerocopy_integration.rs`

Test complex scenarios:
- Multi-tree transactions
- Large datasets (10k+ records)
- Concurrent read transactions
- Error recovery

#### 5.3 Benchmarks

**File**: `benches/redb_zerocopy_bench.rs`

Compare performance:
- Old vs new API
- get() vs get_borrowed()
- Single vs bulk operations

### Phase 6: Examples

#### 6.1 Basic Example

**File**: `examples/redb_zerocopy_basic.rs`

```rust
use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
use netabase_store::{netabase_definition_module, NetabaseModel, netabase};

#[netabase_definition_module(MyDef, MyKeys)]
mod models {
    use super::*;
    
    #[derive(NetabaseModel, Clone, Debug, PartialEq,
             bincode::Encode, bincode::Decode,
             serde::Serialize, serde::Deserialize)]
    #[netabase(MyDef)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
    }
}

use models::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open database
    let store = RedbStoreZeroCopy::<MyDef>::new("example.redb")?;
    
    // Example 1: Explicit transaction
    println!("=== Explicit Transaction ===");
    {
        let mut txn = store.begin_write()?;
        let mut tree = txn.open_tree::<User>()?;
        
        tree.put(User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        })?;
        
        tree.put(User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        })?;
        
        drop(tree);
        txn.commit()?;
        println!("Inserted 2 users");
    }
    
    // Example 2: Zero-copy read
    println!("\n=== Zero-Copy Read ===");
    {
        let txn = store.begin_read()?;
        let tree = txn.open_tree::<User>()?;
        
        let guard = tree.get_borrowed(&1)?.unwrap();
        let user = guard.value();
        println!("User (zero-copy): {} - {}", user.id, user.name);
        // No allocation! user is a reference into the database page
    }
    
    // Example 3: Bulk insert
    println!("\n=== Bulk Insert ===");
    {
        let users: Vec<User> = (3..=10)
            .map(|i| User {
                id: i,
                name: format!("User{}", i),
                email: format!("user{}@example.com", i),
            })
            .collect();
        
        let mut txn = store.begin_write()?;
        let mut tree = txn.open_tree::<User>()?;
        tree.put_many(users)?;
        drop(tree);
        txn.commit()?;
        println!("Inserted 8 more users");
    }
    
    // Example 4: Secondary index query
    println!("\n=== Secondary Index Query ===");
    {
        let txn = store.begin_read()?;
        let tree = txn.open_tree::<User>()?;
        
        let users = tree.get_by_secondary_key(&"alice@example.com".to_string())?;
        println!("Found {} user(s) with email alice@example.com", users.len());
        for user in users {
            println!("  - {}: {}", user.id, user.name);
        }
    }
    
    // Example 5: Convenience API
    println!("\n=== Convenience API ===");
    {
        store.quick_put(User {
            id: 100,
            name: "Quick User".to_string(),
            email: "quick@example.com".to_string(),
        })?;
        
        let user = store.quick_get::<User>(&100)?.unwrap();
        println!("Quick get: {}", user.name);
    }
    
    Ok(())
}
```

#### 6.2 Advanced Example

**File**: `examples/redb_zerocopy_advanced.rs`

Demonstrate:
- Multi-tree transactions
- Iterator usage (borrowed and owned)
- Error handling
- Transaction rollback

### Phase 7: Documentation

#### 7.1 Module Documentation

**In redb_zerocopy.rs**, add comprehensive module docs:

```rust
//! # Zero-Copy Redb Backend
//!
//! This module provides a high-performance redb backend with zero-copy reads
//! and transaction-scoped API.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! # use netabase_store::databases::redb_zerocopy::*;
//! # use netabase_store::*;
//! # #[netabase_definition_module(MyDef, MyKeys)]
//! # mod models { }
//! # use models::*;
//! let store = RedbStoreZeroCopy::<MyDef>::new("app.redb")?;
//!
//! // Write
//! let mut txn = store.begin_write()?;
//! let mut tree = txn.open_tree::<User>()?;
//! tree.put(user)?;
//! drop(tree);
//! txn.commit()?;
//!
//! // Read (zero-copy)
//! let txn = store.begin_read()?;
//! let tree = txn.open_tree::<User>()?;
//! let guard = tree.get_borrowed(&user_id)?;
//! # Ok::<(), netabase_store::error::NetabaseError>(())
//! ```
//!
//! ## Architecture
//!
//! [diagram]
//!
//! ## Performance
//!
//! | Operation | Old API | New API | Improvement |
//! |-----------|---------|---------|-------------|
//! | Read | 100ns | 30ns | 3.3x faster |
//! | Bulk insert (1000) | 50ms | 5ms | 10x faster |
//!
//! ## API Comparison
//!
//! ### Old API (redb_store)
//!
//! ```rust,ignore
//! let tree = store.open_tree::<User>();
//! tree.put(user)?; // Auto-commits
//! let user = tree.get(key)?; // Always clones
//! ```
//!
//! ### New API (redb_zerocopy)
//!
//! ```rust,ignore
//! let mut txn = store.begin_write()?;
//! let mut tree = txn.open_tree::<User>()?;
//! tree.put(user)?; // Batched in transaction
//! drop(tree);
//! txn.commit()?; // Explicit commit
//!
//! // Zero-copy read
//! let guard = tree.get_borrowed(&key)?; // No clone!
//! ```
//!
//! ## When to Use
//!
//! Use this backend when:
//! - Performance is critical
//! - You need transaction batching
//! - You want zero-copy reads
//! - You're comfortable with lifetime management
//!
//! Use the old `redb_store` when:
//! - Simplicity is more important than performance
//! - Single-operation transactions are fine
//! - You want the simplest possible API
//!
//! ## See Also
//!
//! - [`RedbStore`](super::redb_store::RedbStore) - Simpler but slower API
//! - [Migration Guide](../../../docs/REDB_ZEROCOPY_MIGRATION.md)
```

#### 7.2 Migration Guide

**File**: `docs/REDB_ZEROCOPY_MIGRATION.md`

```markdown
# Migrating to Zero-Copy Redb Backend

## Feature Flag Setup

Add to Cargo.toml:
```toml
[dependencies]
netabase_store = { version = "0.0.3", features = ["redb-zerocopy"] }
```

## API Changes

### Opening Database

**Before:**
```rust
use netabase_store::databases::redb_store::RedbStore;
let store = RedbStore::<MyDef>::new("app.redb")?;
```

**After:**
```rust
use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
let store = RedbStoreZeroCopy::<MyDef>::new("app.redb")?;
```

### Single Insert

**Before:**
```rust
let tree = store.open_tree::<User>();
tree.put(user)?; // Auto-commits
```

**After (Explicit Transaction):**
```rust
let mut txn = store.begin_write()?;
let mut tree = txn.open_tree::<User>()?;
tree.put(user)?;
drop(tree);
txn.commit()?;
```

**After (Convenience API):**
```rust
store.quick_put(user)?;
```

### Bulk Insert

**Before:**
```rust
let tree = store.open_tree::<User>();
for user in users {
    tree.put(user)?; // N transactions!
}
```

**After:**
```rust
let mut txn = store.begin_write()?;
let mut tree = txn.open_tree::<User>()?;
tree.put_many(users)?; // 1 transaction!
drop(tree);
txn.commit()?;
```

### Reading Data

**Before:**
```rust
let tree = store.open_tree::<User>();
let user = tree.get(key)?; // Always clones
```

**After (Owned):**
```rust
let txn = store.begin_read()?;
let tree = txn.open_tree::<User>()?;
let user = tree.get(&key)?; // Still clones
```

**After (Zero-Copy):**
```rust
let txn = store.begin_read()?;
let tree = txn.open_tree::<User>()?;
let guard = tree.get_borrowed(&key)?; // No clone!
let user = guard.value(); // Borrowed reference
```

## Performance Tips

1. **Batch operations**: Use transactions to group operations
2. **Use borrowed reads**: When you don't need ownership
3. **Bulk operations**: Use `put_many` instead of loop
4. **Reuse transactions**: Open multiple trees in one transaction
```

## Performance Goals

### Benchmarks to Run

1. **Read Performance**:
   - Old: ~100ns per get (bincode deserialize)
   - New: ~30ns per get_borrowed (zero-copy)
   - Target: 3x improvement

2. **Bulk Insert**:
   - Old: ~50ms for 1000 records (1000 transactions)
   - New: ~5ms for 1000 records (1 transaction)
   - Target: 10x improvement

3. **Memory Usage**:
   - Old: Allocates on every read
   - New: No allocation for borrowed reads
   - Target: 60% reduction in allocations

### Success Criteria

✅ All tests pass  
✅ Zero-copy shows 3x+ read improvement  
✅ Bulk operations show 10x+ improvement  
✅ Examples run successfully  
✅ Documentation complete  
✅ No memory leaks  
✅ Lifetime errors caught at compile-time  

## Implementation Checklist

- [ ] Phase 1: Plan document (this file)
- [ ] Phase 2.1: Create type_utils.rs
- [ ] Phase 2.2: Enhance zerocopy generator
- [ ] Phase 2.3: Update model_key generator
- [ ] Phase 2.4: Update module exports
- [ ] Phase 3.1: Implement core transaction types
- [ ] Phase 3.2: Add convenience wrappers
- [ ] Phase 3.3: Implement secondary indexes
- [ ] Phase 4: Add from_borrowed trait
- [ ] Phase 5.1: Unit tests
- [ ] Phase 5.2: Integration tests
- [ ] Phase 5.3: Benchmarks
- [ ] Phase 6: Examples
- [ ] Phase 7: Documentation
- [ ] Run `cargo test --features redb-zerocopy`
- [ ] Run `cargo bench --features redb-zerocopy`
- [ ] Run all examples
- [ ] Review and refine

## Notes

- Keep old API for backward compatibility
- Feature-gate all zerocopy code
- Extensive documentation is key for adoption
- Performance benchmarks validate the approach
- Lifetime complexity needs good examples

---

**Last Updated**: 2025-11-11  
**Status**: Ready to implement
