# Relational Links in Netabase Store

## Table of Contents

- [Overview](#overview)
- [Concepts](#concepts)
  - [Entity vs Reference](#entity-vs-reference)
  - [Custom Relation Names](#custom-relation-names)
  - [Generated Types and Methods](#generated-types-and-methods)
- [Usage](#usage)
  - [Basic Setup](#basic-setup)
  - [Insertion Methods](#insertion-methods)
  - [Hydration](#hydration)
  - [Helper Methods](#helper-methods)
- [Performance](#performance)
- [Limitations](#limitations)
- [Examples](#examples)

## Overview

Relational Links is a feature that enables type-safe relationships between models in Netabase Store. It provides automatic insertion of related entities, hydration of references, and compile-time guarantees about relationship validity.

### Key Benefits

- **Type Safety**: Relationships are checked at compile time
- **Automatic Insertion**: Related entities can be inserted atomically with the parent model
- **Flexible Storage**: Choose between embedding entities or storing references
- **Zero Runtime Overhead**: When using references, there's no serialization overhead
- **Custom Naming**: Define semantic relation names for better code clarity

## Concepts

### Entity vs Reference

The `RelationalLink<D, M>` enum has two variants:

```rust
pub enum RelationalLink<D, M> {
    /// The related entity is embedded directly
    Entity(M),
    /// The related entity is referenced by its primary key
    Reference(M::PrimaryKey),
}
```

**Entity**: Stores the full related model inline. Useful for:
- Ensuring related data is always available
- Atomic inserts of parent and children
- Denormalized data patterns

**Reference**: Stores only the primary key. Useful for:
- Normalized data patterns
- Reducing serialization size
- Avoiding data duplication

### Custom Relation Names

Use the `#[relation(name)]` attribute to give semantic names to relations:

```rust
#[derive(NetabaseModel)]
#[netabase(BlogDef)]
pub struct Post {
    #[primary_key]
    pub id: u64,
    pub title: String,

    #[relation(post_author)]  // Custom name instead of "author"
    pub author: RelationalLink<BlogDef, User>,
}
```

This generates a relation enum with the custom name:

```rust
pub enum PostRelations {
    PostAuthor,  // Instead of "Author"
}
```

### Generated Types and Methods

For each model with relational links, the following are generated:

#### 1. Relations Enum

```rust
#[derive(Debug, Clone, PartialEq, Eq, /* ... */)]
pub enum {Model}Relations {
    {RelationName},
    // ... one variant per relation field
}
```

#### 2. Trait Implementations

**`HasCustomRelationInsertion<D>`**: Marker trait indicating the model has relations

```rust
impl HasCustomRelationInsertion<BlogDef> for Post {
    const HAS_RELATIONS: bool = true;
}
```

**`NetabaseRelationTrait<D>`**: Provides relation insertion methods

```rust
impl NetabaseRelationTrait<BlogDef> for Post {
    type Relations = PostRelations;

    fn insert_with_relations<S>(&self, store: &S) -> Result<(), NetabaseError>
    where
        S: MultiModelStore<BlogDef>;

    fn insert_relations_only<S>(&self, store: &S) -> Result<(), NetabaseError>
    where
        S: MultiModelStore<BlogDef>;
}
```

#### 3. Standalone Methods

```rust
impl Post {
    /// Insert this model with all its related entities
    pub fn insert_with_relations<S>(&self, store: &S) -> Result<(), NetabaseError>
    where
        S: OpenTree<BlogDef, Self>,
        S: OpenTree<BlogDef, User>,
        Self: Clone + NetabaseModelTrait<BlogDef>;
}
```

#### 4. Helper Methods per Relation

For each relation field, several helper methods are generated:

```rust
impl Post {
    /// Get the relational link
    pub fn get_author(&self) -> &RelationalLink<BlogDef, User>;

    /// Hydrate the linked entity if it's a reference
    pub fn hydrate_author<T>(&self, store: T) -> Result<Option<User>, NetabaseError>
    where
        T: StoreOps<BlogDef, User>;

    /// Check if this field contains an entity
    pub fn is_author_entity(&self) -> bool;

    /// Check if this field contains a reference
    pub fn is_author_reference(&self) -> bool;

    /// Insert the linked entity if it's an Entity variant
    pub fn insert_author_if_entity<S>(&self, store: &S) -> Result<(), NetabaseError>
    where
        S: OpenTree<BlogDef, User>;
}
```

## Usage

### Basic Setup

Define your models with `RelationalLink` fields:

```rust
use netabase_store::{
    NetabaseModel, NetabaseStore,
    links::RelationalLink,
    netabase_definition_module,
};

#[netabase_definition_module(BlogDef, BlogKeys)]
mod models {
    use super::*;
    use netabase_store::netabase;

    #[derive(NetabaseModel, Clone, Debug, PartialEq,
             bincode::Encode, bincode::Decode,
             serde::Serialize, serde::Deserialize)]
    #[netabase(BlogDef)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub email: String,
    }

    #[derive(NetabaseModel, Clone, Debug, PartialEq,
             bincode::Encode, bincode::Decode,
             serde::Serialize, serde::Deserialize)]
    #[netabase(BlogDef)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,

        #[relation(author)]
        pub author: RelationalLink<BlogDef, User>,
    }
}
```

### Insertion Methods

#### 1. Insert with Embedded Entity

```rust
let user = User {
    id: 1,
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
};

let post = Post {
    id: 1,
    title: "Hello World".to_string(),
    content: "...".to_string(),
    author: RelationalLink::Entity(user),
};

// Inserts both the post AND the user
post.insert_with_relations(&store)?;
```

#### 2. Insert with Reference

```rust
// Insert user first
let user = User { id: 1, /* ... */ };
user_tree.put(user)?;

// Create post with reference
let post = Post {
    id: 1,
    title: "Hello World".to_string(),
    content: "...".to_string(),
    author: RelationalLink::Reference(UserPrimaryKey(1)),
};

// Only inserts the post (user already exists)
post.insert_with_relations(&store)?;
```

#### 3. Insert Only Relations

```rust
// Insert only the related entities, not the parent
post.insert_relations_only(&store)?;
```

### Hydration

Convert a `Reference` to an `Entity` by loading from the store:

```rust
let post = post_tree.get(PostPrimaryKey(1))?.unwrap();

// Using the generated helper method
if let Some(author) = post.hydrate_author(&user_tree)? {
    println!("Author: {}", author.name);
}

// Using the RelationalLink method directly
let author_link = &post.author;
if let Some(author) = author_link.hydrate(&user_tree)? {
    println!("Author: {}", author.name);
}
```

### Helper Methods

```rust
// Check the variant type
if post.is_author_entity() {
    println!("Author is embedded");
}

if post.is_author_reference() {
    println!("Author is a reference");
}

// Get the link
let author_link = post.get_author();

// Insert only if it's an entity
post.insert_author_if_entity(&store)?;
```

## Performance

### Benchmark Results

Based on the `relational_links_overhead` benchmark:

| Operation | Plain Models | Relational (Reference) | Relational (Entity) | Overhead |
|-----------|-------------|----------------------|---------------------|----------|
| 10 Inserts | baseline | ~5% slower | ~10% slower | Minimal |
| 100 Inserts | baseline | ~5% slower | ~10% slower | Minimal |
| 1000 Inserts | baseline | ~5% slower | ~12% slower | Minimal |
| Hydration (100) | N/A | ~15% of insert time | N/A | Acceptable |

### Performance Tips

1. **Use References for Large Models**: If the related model is large, use references to avoid serialization overhead
2. **Use Entities for Small, Frequently-Accessed Models**: Embedding small models avoids the hydration lookup
3. **Batch Operations**: When inserting multiple models with the same relations, consider inserting the relations once and using references
4. **Hydration Caching**: If you need to hydrate the same reference multiple times, cache the result

### Serialization Overhead

```rust
// Plain post with foreign key: ~200 bytes
struct PlainPost {
    id: u64,
    title: String,
    content: String,
    author_id: u64,  // 8 bytes
}

// Post with Reference: ~208 bytes (+8 bytes for enum discriminant)
struct PostWithReference {
    id: u64,
    title: String,
    content: String,
    author: RelationalLink<_, User>,  // Reference variant
}

// Post with embedded Entity: ~200 + sizeof(User) bytes
struct PostWithEntity {
    id: u64,
    title: String,
    content: String,
    author: RelationalLink<_, User>,  // Entity variant
}
```

## Limitations

### Current Limitations

1. **No Collection Support**: `Vec<RelationalLink<D, M>>`, `Option<RelationalLink<D, M>>`, and `Box<RelationalLink<D, M>>` are not currently supported. Relations must be direct fields.

2. **No Recursive Relations**: Self-referential models (e.g., a Comment that has a parent Comment) are not yet supported.

3. **Single-Level Relations**: Relations of relations are not automatically inserted. Each level must be inserted explicitly.

### Future Enhancements

- Collection support for one-to-many and many-to-many relationships
- Recursive relation insertion with depth control
- Cascade delete options
- Bi-directional relation tracking
- Relation queries and filters

## Examples

### Example 1: Blog System

```rust
#[netabase_definition_module(BlogDef, BlogKeys)]
mod models {
    use super::*;
    use netabase_store::netabase;

    #[derive(NetabaseModel, Clone, Debug, PartialEq,
             bincode::Encode, bincode::Decode,
             serde::Serialize, serde::Deserialize)]
    #[netabase(BlogDef)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub email: String,
    }

    #[derive(NetabaseModel, Clone, Debug, PartialEq,
             bincode::Encode, bincode::Decode,
             serde::Serialize, serde::Deserialize)]
    #[netabase(BlogDef)]
    pub struct Category {
        #[primary_key]
        pub id: u64,
        pub name: String,
    }

    #[derive(NetabaseModel, Clone, Debug, PartialEq,
             bincode::Encode, bincode::Decode,
             serde::Serialize, serde::Deserialize)]
    #[netabase(BlogDef)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,

        #[relation(post_author)]
        pub author: RelationalLink<BlogDef, User>,

        #[relation(post_category)]
        pub category: RelationalLink<BlogDef, Category>,
    }
}

use models::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = NetabaseStore::temp()?;

    // Create entities
    let author = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let category = Category {
        id: 1,
        name: "Technology".to_string(),
    };

    // Create post with embedded entities
    let post = Post {
        id: 1,
        title: "Rust Programming".to_string(),
        content: "An introduction to Rust...".to_string(),
        author: RelationalLink::Entity(author),
        category: RelationalLink::Entity(category),
    };

    // Insert post and all related entities atomically
    post.insert_with_relations(&store)?;

    // Retrieve and verify
    let post_tree = store.open_tree::<Post>();
    let retrieved = post_tree.get(PostPrimaryKey(1))?.unwrap();

    if let Some(author) = retrieved.hydrate_author(&store.open_tree::<User>())? {
        println!("Post by: {}", author.name);
    }

    Ok(())
}
```

### Example 2: Mixed Entities and References

```rust
// Insert some users first
let user1 = User { id: 1, name: "Alice".to_string(), /* ... */ };
let user2 = User { id: 2, name: "Bob".to_string(), /* ... */ };

user_tree.put(user1.clone())?;
user_tree.put(user2.clone())?;

// Create posts with different link types
let post1 = Post {
    id: 1,
    title: "Post 1".to_string(),
    content: "...".to_string(),
    author: RelationalLink::Entity(user1), // Embedded
};

let post2 = Post {
    id: 2,
    title: "Post 2".to_string(),
    content: "...".to_string(),
    author: RelationalLink::Reference(UserPrimaryKey(2)), // Reference
};

// Both work with the same insertion method
post1.insert_with_relations(&store)?;
post2.insert_with_relations(&store)?;
```

### Example 3: Hydration Patterns

```rust
let posts: Vec<Post> = /* ... load posts ... */;
let user_tree = store.open_tree::<User>();

// Pattern 1: Individual hydration
for post in &posts {
    if let Some(author) = post.hydrate_author(&user_tree)? {
        println!("{} by {}", post.title, author.name);
    }
}

// Pattern 2: Cached hydration (for better performance)
let mut author_cache = std::collections::HashMap::new();

for post in &posts {
    if let RelationalLink::Reference(user_id) = &post.author {
        let author = author_cache
            .entry(user_id.clone())
            .or_insert_with(|| {
                user_tree.get(user_id.clone()).ok().flatten()
            });

        if let Some(author) = author {
            println!("{} by {}", post.title, author.name);
        }
    }
}
```

## Testing

See the `tests/relational_links_tests.rs` file for comprehensive tests covering:

- Basic insertion with entities and references
- Hydration functionality
- Helper method correctness
- Error handling
- Performance characteristics

## Contributing

When adding new relational link features:

1. Update this documentation
2. Add tests to `tests/relational_links_tests.rs`
3. Add benchmarks to `benches/relational_links_overhead.rs`
4. Update the examples in `examples/`

## License

Same as the Netabase Store project license.
