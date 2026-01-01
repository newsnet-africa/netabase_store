# Netabase Store - Comprehensive Test Suite Documentation

This document provides detailed documentation of the test suite, explaining what each test validates and how to use the library's API.

## Core CRUD Operations

### Creating Models

**API:** `transaction.create_redb(&model)`

Creates a new model instance in the database. All fields are persisted including:
- Primary key (unique identifier)
- Secondary keys (indexed fields for queries)
- Relational links (references to other models)
- Subscriptions (pub/sub topic associations)
- Blob data (automatically chunked for large data)

**Example:**
```rust
let user = User {
    id: UserID("alice_123".to_string()),
    name: "Alice Johnson".to_string(),
    age: 28,
    partner: RelationalLink::new_dehydrated(UserID("bob_456".to_string())),
    category: RelationalLink::new_dehydrated(CategoryID("tech".to_string())),
    subscriptions: vec![DefinitionSubscriptions::Topic1],
    bio: LargeUserFile { data: vec![1, 2, 3], metadata: "Bio".to_string() },
    another: AnotherLargeUserFile(vec![10, 20, 30]),
};

let txn = store.begin_transaction()?;
txn.create_redb(&user)?;
txn.commit()?;
```

### Reading Models

**API:** `Model::read_default(&primary_key, &tables)`

Retrieves a model by its primary key. Returns `Option<Model>` - `None` if not found.

**Example:**
```rust
let txn = store.begin_transaction()?;
{
    let table_defs = User::table_definitions();
    let tables = txn.open_model_tables(table_defs, None)?;
    
    if let Some(user) = User::read_default(&user_id, &tables)? {
        println!("Found user: {} (age: {})", user.name, user.age);
    }
}
txn.commit()?;
```

### Updating Models

**API:** `transaction.update_redb(&model)`

Updates an existing model. The primary key must match for the update to locate the correct record.
All fields are updated to the new values, including indexes.

**Example:**
```rust
let updated_user = User {
    id: user_id.clone(),  // Same ID
    name: "Updated Name".to_string(),  // Changed
    age: 30,  // Changed
    // ... other fields
};

let txn = store.begin_transaction()?;
txn.update_redb(&updated_user)?;
txn.commit()?;
```

### Deleting Models

**API:** `transaction.delete_redb::<Model>(&primary_key)`

Removes a model and all its associated indexes. Idempotent - deleting a non-existent model succeeds.

**Example:**
```rust
let txn = store.begin_transaction()?;
txn.delete_redb::<User>(&user_id)?;
txn.commit()?;
```

## Relational Links - The Four Variants

Relational links connect models together. There are four variants optimized for different use cases:

### 1. Dehydrated - Primary Key Only
- **Memory**: Minimal (just the primary key)
- **Lifetime**: No lifetime constraints (`'static`)
- **Use case**: Serialization, storage, passing around references
- **API**: `RelationalLink::new_dehydrated(primary_key)`

```rust
let partner_link = RelationalLink::<Standalone, Definition, Definition, User>::new_dehydrated(
    UserID("bob_123".to_string())
);

assert!(partner_link.is_dehydrated());
assert_eq!(partner_link.get_primary_key().0, "bob_123");
assert!(partner_link.get_model().is_none());  // No model data
```

### 2. Owned - Full Model Ownership
- **Memory**: Stores complete model in `Box<M>`
- **Lifetime**: No lifetime constraints (`'static`)
- **Use case**: When you construct a model and want to store it with full ownership
- **API**: `RelationalLink::new_owned(primary_key, model)`

```rust
let partner = User { /* ... */ };
let partner_link = RelationalLink::<Standalone, Definition, Definition, User>::new_owned(
    UserID("bob_123".to_string()),
    partner
);

assert!(partner_link.is_owned());
let model_ref = partner_link.get_model().unwrap();
println!("Partner name: {}", model_ref.name);

// Can extract the owned model
let extracted: Option<User> = partner_link.clone().into_owned();
```

### 3. Hydrated - User-Controlled Reference
- **Memory**: Stores reference to model
- **Lifetime**: Tied to referenced data (`'data`)
- **Use case**: When you have a model in memory and want to create a link to it
- **API**: `RelationalLink::new_hydrated(primary_key, &model)`

```rust
let partner = User { /* ... */ };
let partner_link = RelationalLink::<Standalone, Definition, Definition, User>::new_hydrated(
    UserID("bob_123".to_string()),
    &partner  // Reference
);

assert!(partner_link.is_hydrated());
let model_ref = partner_link.get_model().unwrap();
```

### 4. Borrowed - Database AccessGuard Reference
- **Memory**: Stores reference from database
- **Lifetime**: Tied to `AccessGuard` (`'data`)
- **Use case**: Zero-copy access from database reads
- **API**: `RelationalLink::new_borrowed(primary_key, &model)`

```rust
// Typically created automatically by database operations
let partner_link = RelationalLink::<Standalone, Definition, Definition, User>::new_borrowed(
    UserID("bob_123".to_string()),
    &partner_from_db  // From database AccessGuard
);

assert!(partner_link.is_borrowed());
let model_ref = partner_link.as_borrowed().unwrap();
```

### Variant Ordering and Conversions

Variants have a natural ordering: `Dehydrated < Owned < Hydrated < Borrowed`

All variants can be converted to `Dehydrated` using `.dehydrate()`:
```rust
let dehydrated = owned_link.dehydrate();  // Drops the model, keeps key
let dehydrated = hydrated_link.dehydrate();  // Drops the reference, keeps key
let dehydrated = borrowed_link.dehydrate();  // Drops the reference, keeps key
```

## Transactions

### Atomicity
Changes in a transaction are all-or-nothing:

```rust
let txn = store.begin_transaction()?;
txn.create_redb(&user1)?;
txn.create_redb(&user2)?;
txn.create_redb(&user3)?;
txn.commit()?;  // All three created atomically

// Or drop without committing for automatic rollback
{
    let txn = store.begin_transaction()?;
    txn.create_redb(&user)?;
    // Transaction dropped here - rollback, user not persisted
}
```

### Batching Operations
Multiple operations in a single transaction are more efficient:

```rust
let txn = store.begin_transaction()?;
for i in 0..1000 {
    let user = create_user(i);
    txn.create_redb(&user)?;
}
txn.commit()?;  // All 1000 users committed at once
```

## Listing and Counting

### Count Entries
Get the total number of model instances:

```rust
let txn = store.begin_transaction()?;
{
    let table_defs = User::table_definitions();
    let tables = txn.open_model_tables(table_defs, None)?;
    
    let count = User::count_entries(&tables)?;
    println!("Total users: {}", count);
}
txn.commit()?;
```

### List All Entries
Retrieve all instances of a model:

```rust
let txn = store.begin_transaction()?;
{
    let table_defs = User::table_definitions();
    let tables = txn.open_model_tables(table_defs, None)?;
    
    let users: Vec<User> = User::list_default(&tables)?;
    for user in users {
        println!("User: {} (age: {})", user.name, user.age);
    }
}
txn.commit()?;
```

## Blob Storage

Large data is automatically chunked and stored efficiently:

```rust
// Create user with 200KB of blob data
let large_bio_data: Vec<u8> = (0..200_000).map(|i| (i % 256) as u8).collect();

let user = User {
    id: UserID("blob_user".to_string()),
    // ... other fields
    bio: LargeUserFile {
        data: large_bio_data.clone(),
        metadata: "Large bio".to_string(),
    },
    // ...
};

// Store (automatically chunked)
let txn = store.begin_transaction()?;
txn.create_redb(&user)?;
txn.commit()?;

// Retrieve (automatically reassembled)
let txn = store.begin_transaction()?;
{
    let table_defs = User::table_definitions();
    let tables = txn.open_model_tables(table_defs, None)?;
    
    let read_user = User::read_default(&user_id, &tables)?.unwrap();
    assert_eq!(read_user.bio.data, large_bio_data);  // Identical!
}
txn.commit()?;
```

## Subscriptions

Models can subscribe to topics for pub/sub functionality:

```rust
let user = User {
    id: UserID("alice".to_string()),
    // ... other fields
    subscriptions: vec![
        DefinitionSubscriptions::Topic1,
        DefinitionSubscriptions::Topic2,
    ],
    // ...
};

// Store with subscriptions
let txn = store.begin_transaction()?;
txn.create_redb(&user)?;
txn.commit()?;

// Read back and verify
let txn = store.begin_transaction()?;
{
    let table_defs = User::table_definitions();
    let tables = txn.open_model_tables(table_defs, None)?;
    
    let read_user = User::read_default(&user_id, &tables)?.unwrap();
    assert_eq!(read_user.subscriptions.len(), 2);
    assert!(read_user.subscriptions.contains(&DefinitionSubscriptions::Topic1));
}
txn.commit()?;
```

## Repository Isolation

Definitions can exist in repositories for isolation:

### Standalone Repository (Default)
Definitions without explicit `repos()` belong to the `Standalone` repository and can link to each other:

```rust
// Both User and Category are in Standalone (no repos() specified)
let user = User {
    id: UserID("alice".to_string()),
    partner: RelationalLink::new_dehydrated(UserID("bob".to_string())),  // Same definition
    category: RelationalLink::new_dehydrated(CategoryID("tech".to_string())),  // Cross-definition
    // ...
};
```

### Explicit Repositories
When definitions specify `repos(RepoName)`, they can only link to others in the same repository,
enforced at compile time for security.

## Error Handling

### Graceful Non-Existence
Reading or deleting non-existent models doesn't error:

```rust
// Reading
let result = User::read_default(&UserID("nonexistent".to_string()), &tables)?;
assert!(result.is_none());  // Returns None, not an error

// Deleting
txn.delete_redb::<User>(&UserID("nonexistent".to_string()))?;  // Succeeds
```

### Empty Database Operations
Operations on empty databases return appropriate empty results:

```rust
let count = User::count_entries(&tables)?;  // Returns 0
let users = User::list_default(&tables)?;   // Returns vec![]
let user = User::read_default(&id, &tables)?;  // Returns None
```

## Complex Scenarios

### Multi-Model Relationships

```rust
// Create interconnected network
let alice = User {
    id: alice_id.clone(),
    partner: RelationalLink::new_dehydrated(bob_id.clone()),  // Links to Bob
    category: RelationalLink::new_dehydrated(tech_category.clone()),
    // ...
};

let bob = User {
    id: bob_id.clone(),
    partner: RelationalLink::new_dehydrated(alice_id.clone()),  // Links back to Alice
    category: RelationalLink::new_dehydrated(tech_category.clone()),
    // ...
};

// Store both
let txn = store.begin_transaction()?;
txn.create_redb(&alice)?;
txn.create_redb(&bob)?;
txn.commit()?;

// Verify bidirectional relationship
let txn = store.begin_transaction()?;
{
    let table_defs = User::table_definitions();
    let tables = txn.open_model_tables(table_defs, None)?;
    
    let alice_read = User::read_default(&alice_id, &tables)?.unwrap();
    let bob_read = User::read_default(&bob_id, &tables)?.unwrap();
    
    assert_eq!(alice_read.partner.get_primary_key(), &bob_id);
    assert_eq!(bob_read.partner.get_primary_key(), &alice_id);
}
txn.commit()?;
```

## Test Organization

Tests are organized by functionality:
- `integration_crud.rs` - Basic CRUD operations
- `integration_indexes.rs` - Secondary key behavior
- `integration_list.rs` - Listing and counting
- `comprehensive_functionality.rs` - Full API surface documentation

Each test demonstrates a specific feature and verifies the database state thoroughly.
