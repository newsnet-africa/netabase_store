# Netabase Store Examples

Examples, tests, and benchmarks for the `netabase_store` embedded database library.

## 📚 Documentation

- **[GUIDE.md](./GUIDE.md)** - Comprehensive beginner's guide with step-by-step examples
- **[Parent README](../README.md)** - Main library documentation

## 🚀 Quick Start

```bash
# Run the demonstration
cargo run -p netabase_store_examples

# Run all tests
cargo test -p netabase_store_examples

# Run benchmarks
cargo bench -p netabase_store_examples

# Clean up benchmark artifacts (recommended after benchmarks)
rm -rf boilerplate/tmp/
```

> **Note:** Benchmarks create temporary database files in `./tmp/`. These are automatically cleaned up after each benchmark run via `CleanupGuard`, but if benchmarks are interrupted, you may need to manually clean with `rm -rf ./tmp/`.

## 📁 Project Structure

```
boilerplate/
├── src/
│   ├── main.rs                    # Feature demonstration program
│   ├── lib.rs                     # Library exports
│   └── boilerplate_lib/
│       ├── mod.rs                 # Main model definitions (User, Post, Category)
│       ├── repository_example.rs  # Advanced repository pattern
│       └── simple_repo_example.rs # Simplified repository example
├── tests/
│   ├── schema_export.rs           # Schema serialization tests
│   ├── schema_import.rs           # Schema import tests
│   ├── migration_logic.rs         # Model migration tests
│   └── macro_test.rs              # Macro expansion tests
├── benches/
│   ├── crud.rs                    # CRUD performance benchmarks
│   ├── stress.rs                  # High-load stress tests
│   └── record_store.rs            # Record storage benchmarks
└── GUIDE.md                       # Beginner's guide (start here!)
```

## 🎯 What's Demonstrated

### Core Features
- ✅ **Type-safe models** with compile-time validation
- ✅ **Primary and secondary keys** for fast lookups
- ✅ **Relational links** between models
- ✅ **Blob storage** for large binary data (auto-chunked)
- ✅ **Schema versioning** with automatic migration
- ✅ **Repository pattern** for access control

### Models Included

#### Definition (Main Schema)
- **User** (versioned: V1 → V2)
  - Primary key: `id`
  - Secondary keys: `first_name`, `last_name`, `age`
  - Links: `partner` (self-reference), `category` (cross-definition)
  - Blobs: `bio`, `another`
  - Subscriptions: `Topic1`, `Topic2`

- **Post** (versioned: V1 → V2)
  - Primary key: `id`
  - Secondary keys: `title`, `author_id`
  - Subscriptions: `Topic3`, `Topic4`

- **HeavyModel** (for stress testing)
  - Multiple secondary keys and links
  - Large blob attachment
  - All topic subscriptions

#### DefinitionTwo (Secondary Schema)
- **Category**
  - Primary key: `id`
  - Secondary key: `name`
  - Subscription: `General`

### Repository Examples

1. **MainRepository**: Combines Definition + DefinitionTwo
2. **EmployeeRepo**: Demonstrates bounded access patterns
3. **SimpleRepo**: Minimal repository setup

## 💡 Usage Examples

### Basic CRUD

```rust
use netabase_store_examples::*;
use netabase_store::prelude::*;
use netabase_store::traits::database::store::NBStore;

// Create store
let (store, _temp) = RedbStore::<Definition>::new_temporary()?;

// Write
let txn = store.begin_write()?;
txn.create(&User {
    id: UserID("alice".into()),
    first_name: "Alice".into(),
    last_name: "Smith".into(),
    age: 30,
    // ... other fields
})?;
txn.commit()?;

// Read
let txn = store.begin_read()?;
let user: Option<User> = txn.read(&UserID("alice".into()))?;
```

### Relationships

```rust
use netabase_store::relational::RelationalLink;

let post = Post {
    id: PostID("post1".into()),
    title: "Hello World".into(),
    author_id: "alice".into(),
    // ...
};

// The author_id can be used to look up the User
let author: Option<User> = txn.read(&UserID(post.author_id.clone()))?;
```

### Schema Migration

```rust
use netabase_store::traits::migration::MigrateFrom;

// Old data (UserV1) is automatically migrated to User (V2)
let old = UserV1 { id: ..., name: "Alice Smith", ... };
let new = User::migrate_from(old);
// new.first_name == "Alice", new.last_name == "Smith"
```

## 🧪 Testing

```bash
# All tests
cargo test -p netabase_store_examples

# Schema export (must run before import)
cargo test -p netabase_store_examples --test 0_schema_export

# Schema import
cargo test -p netabase_store_examples --test 1_schema_import

# Migration logic
cargo test -p netabase_store_examples --test migration_logic
```

## 📊 Benchmarks

```bash
# CRUD operations benchmark
cargo bench -p netabase_store_examples --bench crud

# Stress testing (1000+ records)
cargo bench -p netabase_store_examples --bench stress

# Record store performance
cargo bench -p netabase_store_examples --bench record_store
```

## 📖 Learning Path

1. **Start with [GUIDE.md](./GUIDE.md)** - Complete beginner's guide
2. **Read `src/main.rs`** - See all features in action
3. **Explore `src/boilerplate_lib/mod.rs`** - Understand model definitions
4. **Review tests** - Integration examples
5. **Check benchmarks** - Performance characteristics

## 🔧 Development

This crate uses Rust edition 2024 and requires:
- `serde` for serialization
- `postcard` for binary encoding
- `redb` as the underlying database
- `netabase_macros` for code generation

## 📝 License

Same as parent crate.
