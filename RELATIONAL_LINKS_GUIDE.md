# Relational Links Guide

Complete guide to using relational links in NetabaseStore for building graph-based data structures.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Core Concepts](#core-concepts)
4. [Model Definition](#model-definition)
5. [Working with Links](#working-with-links)
6. [Generated Helper Methods](#generated-helper-methods)
7. [Insertion Strategies](#insertion-strategies)
8. [Query Patterns](#query-patterns)
9. [Best Practices](#best-practices)
10. [Advanced Patterns](#advanced-patterns)

## Overview

Relational links provide a type-safe way to model relationships between entities in NetabaseStore. They support:

- **Eager Loading**: Embed full entities for immediate access
- **Lazy Loading**: Store only keys and load entities on demand
- **Automatic Code Generation**: Helper methods for common operations
- **Type Safety**: Compile-time checking of relationship types
- **Flexible Serialization**: Preserve link structure across serialization

## Quick Start

```rust
use netabase_store::{NetabaseModel, netabase, netabase_definition_module};
use netabase_store::links::RelationalLink;

// Define your schema
#[netabase_definition_module(BlogDef, BlogKeys)]
mod models {
    use super::*;
    use netabase_store::{NetabaseModel, netabase};

    #[derive(NetabaseModel, Clone, Debug, PartialEq,
             bincode::Encode, bincode::Decode,
             serde::Serialize, serde::Deserialize)]
    #[netabase(BlogDef)]
    pub struct User {
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

        // Relational link to User
        pub author: RelationalLink<BlogDef, User>,
    }
}

use models::*;

// Create entities
let user = User {
    id: 1,
    name: "Alice".to_string(),
};

// Create post with embedded author (eager loading)
let post = Post {
    id: 1,
    title: "Hello World".to_string(),
    content: "My first post!".to_string(),
    author: RelationalLink::Entity(user),
};

// Or create with just a reference (lazy loading)
let post = Post {
    id: 1,
    title: "Hello World".to_string(),
    content: "My first post!".to_string(),
    author: RelationalLink::Reference(UserPrimaryKey(1)),
};
```

## Core Concepts

### RelationalLink<D, M>

The core type representing a relationship. It has two variants:

1. **Reference(PrimaryKey)**: Stores only the key
   - Minimal memory footprint
   - Requires loading to access entity data
   - Prevents circular dependencies

2. **Entity(M)**: Stores the complete entity
   - Immediate access to data
   - No additional database lookups
   - Useful for bundling related data

### Type Parameters

- `D`: Your definition trait (e.g., `BlogDef`)
- `M`: The target model type (e.g., `User`)

### NetabaseRelationTrait

Models with `RelationalLink` fields automatically implement this trait, providing:
- Metadata about relationships
- Runtime introspection capabilities
- Generated helper methods

## Model Definition

### Basic Relationship

```rust
#[derive(NetabaseModel, Clone, Debug, PartialEq,
         bincode::Encode, bincode::Decode,
         serde::Serialize, serde::Deserialize)]
#[netabase(BlogDef)]
pub struct Post {
    #[primary_key]
    pub id: u64,
    pub title: String,

    // Single relationship
    pub author: RelationalLink<BlogDef, User>,
}
```

### Multiple Relationships

```rust
#[derive(NetabaseModel, Clone, Debug, PartialEq,
         bincode::Encode, bincode::Decode,
         serde::Serialize, serde::Deserialize)]
#[netabase(BlogDef)]
pub struct Post {
    #[primary_key]
    pub id: u64,
    pub title: String,

    // Multiple different relationships
    pub author: RelationalLink<BlogDef, User>,
    pub category: RelationalLink<BlogDef, Category>,
    pub editor: RelationalLink<BlogDef, User>,  // Can link to same type
}
```

### Collection Relationships

```rust
#[derive(NetabaseModel, Clone, Debug, PartialEq,
         bincode::Encode, bincode::Decode,
         serde::Serialize, serde::Deserialize)]
#[netabase(BlogDef)]
pub struct Post {
    #[primary_key]
    pub id: u64,
    pub title: String,

    // One-to-many: Post has many Comments
    pub comments: Vec<RelationalLink<BlogDef, Comment>>,

    // Many-to-many: Post has many Tags
    pub tags: Vec<RelationalLink<BlogDef, Tag>>,
}
```

## Working with Links

### Creating Links

```rust
// From an entity (eager)
let link = RelationalLink::Entity(user);

// From a key (lazy)
let link = RelationalLink::Reference(user_id);

// Using From trait
let link: RelationalLink<BlogDef, User> = user.into();

// Using constructor methods
let link = RelationalLink::from_entity(user);
let link = RelationalLink::from_key(user_id);
```

### Accessing Data

```rust
// Get the key (works for both variants)
let user_id = link.key();

// Check which variant
if link.is_entity() {
    println!("Entity is loaded");
}
if link.is_reference() {
    println!("Only have the reference");
}

// Extract entity if present
if let Some(user) = link.as_entity() {
    println!("User: {}", user.name);
}

// Extract key if it's a reference
if let Some(key) = link.as_reference() {
    println!("User ID: {:?}", key);
}
```

### Hydration (Loading Entities)

```rust
// Load entity from store if it's a reference
let user_option = link.hydrate(&store)?;

if let Some(user) = user_option {
    println!("Loaded user: {}", user.name);
}

// Hydrate collections
let comments: Vec<Comment> = post.comments
    .into_iter()
    .filter_map(|link| link.hydrate(&store).ok().flatten())
    .collect();
```

### Converting Between Variants

```rust
// Convert Entity to Reference
let entity_link = RelationalLink::Entity(user);
let ref_link = entity_link.to_reference();  // Extracts key

// No direct Reference -> Entity conversion
// Use hydrate() instead
```

## Generated Helper Methods

For each `RelationalLink` field, the macro generates helpful methods:

### Example Model

```rust
pub struct Post {
    pub author: RelationalLink<BlogDef, User>,
}
```

### Generated Methods

```rust
impl Post {
    // Get the link
    pub fn author(&self) -> &RelationalLink<BlogDef, User> {
        &self.author
    }

    // Hydrate if it's a reference
    pub fn author_hydrate<T>(&self, store: T)
        -> Result<Option<User>, NetabaseError>
    where
        T: StoreOps<BlogDef, User>,
    {
        self.author.clone().hydrate(&store)
    }

    // Insert if it's an entity
    pub fn insert_author_if_entity<S>(&self, store: &S)
        -> Result<(), NetabaseError>
    where
        S: OpenTree<BlogDef, User>,
    {
        if let RelationalLink::Entity(entity) = &self.author {
            let tree = store.open_tree();
            tree.put_raw(entity.clone())
        } else {
            Ok(())
        }
    }

    // Check if it's an entity
    pub fn author_is_entity(&self) -> bool {
        self.author.is_entity()
    }

    // Get the key
    pub fn author_key(&self) -> UserPrimaryKey {
        self.author.key()
    }
}
```

## Insertion Strategies

### Strategy 1: Insert Entities Manually

```rust
// Insert related entities first
if post.author_is_entity() {
    post.insert_author_if_entity(&store)?;
}

// Then insert the main entity
let tree = store.open_tree();
tree.put(post)?;
```

### Strategy 2: Batch Insertion

```rust
// Collect all entities to insert
let mut users_to_insert = Vec::new();

for post in posts {
    if let Some(user) = post.author.as_entity() {
        users_to_insert.push(user.clone());
    }
}

// Batch insert users
let user_tree = store.open_tree();
for user in users_to_insert {
    user_tree.put(user)?;
}

// Insert posts
let post_tree = store.open_tree();
for post in posts {
    post_tree.put(post)?;
}
```

### Strategy 3: Use References Only

```rust
// Insert entities independently
let user_tree = store.open_tree();
user_tree.put(user.clone())?;

// Create post with reference
let post = Post {
    id: 1,
    title: "Hello".to_string(),
    author: RelationalLink::Reference(user.primary_key()),
};

let post_tree = store.open_tree();
post_tree.put(post)?;
```

## Query Patterns

### Loading Related Entities

```rust
// Get post
let post_tree = store.open_tree::<Post>();
let post = post_tree.get(post_id)?.unwrap();

// Load author
let author = post.author_hydrate(&store)?.unwrap();
println!("Author: {}", author.name);
```

### Eager Loading Graph

```rust
// Load post with author embedded
fn load_post_with_author(
    store: &NetabaseStore<D, Backend>,
    post_id: u64,
) -> Result<Post, NetabaseError> {
    let post_tree = store.open_tree::<Post>();
    let mut post = post_tree.get(post_id)?.unwrap();

    // If author is a reference, load it
    if post.author.is_reference() {
        let author = post.author_hydrate(store)?.unwrap();
        post.author = RelationalLink::Entity(author);
    }

    Ok(post)
}
```

### Denormalization

```rust
// Convert references to entities for caching
fn denormalize_post(
    store: &NetabaseStore<D, Backend>,
    post: Post,
) -> Result<Post, NetabaseError> {
    let author = post.author_hydrate(store)?.unwrap();

    Ok(Post {
        author: RelationalLink::Entity(author),
        ..post
    })
}
```

## Best Practices

### 1. Choose the Right Strategy

**Use References when:**
- Building large graphs
- Entities change frequently
- Minimizing memory usage
- Serializing for network transfer

**Use Entities when:**
- Small, tightly coupled data
- Frequently accessed together
- Bundling for client delivery
- Avoiding lookup overhead

### 2. Avoid Circular Dependencies

```rust
// ❌ BAD: Circular embedding
pub struct User {
    pub posts: Vec<RelationalLink<Def, Post>>,  // Entity variants
}

pub struct Post {
    pub author: RelationalLink<Def, User>,  // Entity variant
}

// ✅ GOOD: Use references to break cycles
pub struct User {
    pub posts: Vec<RelationalLink<Def, Post>>,  // Reference variants
}

pub struct Post {
    pub author: RelationalLink<Def, User>,  // Can be Entity or Reference
}
```

### 3. Batch Operations

```rust
// ✅ GOOD: Batch related inserts
let users: Vec<User> = ...;
let user_tree = store.open_tree();

for user in users {
    user_tree.put(user)?;
}

// ❌ BAD: Open tree repeatedly
for user in users {
    let tree = store.open_tree();  // Don't do this in a loop!
    tree.put(user)?;
}
```

### 4. Use Metadata for Introspection

```rust
// Check if model has relations
if post.has_relations() {
    let relations = post.relations();
    for (_, rel) in relations {
        println!("Field: {} -> {}",
            rel.field_name(),
            rel.target_model_name());
    }
}
```

## Advanced Patterns

### Polymorphic Relationships

```rust
// Using enums for polymorphic links
#[derive(NetabaseModel, ...)]
pub struct Comment {
    pub id: u64,
    pub content: String,
    pub parent: CommentParent,
}

pub enum CommentParent {
    Post(RelationalLink<Def, Post>),
    Comment(RelationalLink<Def, Comment>),
}
```

### Soft Deletes with Relations

```rust
#[derive(NetabaseModel, ...)]
pub struct Post {
    pub id: u64,
    pub title: String,
    pub author: RelationalLink<Def, User>,
    pub deleted_at: Option<DateTime>,
}

// Query with cascade check
fn is_accessible(post: &Post, store: &Store) -> Result<bool, Error> {
    if post.deleted_at.is_some() {
        return Ok(false);
    }

    if let Some(author) = post.author_hydrate(store)? {
        Ok(author.deleted_at.is_none())
    } else {
        Ok(false)
    }
}
```

### Bi-directional Relationships

```rust
// Maintain consistency manually
fn link_user_and_post(
    store: &Store,
    user: &mut User,
    post: &mut Post,
) -> Result<(), Error> {
    // Set up bidirectional links
    post.author = RelationalLink::Reference(user.primary_key());
    user.posts.push(RelationalLink::Reference(post.primary_key()));

    // Save both
    let user_tree = store.open_tree();
    user_tree.put(user.clone())?;

    let post_tree = store.open_tree();
    post_tree.put(post.clone())?;

    Ok(())
}
```

### Caching Strategies

```rust
use std::collections::HashMap;

struct EntityCache<D, M> {
    cache: HashMap<M::PrimaryKey, M>,
    _phantom: PhantomData<D>,
}

impl<D, M> EntityCache<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
{
    fn hydrate_with_cache(
        &mut self,
        link: RelationalLink<D, M>,
        store: &Store,
    ) -> Result<M, Error> {
        match link {
            RelationalLink::Entity(e) => Ok(e),
            RelationalLink::Reference(key) => {
                if let Some(entity) = self.cache.get(&key) {
                    Ok(entity.clone())
                } else {
                    let entity = link.hydrate(store)?.unwrap();
                    self.cache.insert(key, entity.clone());
                    Ok(entity)
                }
            }
        }
    }
}
```

## Summary

Relational links provide a powerful, type-safe system for modeling relationships in NetabaseStore:

- ✅ Flexible loading strategies (eager vs lazy)
- ✅ Type-safe relationships
- ✅ Automatic helper method generation
- ✅ Runtime introspection
- ✅ Efficient serialization
- ✅ Works with all NetabaseStore backends

Start simple with single relationships, then expand to complex graphs as needed!
