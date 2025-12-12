# Getting Started with NetabaseStore

This guide will walk you through creating your first NetabaseStore application, from basic setup to advanced features.

## 📋 Prerequisites

- Rust 1.70+ installed
- Basic familiarity with Rust syntax
- Understanding of serialization/deserialization concepts

## 🚀 Installation

Add NetabaseStore to your `Cargo.toml`:

```toml
[dependencies]
netabase_store = "0.1.0"
netabase_macros = "0.1.0"
serde = { version = "1.0", features = ["derive"] }

# Optional: for persistent storage
sled = { version = "0.34", optional = true }
redis = { version = "0.24", optional = true }
```

## 📖 Step 1: Your First Model

Let's create a simple user management system:

```rust
use netabase_store::prelude::*;
use netabase_macros::netabase;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[netabase]
struct User {
    id: u64,
    username: String,
    email: String,
    created_at: u64,
    is_active: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The #[netabase] macro generates a complete UserStore for us
    let mut user_store = UserStore::new_in_memory()?;
    
    // Create a new user
    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        created_at: 1639123200, // Unix timestamp
        is_active: true,
    };
    
    // Store the user
    user_store.insert(1, alice.clone())?;
    
    // Retrieve the user
    if let Some(retrieved_user) = user_store.get(&1)? {
        println!("Found user: {:?}", retrieved_user);
        assert_eq!(alice, retrieved_user);
    }
    
    // List all users
    let all_users: Vec<User> = user_store.scan()?.collect();
    println!("Total users: {}", all_users.len());
    
    Ok(())
}
```

## 📦 Step 2: Working with Different Backends

### In-Memory Storage (Default)
Perfect for development, testing, and caching:

```rust
let store = UserStore::new_in_memory()?;
```

### Persistent Storage with Sled
Great for local applications and embedded systems:

```rust
// Add to Cargo.toml: sled = "0.34"
let store = UserStore::new_sled("./my_database")?;
```

### Redis Integration
Ideal for distributed applications and caching:

```rust
// Add to Cargo.toml: redis = "0.24"
let store = UserStore::new_redis("redis://127.0.0.1/")?;
```

## 🔍 Step 3: Querying and Filtering

```rust
use netabase_store::prelude::*;

fn example_queries() -> Result<(), Box<dyn std::error::Error>> {
    let mut user_store = UserStore::new_in_memory()?;
    
    // Insert some test data
    for i in 1..=5 {
        let user = User {
            id: i,
            username: format!("user{}", i),
            email: format!("user{}@example.com", i),
            created_at: 1639123200 + i * 3600,
            is_active: i % 2 == 0, // Even IDs are active
        };
        user_store.insert(i, user)?;
    }
    
    // Get specific user
    if let Some(user) = user_store.get(&3)? {
        println!("User 3: {}", user.username);
    }
    
    // Check if user exists
    if user_store.contains_key(&1)? {
        println!("User 1 exists!");
    }
    
    // Scan all users
    let all_users: Vec<User> = user_store.scan()?.collect();
    println!("Found {} users", all_users.len());
    
    // Filter active users (custom logic)
    let active_users: Vec<User> = user_store
        .scan()?
        .filter(|user| user.is_active)
        .collect();
    println!("Active users: {}", active_users.len());
    
    // Update user
    if let Some(mut user) = user_store.get(&2)? {
        user.email = "newemail@example.com".to_string();
        user_store.insert(2, user)?;
    }
    
    // Delete user
    user_store.remove(&5)?;
    
    Ok(())
}
```

## 📊 Step 4: Working with Enums

NetabaseStore handles enums seamlessly:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum UserRole {
    Admin,
    Moderator,
    User,
    Guest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum UserStatus {
    Active,
    Inactive,
    Suspended { reason: String, until: Option<u64> },
    Banned { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[netabase]
struct User {
    id: u64,
    username: String,
    role: UserRole,
    status: UserStatus,
}

fn enum_examples() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = UserStore::new_in_memory()?;
    
    let admin = User {
        id: 1,
        username: "admin".to_string(),
        role: UserRole::Admin,
        status: UserStatus::Active,
    };
    
    let suspended_user = User {
        id: 2,
        username: "troublemaker".to_string(),
        role: UserRole::User,
        status: UserStatus::Suspended {
            reason: "Spam posting".to_string(),
            until: Some(1640000000),
        },
    };
    
    store.insert(1, admin)?;
    store.insert(2, suspended_user)?;
    
    // Filter by role
    let admins: Vec<User> = store
        .scan()?
        .filter(|user| matches!(user.role, UserRole::Admin))
        .collect();
    
    println!("Found {} admins", admins.len());
    
    Ok(())
}
```

## 🏗️ Step 5: Complex Data Structures

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct UserProfile {
    bio: String,
    avatar_url: Option<String>,
    social_links: HashMap<String, String>,
    preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct UserPreferences {
    theme: String,
    notifications_enabled: bool,
    language: String,
    timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[netabase]
struct User {
    id: u64,
    username: String,
    email: String,
    profile: UserProfile,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
}

fn complex_data_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = UserStore::new_in_memory()?;
    
    let user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        profile: UserProfile {
            bio: "Software engineer passionate about Rust".to_string(),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            social_links: {
                let mut links = HashMap::new();
                links.insert("github".to_string(), "https://github.com/alice".to_string());
                links.insert("twitter".to_string(), "https://twitter.com/alice".to_string());
                links
            },
            preferences: UserPreferences {
                theme: "dark".to_string(),
                notifications_enabled: true,
                language: "en".to_string(),
                timezone: "UTC".to_string(),
            },
        },
        tags: vec!["developer".to_string(), "rust".to_string(), "open-source".to_string()],
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("last_login".to_string(), "2024-01-01".to_string());
            meta.insert("login_count".to_string(), "42".to_string());
            meta
        },
    };
    
    store.insert(1, user.clone())?;
    
    // Retrieve and verify complex data
    if let Some(retrieved) = store.get(&1)? {
        println!("User bio: {}", retrieved.profile.bio);
        println!("Theme preference: {}", retrieved.profile.preferences.theme);
        println!("Tags: {:?}", retrieved.tags);
        
        if let Some(github) = retrieved.profile.social_links.get("github") {
            println!("GitHub profile: {}", github);
        }
    }
    
    Ok(())
}
```

## 🎯 Step 6: Error Handling

NetabaseStore provides comprehensive error handling:

```rust
use netabase_store::{NetabaseError, NetabaseResult};

fn error_handling_example() -> NetabaseResult<()> {
    let mut store = UserStore::new_in_memory()?;
    
    // Handle potential errors gracefully
    match store.get(&999) {
        Ok(Some(user)) => println!("Found user: {:?}", user),
        Ok(None) => println!("User not found"),
        Err(NetabaseError::StoreNotLoaded) => {
            println!("Store is not properly initialized");
        }
        Err(NetabaseError::SerializationFailed(msg)) => {
            println!("Serialization error: {}", msg);
        }
        Err(e) => println!("Other error: {:?}", e),
    }
    
    // Using the ? operator for error propagation
    let user = store.get(&1)?.ok_or(NetabaseError::StoreNotLoaded)?;
    println!("Successfully retrieved user: {}", user.username);
    
    Ok(())
}
```

## 🔧 Step 7: Custom Validation

You can add custom validation logic:

```rust
impl User {
    /// Validates the user data before storing
    pub fn validate(&self) -> Result<(), String> {
        if self.username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        
        if self.username.len() < 3 {
            return Err("Username must be at least 3 characters".to_string());
        }
        
        if !self.email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        
        Ok(())
    }
}

fn validation_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = UserStore::new_in_memory()?;
    
    let user = User {
        id: 1,
        username: "al".to_string(), // Too short!
        email: "invalid-email".to_string(), // Invalid format!
        // ... other fields
    };
    
    // Validate before storing
    match user.validate() {
        Ok(()) => {
            store.insert(user.id, user)?;
            println!("User stored successfully!");
        }
        Err(validation_error) => {
            println!("Validation failed: {}", validation_error);
        }
    }
    
    Ok(())
}
```

## 🚦 Next Steps

Congratulations! You've learned the basics of NetabaseStore. Here's what to explore next:

1. **[Architecture Guide](architecture.md)** - Understand the internal design
2. **[Permission System](permissions.md)** - Learn about hierarchical permissions
3. **[Cross-Definition Linking](cross_definitions.md)** - Connect different data types
4. **[Examples](../examples/)** - Real-world application examples
5. **[Performance Guide](performance.md)** - Optimization techniques

## 💡 Tips for Success

1. **Start Simple**: Begin with basic CRUD operations before adding complexity
2. **Use Type Safety**: Let the Rust compiler catch errors at compile time
3. **Choose the Right Backend**: In-memory for development, Sled for local apps, Redis for distributed systems
4. **Validate Early**: Add validation logic to catch data issues early
5. **Read the Examples**: The examples directory contains complete, working applications

## ❓ Common Questions

**Q: Can I use custom types as keys?**
A: Yes, as long as they implement the required traits (Hash, Eq, etc.)

**Q: How do I handle schema migrations?**
A: NetabaseStore focuses on type safety. For migrations, handle them at the application level.

**Q: Can I use async/await?**
A: The current version is synchronous, but async support is planned for future releases.

**Q: Is NetabaseStore production-ready?**
A: Yes, it's designed for production use with comprehensive testing and error handling.

---

**Ready to build more complex applications? Check out our [E-commerce Example](../examples/ecommerce.rs) for a complete real-world application!**