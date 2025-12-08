# Tree Access Enums Pattern

## Overview

Tree Access Enums are a design pattern for type-safe, efficient tree/table identification without carrying data payloads.

## Problem Statement

Previously, we used data-containing enums for both:
1. **Data storage** - Secondary/relational keys with actual values
2. **Tree identification** - Determining which table to access

This created issues:
- ❌ Data enums implementing `AsRef<str>` was semantically incorrect
- ❌ Cloning data just to get a tree name was inefficient
- ❌ Mixed concerns (data vs. identification)

## Solution: Tree Access Enums

Separate enums with **no inner types**, used purely for tree identification.

### Example

```rust
// DATA-CONTAINING ENUM (for storage)
#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(UserSecondaryKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum UserSecondaryKeys {
    Email(UserEmail),      // Contains data
    Name(UserName),        // Contains data
    Age(UserAge),          // Contains data
}

// TREE ACCESS ENUM (for identification)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum UserSecondaryTreeNames {
    Email,    // No inner type!
    Name,     // No inner type!
    Age,      // No inner type!
}

impl DiscriminantName for UserSecondaryTreeNames {}
```

## Key Differences

| Aspect | Data Enum | Tree Access Enum |
|--------|-----------|------------------|
| Inner Types | ✅ Has data | ❌ No data |
| Copy | ❌ No (Clone only) | ✅ Yes |
| Size | Large (contains data) | Tiny (just discriminant) |
| AsRef<str> | ❌ No | ✅ Yes |
| Purpose | Store actual keys | Identify tables |
| Use Case | Database operations | Table lookup, registry |

## Benefits

### 1. **Type Safety**
```rust
// Compiler prevents using wrong tree name
fn open_user_email_table(tree: UserSecondaryTreeNames) {
    // Can only pass Email, Name, or Age
    // Cannot accidentally pass ProductSecondaryTreeNames
}
```

### 2. **Efficiency (Copy vs Clone)**
```rust
// Tree access enum - cheap copy
let tree = UserSecondaryTreeNames::Email;
let tree_copy = tree;  // Just copies a small discriminant
process_tree(tree);     // Original still usable
process_tree(tree_copy); // Copy still usable

// Data enum - expensive clone
let key = UserSecondaryKeys::Email(UserEmail("test@example.com".into()));
let key_clone = key.clone();  // Allocates and copies String
// Original is moved, can't use 'key' here
```

### 3. **Clear Separation of Concerns**
```rust
// Table registry uses tree names only
pub struct TableRegistry {
    user_secondary_tables: HashMap<UserSecondaryTreeNames, TableDef>,
    product_secondary_tables: HashMap<ProductSecondaryTreeNames, TableDef>,
}

// No need to carry data around just to identify tables
impl TableRegistry {
    pub fn get_user_secondary_table(&self, tree: UserSecondaryTreeNames) -> &TableDef {
        &self.user_secondary_tables[&tree]
    }
}
```

### 4. **Exhaustive Matching**
```rust
fn generate_table_name(tree: UserSecondaryTreeNames) -> String {
    match tree {
        UserSecondaryTreeNames::Email => "User_sec_Email".to_string(),
        UserSecondaryTreeNames::Name => "User_sec_Name".to_string(),
        UserSecondaryTreeNames::Age => "User_sec_Age".to_string(),
        // Compiler ensures all variants are handled
    }
}
```

## Usage Patterns

### Pattern 1: Table Registration
```rust
pub struct TableDefRegistry {
    secondary_trees: HashMap<UserSecondaryTreeNames, TableDefinition<...>>,
}

impl TableDefRegistry {
    pub fn register_all() -> Self {
        let mut registry = Self {
            secondary_trees: HashMap::new(),
        };

        // Iterate over all tree names (cheap, Copy)
        for tree_name in UserSecondaryTreeNames::iter() {
            let table_def = create_table_def(tree_name);
            registry.secondary_trees.insert(tree_name, table_def);
        }

        registry
    }
}
```

### Pattern 2: Dynamic Table Access
```rust
pub fn access_secondary_table<F>(
    tree: UserSecondaryTreeNames,
    operation: F,
) -> Result<()>
where
    F: FnOnce(/* table */) -> Result<()>,
{
    let table_name = tree.as_ref();  // "Email", "Name", or "Age"
    let table_def = TableDefinition::new(table_name);
    // ... open table and execute operation
    Ok(())
}

// Usage
access_secondary_table(UserSecondaryTreeNames::Email, |table| {
    // Work with email table
    Ok(())
})?;
```

### Pattern 3: Conversion Between Discriminant and Tree Name
```rust
impl From<UserSecondaryKeysDiscriminants> for UserSecondaryTreeNames {
    fn from(discriminant: UserSecondaryKeysDiscriminants) -> Self {
        match discriminant {
            UserSecondaryKeysDiscriminants::Email => UserSecondaryTreeNames::Email,
            UserSecondaryKeysDiscriminants::Name => UserSecondaryTreeNames::Name,
            UserSecondaryKeysDiscriminants::Age => UserSecondaryTreeNames::Age,
        }
    }
}

// Or vice versa
impl From<UserSecondaryTreeNames> for UserSecondaryKeysDiscriminants {
    fn from(tree: UserSecondaryTreeNames) -> Self {
        match tree {
            UserSecondaryTreeNames::Email => UserSecondaryKeysDiscriminants::Email,
            UserSecondaryTreeNames::Name => UserSecondaryKeysDiscriminants::Name,
            UserSecondaryTreeNames::Age => UserSecondaryKeysDiscriminants::Age,
        }
    }
}
```

### Pattern 4: Iterator Pattern
```rust
// Iterate over all possible trees
for tree_name in UserSecondaryTreeNames::iter() {
    let table_name = tree_name.as_ref();
    println!("Initializing table: {}", table_name);
    initialize_table(table_name);
}

// Works great with EnumIter from strum
```

## Implementation Checklist

For each model, create:

- [ ] Secondary tree access enum (if model has secondary keys)
- [ ] Relational tree access enum (if model has relationships)
- [ ] Impl DiscriminantName for both
- [ ] Derive: Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr

### Template

```rust
// 1. Define discriminants via strum
#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(ModelSecondaryKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum ModelSecondaryKeys {
    Field1(Field1Type),
    Field2(Field2Type),
}

impl DiscriminantName for ModelSecondaryKeysDiscriminants {}

// 2. Define tree access enum (mirrors variants, no inner types)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum ModelSecondaryTreeNames {
    Field1,  // No inner type
    Field2,  // No inner type
}

impl DiscriminantName for ModelSecondaryTreeNames {}

// 3. Repeat for relational keys
#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(ModelRelationalKeysDiscriminants))]
#[strum_discriminants(derive(Hash, AsRefStr, Encode, Decode, EnumIter))]
pub enum ModelRelationalKeys {
    Relationship1(RelatedModelKey),
    Relationship2(OtherModelKey),
}

impl DiscriminantName for ModelRelationalKeysDiscriminants {}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr)]
pub enum ModelRelationalTreeNames {
    Relationship1,
    Relationship2,
}

impl DiscriminantName for ModelRelationalTreeNames {}
```

## Migration Guide

### Before (Data enum for tree identification)
```rust
// Mixed concerns - using data enum for identification
let email_key = UserSecondaryKeys::Email(UserEmail("test@example.com".into()));
let table_name = email_key.as_ref();  // ❌ AsRef<str> on data enum
let table_def = TableDefinition::new(table_name);

// Problem: Had to clone/construct data just to get tree name
```

### After (Tree access enum)
```rust
// Separate identification from data
let tree = UserSecondaryTreeNames::Email;  // ✅ Just identification
let table_name = tree.as_ref();            // ✅ AsRef<str> on tree enum
let table_def = TableDefinition::new(table_name);

// Data is only created when actually storing/fetching
let email_data = UserSecondaryKeys::Email(UserEmail("test@example.com".into()));
```

## Performance Impact

### Memory
- Tree enum: ~1 byte (just discriminant)
- Data enum: varies (discriminant + data size)

### Speed
- Copy: Extremely fast (single register or stack operation)
- Clone: Slower (may involve heap allocation)

### Example Benchmark (Conceptual)
```
Tree enum copy:     ~1ns
Data enum clone:    ~50ns (with String allocation)

50x faster for tree identification!
```

## When to Use Each

### Use Tree Access Enum When:
- ✅ Registering tables
- ✅ Looking up table definitions
- ✅ Iterating over possible trees
- ✅ Switching on tree type
- ✅ Storing tree identifiers in collections

### Use Data Enum When:
- ✅ Actually storing/fetching data
- ✅ Indexing by secondary key value
- ✅ Serializing for network/disk
- ✅ Comparing key values

## Future Enhancements

### Macro Generation
Could auto-generate tree access enums from data enums:
```rust
#[derive(NetabaseModel)]
pub struct User { /* ... */ }

// Auto-generates:
// - UserSecondaryKeys (data)
// - UserSecondaryTreeNames (tree access)
// - Conversions between them
```

### Type-Level Tree Registry
```rust
trait TreeRegistry {
    type SecondaryTrees;
    type RelationalTrees;
}

impl TreeRegistry for User {
    type SecondaryTrees = UserSecondaryTreeNames;
    type RelationalTrees = UserRelationalTreeNames;
}
```

## Conclusion

Tree Access Enums provide:
- ✅ Clear separation between data and identification
- ✅ Type safety through distinct types
- ✅ Performance through Copy instead of Clone
- ✅ Correctness through proper AsRef<str> placement
- ✅ Maintainability through single source of truth

They are a foundational pattern for the netabase_store architecture.
