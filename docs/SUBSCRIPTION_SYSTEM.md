# Subscription System for Netabase Store

## Overview

The subscription system in netabase_store provides a powerful mechanism for tracking data changes and synchronizing between different nodes using merkle trees. This system allows you to organize your data into topics, efficiently detect differences between nodes, and coordinate data synchronization.

## Core Concepts

### 1. Subscription Topics

Topics are logical groupings of data that you want to track and synchronize. They're defined using the `#[streams(...)]` attribute:

```rust
#[netabase_definition_module(BlogDefinition, BlogKeys)]
#[streams(Users, Posts, Comments, Tags)]
mod blog {
    // Your models here
}
```

This generates:
- `BlogDefinitionSubscriptions` enum with variants for each topic
- `BlogDefinitionSubscriptionManager` for managing all subscription trees
- Individual subscription tree types for each topic

### 2. Subscription Trees

Each topic gets its own subscription tree that tracks:
- **Keys**: Unique identifiers for data items
- **Hashes**: Content hashes using BLAKE3 for integrity
- **Merkle Tree**: Hierarchical hash structure for efficient comparison

### 3. ModelHash

A cryptographically secure hash of model data:

```rust
// Automatically generated for each model
let hash = ModelHash::from_data(&serialized_model);
let combined_hash = ModelHash::from_key_and_data(&key, &data);
```

## Generated Code Structure

When you use `#[streams(Topic1, Topic2, Topic3)]`, the macro generates:

### Subscription Enum
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum BlogDefinitionSubscriptions {
    Users,
    Posts,
    Comments,
}

impl BlogDefinitionSubscriptions {
    pub fn as_str(&self) -> &'static str { /* ... */ }
    pub fn all_variants() -> Vec<Self> { /* ... */ }
}
```

### Individual Tree Types
```rust
pub struct UsersSubscriptionTree {
    inner: MerkleSubscriptionTree<BlogDefinition>,
}

pub struct PostsSubscriptionTree {
    inner: MerkleSubscriptionTree<BlogDefinition>,
}

// etc.
```

### Subscription Manager
```rust
pub struct BlogDefinitionSubscriptionManager {
    pub users_tree: UsersSubscriptionTree,
    pub posts_tree: PostsSubscriptionTree,
    pub comments_tree: CommentsSubscriptionTree,
}

impl BlogDefinitionSubscriptionManager {
    pub fn subscribe_item<T>(&mut self, topic: BlogDefinitionSubscriptions, key: Vec<u8>, data: &T) -> Result<(), NetabaseError>;
    pub fn compare_with(&mut self, other: &mut Self) -> Result<HashMap<BlogDefinitionSubscriptions, SubscriptionDiff<BlogDefinition>>, NetabaseError>;
    // ... other methods
}
```

### Synchronization Helpers
```rust
pub struct BlogDefinitionSyncHelper;

impl BlogDefinitionSyncHelper {
    pub fn create_sync_plan(local: &mut BlogDefinitionSubscriptionManager, remote: &mut BlogDefinitionSubscriptionManager) -> Result<SyncPlan<BlogDefinitionSubscriptions>, NetabaseError>;
    pub fn compare_roots(local: &mut BlogDefinitionSubscriptionManager, remote: &mut BlogDefinitionSubscriptionManager) -> Result<HashMap<BlogDefinitionSubscriptions, (Option<[u8; 32]>, Option<[u8; 32]>)>, NetabaseError>;
}
```

## Usage Examples

### Basic Setup

```rust
use netabase_store::{netabase_definition_module, NetabaseModel, netabase};

#[netabase_definition_module(MyAppDefinition, MyAppKeys)]
#[streams(Users, Posts, Comments)]
mod my_app {
    use super::*;
    
    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode)]
    #[netabase(MyAppDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
    }
    
    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode)]
    #[netabase(MyAppDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub author_id: u64,
    }
}

use my_app::*;
```

### Working with Subscription Manager

```rust
// Create a subscription manager
let mut manager = MyAppDefinitionSubscriptionManager::new();

// Add data to subscription tracking
let user = User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() };
let user_data = bincode::encode_to_vec(&user, bincode::config::standard())?;
let user_key = bincode::encode_to_vec(&user.primary_key(), bincode::config::standard())?;

manager.subscribe_item(MyAppDefinitionSubscriptions::Users, user_key, &user_data)?;

// Get statistics
let stats = manager.stats();
println!("Total items: {}", stats.total_items);
println!("Active topics: {}", stats.active_topics);
```

### Comparing and Synchronizing

```rust
// Create two managers (representing different nodes)
let mut node_a = MyAppDefinitionSubscriptionManager::new();
let mut node_b = MyAppDefinitionSubscriptionManager::new();

// ... add different data to each ...

// Compare the managers
let diffs = node_a.compare_with(&mut node_b)?;

for (topic, diff) in &diffs {
    println!("Topic {:?} has {} differences", topic, diff.total_differences());
    
    // Keys that node_a is missing
    for key in diff.keys_needed_by_self() {
        println!("Node A needs key: {:?}", key);
    }
    
    // Keys that node_b is missing
    for key in diff.keys_needed_by_other() {
        println!("Node B needs key: {:?}", key);
    }
    
    // Keys with conflicting values
    for key in diff.conflicting_keys() {
        println!("Conflict at key: {:?}", key);
    }
}
```

### Creating Sync Plans

```rust
// Generate a synchronization plan
let sync_plan = MyAppDefinitionSyncHelper::create_sync_plan(&mut node_a, &mut node_b)?;

println!("Total operations needed: {}", sync_plan.total_operations());

// Handle downloads (what node_a needs from node_b)
for (topic, keys) in &sync_plan.downloads {
    for key in keys {
        // Fetch data for this key from node_b and apply to node_a
        if let Some(data) = get_data_from_node_b(topic, key) {
            node_a.subscribe_item(topic, key.clone(), &data)?;
        }
    }
}

// Handle uploads (what node_b needs from node_a)
for (topic, keys) in &sync_plan.uploads {
    for key in keys {
        // Send data from node_a to node_b
        if let Some(data) = get_data_from_node_a(topic, key) {
            send_data_to_node_b(topic, key, &data);
        }
    }
}
```

## Database Integration

The subscription system integrates with all supported database backends:

### Memory Store
```rust
let store = MemoryStore::<MyAppDefinition>::with_subscriptions();
// Subscriptions are automatically tracked when you modify data
```

### Sled Store
```rust
let store = SledStore::<MyAppDefinition>::new("./database")?;
store.enable_subscriptions()?;
```

### Redb Store
```rust
let store = RedbStore::<MyAppDefinition>::new("./database.redb")?;
store.enable_subscriptions()?;
```

## Advanced Features

### Custom Subscription Filters

You can customize which data gets included in which topics:

```rust
pub struct CustomFilter;

impl SubscriptionFilter<MyAppDefinition> for CustomFilter {
    fn should_include<T>(&self, topic: MyAppDefinitionSubscriptions, key: &[u8], data: &T) -> bool
    where
        T: AsRef<[u8]>,
    {
        match topic {
            MyAppDefinitionSubscriptions::Users => {
                // Include all users
                true
            }
            MyAppDefinitionSubscriptions::Posts => {
                // Only include posts from specific authors
                // (You'd decode the data and check author_id here)
                true
            }
            MyAppDefinitionSubscriptions::Comments => {
                // Include all comments
                true
            }
        }
    }
    
    fn applicable_topics<T>(&self, key: &[u8], data: &T) -> Vec<MyAppDefinitionSubscriptions>
    where
        T: AsRef<[u8]>,
    {
        // Determine which topics should include this data
        vec![MyAppDefinitionSubscriptions::Users] // Example
    }
}
```

### Export/Import Functionality

```rust
// Export subscription data
let exported_data = manager.export_data();

// Import to another manager
let mut new_manager = MyAppDefinitionSubscriptionManager::new();
new_manager.import_data(exported_data)?;
```

## Performance Considerations

### Merkle Tree Efficiency
- **O(log n)** comparison complexity for detecting differences
- **O(n)** rebuild cost when data changes significantly
- Automatic incremental updates for small changes

### Memory Usage
- Each subscription tree stores: `BTreeMap<Vec<u8>, ModelHash>`
- ModelHash is 32 bytes (BLAKE3 hash)
- Merkle tree nodes are computed on-demand

### Network Optimization
- Only merkle roots need to be exchanged for initial comparison
- Detailed diffs are computed only when differences are detected
- Sync plans minimize data transfer by identifying exact missing items

## Error Handling

The subscription system uses `NetabaseError` for all error cases:

```rust
match manager.subscribe_item(topic, key, data) {
    Ok(()) => println!("Successfully subscribed item"),
    Err(NetabaseError::Storage(msg)) => eprintln!("Storage error: {}", msg),
    Err(NetabaseError::Serialization(msg)) => eprintln!("Serialization error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Future Enhancements

### Planned Features
1. **Network Protocols**: Built-in support for libp2p-based synchronization
2. **Conflict Resolution**: Automatic and manual conflict resolution strategies
3. **Partial Sync**: Ability to sync only specific topics or key ranges
4. **Compression**: Optional compression for large subscription data
5. **Persistence**: Save/restore subscription state across restarts

### Extension Points
- Custom hash algorithms (beyond BLAKE3)
- Pluggable merkle tree implementations
- Custom synchronization protocols
- Topic-specific retention policies

## Best Practices

1. **Topic Design**: Group related data that should sync together
2. **Key Management**: Use consistent, deterministic key generation
3. **Batch Operations**: Use bulk subscription operations for better performance
4. **Error Handling**: Always handle subscription errors gracefully
5. **Resource Management**: Clear unused subscription data periodically
6. **Testing**: Test synchronization scenarios with mock data

## Troubleshooting

### Common Issues

**Merkle roots don't match despite same data**
- Ensure key serialization is deterministic
- Check that data is hashed consistently
- Verify topic assignments are identical

**High memory usage**
- Consider clearing unused subscription trees
- Use subscription filters to reduce tracked data
- Implement periodic cleanup of old entries

**Slow synchronization**
- Profile merkle tree rebuild operations
- Consider batching subscription updates
- Use partial sync for large datasets

**Network errors during sync**
- Implement retry logic with exponential backoff
- Use checksums to verify data integrity
- Handle partial transfer scenarios