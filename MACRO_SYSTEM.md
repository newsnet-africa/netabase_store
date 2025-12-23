# Netabase Macro System Documentation

## Overview

The Netabase macro system provides a declarative way to define database models, definitions, and global schemas using procedural macros. The system automatically generates all necessary boilerplate code including wrapper types, key enums, trait implementations, and serialization logic.

## Architecture

The macro system follows a **visitor/generator pattern** with three hierarchical levels:

### 1. Global Level - `#[netabase(GlobalName)]`
Wraps the root module and generates global enums that encapsulate all definitions.

### 2. Definition Level - `#[netabase_definition(Name, subscriptions(...))]`
Wraps definition modules and generates:
- Definition enum wrapping all models
- Subscription enums
- Tree names for database table management
- Trait implementations for NetabaseDefinition

### 3. Model Level - `#[derive(NetabaseModel)]`
Applied to struct models within definitions. Automatically generates:
- Wrapper types (ID types, field wrappers)
- Key enums (secondary, relational, subscription, blob)
- TreeName discriminants
- All necessary trait implementations

## Syntax Reference

### Basic Example

```rust
use netabase_macros::{NetabaseModel, netabase_definition};
use bincode::{Encode, Decode};
use serde::{Serialize, Deserialize};

#[netabase_definition(MyDef, subscriptions(TopicA, TopicB))]
pub mod my_definition {
    use super::*;

    #[derive(NetabaseModel)]
    #[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    #[subscribe(TopicA)]  // Optional: model subscribes to these topics
    pub struct User {
        #[primary_key]
        id: String,

        #[secondary_key]
        name: String,

        #[secondary_key]
        email: String,

        // Regular field
        bio: String,
    }

    #[derive(NetabaseModel)]
    #[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Post {
        #[primary_key]
        id: String,

        #[secondary_key]
        title: String,

        #[link(MyDef, User)]  // Relational link to User in same definition
        author: String,

        content: String,
    }
}
```

### Complete Example with All Features

```rust
#[netabase(GlobalSchema)]
pub mod root {
    use super::*;

    #[netabase_definition(MainDef, subscriptions(Created, Updated, Deleted))]
    pub mod main_definition {
        use super::*;

        // Define blob type before using it
        #[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
        pub struct ProfileData {
            pub data: Vec<u8>,
        }

        // Implement NetabaseBlobItem for the blob type
        impl netabase_store::blob::NetabaseBlobItem for ProfileData {
            type Blobs = Self;

            fn split_into_blobs(&self) -> Vec<Self::Blobs> {
                vec![self.clone()]
            }

            fn reconstruct_from_blobs(blobs: Vec<Self::Blobs>) -> Self {
                blobs.into_iter().next().unwrap_or_default()
            }

            fn wrap_blob(_index: u8, data: Vec<u8>) -> Self::Blobs {
                Self { data }
            }

            fn unwrap_blob(blob: &Self::Blobs) -> Option<(u8, Vec<u8>)> {
                Some((0, blob.data.clone()))
            }
        }

        #[derive(NetabaseModel)]
        #[derive(Debug, Clone, Encode, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[subscribe(Created, Updated)]  // Subscribes to these topics
        pub struct User {
            #[primary_key]
            id: String,

            #[secondary_key]
            username: String,

            #[secondary_key]
            email: String,

            #[link(MainDef, User)]  // Self-referential link
            friend: String,

            #[link(MainDef, Organization)]  // Link to another model
            org: String,

            #[blob]  // Large data stored separately
            profile: ProfileData,

            // Regular fields
            age: u8,
            active: bool,
        }

        #[derive(NetabaseModel)]
        #[derive(Debug, Clone, Encode, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct Organization {
            #[primary_key]
            id: String,

            #[secondary_key]
            name: String,

            description: String,
        }
    }
}
```

## Field Attributes

### `#[primary_key]`
- **Required**: Every model must have exactly one primary key
- **Effect**: Field type is wrapped in `<Model>ID` type
- **Example**: `id: String` becomes `id: UserID`

### `#[secondary_key]`
- **Optional**: Zero or more secondary keys for indexing
- **Effect**: Generates wrapper type and adds to `<Model>SecondaryKeys` enum
- **Example**: `name: String` generates `UserName(String)` wrapper

### `#[link(Definition, TargetModel)]`
- **Optional**: Creates a relational link to another model
- **Parameters**:
  - `Definition`: The definition containing the target model
  - `TargetModel`: The target model type
- **Effect**: Field type becomes `RelationalLink<'static, CurrentDef, TargetDef, TargetModel>`
- **Example**: `#[link(MyDef, User)] friend: String` becomes `friend: RelationalLink<'static, MyDef, MyDef, User>`

### `#[blob]`
- **Optional**: Marks field as large binary data
- **Requirement**: Field type must implement `NetabaseBlobItem`
- **Effect**: Data is split and stored separately in blob tables
- **Example**: `#[blob] data: LargeFile`

### `#[subscribe(...)]`
- **Optional**: Applied to the struct (not individual fields)
- **Parameters**: Topic names from the definition's subscription list
- **Effect**: Generates subscription key enum and adds subscription field to struct
- **Example**: `#[subscribe(Topic1, Topic2)]`

## Generated Code

For a model like:

```rust
#[derive(NetabaseModel)]
pub struct User {
    #[primary_key]
    id: String,

    #[secondary_key]
    name: String,

    #[link(MyDef, Post)]
    favorite_post: String,
}
```

The macro generates:

### Wrapper Types
```rust
pub struct UserID(pub String);
pub struct UserName(pub String);
pub struct UserFavoritePost(pub PostID);
```

### Key Enums
```rust
// TreeName discriminants (for database table names)
pub enum UserSecondaryKeysTreeName {
    Name
}

pub enum UserRelationalKeysTreeName {
    FavoritePost
}

// Key enums
pub enum UserSecondaryKeys {
    Name(UserName)
}

pub enum UserRelationalKeys {
    FavoritePost(UserFavoritePost)
}

pub enum UserSubscriptions {
    // Generated based on #[subscribe(...)]
}

pub enum UserBlobKeys {
    None  // No blobs in this model
}

pub struct UserBlobItem;  // Empty struct when no blobs

// Unified keys enum
pub enum UserKeys {
    Primary(UserID),
    Secondary(UserSecondaryKeys),
    Relational(UserRelationalKeys),
    Subscription(UserSubscriptions),
    Blob(UserBlobKeys),
}
```

### Trait Implementations
```rust
// NetabaseModel trait with TREE_NAMES constant
impl NetabaseModel<MyDef> for User { ... }

// NetabaseModelKeys trait
impl NetabaseModelKeys<MyDef, User> for UserKeys { ... }

// Store traits
impl StoreKey<MyDef, User> for UserID { ... }
impl StoreValue<MyDef, UserID> for User { ... }
impl StoreKeyMarker<MyDef> for UserID { ... }
impl StoreValueMarker<MyDef> for User { ... }

// Key type traits
impl NetabaseModelPrimaryKey for UserID { ... }
impl NetabaseModelSecondaryKey for UserSecondaryKeys { ... }
impl NetabaseModelRelationalKey for UserRelationalKeys { ... }
impl NetabaseModelBlobKey for UserBlobKeys { ... }

// Serialization
impl redb::Value for User { ... }
impl redb::Key for User { ... }
impl redb::Value for UserID { ... }
impl redb::Key for UserID { ... }
// ... and for all key enums

// Redb integration
impl RedbNetabaseModel for User { ... }
```

## Naming Conventions

The macro system uses **TreeName** (not Discriminant) for discriminant enums:

- `<Model><KeyType>TreeName` - Simple discriminant enums
  - Example: `UserSecondaryKeysTreeName`, `UserRelationalKeysTreeName`

- `<Definition>TreeName` - Simple discriminant for Definition enum
  - Example: `MyDefTreeName`

- `<Definition>TreeNames` - Complex enum containing ModelTreeNames
  - Example: `MyDefTreeNames`

## Nested Definitions

The system supports nested definitions for hierarchical permissions:

```rust
#[netabase_definition(ParentDef, subscriptions(Topic1))]
pub mod parent {
    use super::*;

    #[derive(NetabaseModel)]
    pub struct ParentModel {
        #[primary_key]
        id: String,
    }

    // Nested definition
    #[netabase_definition(ChildDef, subscriptions(Topic2))]
    pub mod child {
        use super::*;

        #[derive(NetabaseModel)]
        pub struct ChildModel {
            #[primary_key]
            id: String,

            // Can link to parent with explicit permission
            #[link(ParentDef, ParentModel)]
            parent: String,
        }
    }
}
```

**Permission Model**:
- Parents have full permissions to children
- Siblings/cousins need explicit linking for relational access

## Subscription System

Definitions declare available subscription topics:

```rust
#[netabase_definition(MyDef, subscriptions(Created, Updated, Deleted, Custom))]
```

Models subscribe to specific topics:

```rust
#[derive(NetabaseModel)]
#[subscribe(Created, Updated)]  // Subscribes to 2 of 4 available topics
pub struct User { ... }
```

The macro generates:
- `MyDefSubscriptions` enum with all topics
- `UserSubscriptions` enum with subscribed topics
- Subscription registry mapping topics to models
- Automatic `From` and `TryInto` conversions

## Error Handling

The macro system provides rich error messages for common issues:

- **Missing primary key**: "Model must have exactly one #[primary_key]"
- **Multiple primary keys**: "Only one field can be marked as primary key"
- **Invalid link target**: "Link target must reference a valid definition and model"
- **Unsupported subscription**: "Model subscribes to topic not defined in definition"
- **Duplicate field attributes**: "Field cannot have multiple key attributes"

## Limitations & Considerations

1. **Macro Determinism**: Macros are not deterministic, so code that depends on generated types from multiple macros should be generated by the nearest common parent macro.

2. **Blob Types**: Blob types must be defined before the model that uses them and must implement `NetabaseBlobItem`.

3. **Link Resolution**: Links use paths, so the target definition and model must be in scope.

4. **Derive Requirements**: Models must derive required traits:
   - `Clone`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`
   - `Encode`, `Decode` (bincode)
   - `Serialize`, `Deserialize` (serde)

5. **Field Types**: Primary key fields are type-wrapped. Ensure you're using the generated wrapper types when interacting with the API.

## Testing

All macro test files in the boilerplate library verify correct functionality:

- `test_simple_macro.rs` - Basic model with primary and secondary keys
- `macro_test_simple.rs` - Simple definition test
- `macro_test_def_simple.rs` - Definition-level features
- `macro_test.rs` - Complex test with blobs and links
- `macro_test_complete.rs` - Complete feature test

Run tests with:
```bash
cargo check --lib
cargo test
```

## Performance Considerations

The macro system:
- Generates code at compile time (zero runtime overhead)
- Creates type-safe wrappers preventing common errors
- Supports efficient database indexing through tree names
- Enables compile-time verification of links and subscriptions

## Migration from Boilerplate

To migrate existing boilerplate code to use macros:

1. Replace manual struct definitions with `#[derive(NetabaseModel)]`
2. Add field attributes (`#[primary_key]`, `#[secondary_key]`, etc.)
3. Wrap models in `#[netabase_definition(...)]` module
4. Optionally add `#[netabase(...)]` global wrapper
5. Remove manual implementations (the macro generates them)

The generated code structure matches the boilerplate exactly, ensuring compatibility.
