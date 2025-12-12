# NetabaseStore Architecture

This document provides a deep dive into NetabaseStore's architecture, design decisions, and internal workings.

## 🏗️ High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Application Layer                            │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐   │
│  │  User Models    │ │ Business Logic  │ │   API Routes    │   │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                  Generated Code Layer                           │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐   │
│  │  Store Types    │ │  Validators     │ │  Permissions    │   │
│  │  (UserStore)    │ │  (Type Safety)  │ │  (Hierarchical) │   │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                   NetabaseStore Core                           │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐   │
│  │ Permission      │ │ Cross-Definition│ │  Error          │   │
│  │ Manager         │ │ Linker          │ │  Handling       │   │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘   │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐   │
│  │ Serialization   │ │ Transaction     │ │  Caching        │   │
│  │ Engine          │ │ Manager         │ │  System         │   │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                    Backend Abstraction Layer                   │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐   │
│  │  Backend Trait  │ │  Key-Value API  │ │  Batch Ops      │   │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                      Storage Backends                          │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐   │
│  │   In-Memory     │ │      Sled       │ │      Redis      │   │
│  │   HashMap       │ │   Embedded DB   │ │  Distributed    │   │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## 🧠 Core Design Principles

### 1. **Type Safety First**
Every operation is validated at compile time to prevent runtime errors:

```rust
// This will fail at compile time if Product doesn't exist
order_store.insert_with_cross_validation(&product_store, order_id, order)?;

// This is impossible to write incorrectly
let user: User = user_store.get(&user_id)?.unwrap();
```

### 2. **Zero-Cost Abstractions**
All validation and type checking happens at compile time:

```rust
// The macro generates code that compiles to optimal machine code
#[netabase(cross_links = ["Product"])]
struct Order {
    product_ids: Vec<u64>,  // Validated against Product store at compile time
}
```

### 3. **Backend Agnostic**
The same code works with any storage backend:

```rust
// These all provide the same API
let store1 = UserStore::new_in_memory()?;
let store2 = UserStore::new_sled("./db")?;
let store3 = UserStore::new_redis("redis://localhost")?;
```

### 4. **Hierarchical Permissions**
Parent-child relationships are enforced at both compile time and runtime:

```rust
#[netabase(parent = "Organization")]
struct Department {
    org_id: u64,  // Must reference valid Organization
}
```

## 🔧 Component Deep Dive

### Macro System (`netabase_macros`)

The macro system is the heart of NetabaseStore, generating type-safe code at compile time:

```rust
#[proc_macro_derive(netabase, attributes(cross_links, parent, permissions))]
pub fn netabase_derive(input: TokenStream) -> TokenStream {
    // Parse the input struct
    let input = parse_macro_input!(input as DeriveInput);
    
    // Extract netabase attributes
    let attrs = extract_netabase_attributes(&input.attrs);
    
    // Generate store implementation
    let store_impl = generate_store_implementation(&input, &attrs);
    
    // Generate permission management
    let permission_impl = generate_permission_system(&input, &attrs);
    
    // Generate cross-definition validation
    let validation_impl = generate_cross_validation(&input, &attrs);
    
    // Combine all generated code
    quote! {
        #store_impl
        #permission_impl
        #validation_impl
    }.into()
}
```

#### Generated Store Structure

For each `#[netabase]` struct, the macro generates:

```rust
// For: #[netabase] struct User { ... }

pub struct UserStore<B: Backend> {
    backend: B,
    permission_manager: PermissionManager<User>,
    cross_definition_links: CrossDefinitionLinks,
}

impl<B: Backend> UserStore<B> {
    // Standard CRUD operations
    pub fn insert(&mut self, key: u64, value: User) -> NetabaseResult<()> { ... }
    pub fn get(&self, key: &u64) -> NetabaseResult<Option<User>> { ... }
    pub fn remove(&mut self, key: &u64) -> NetabaseResult<Option<User>> { ... }
    pub fn scan(&self) -> NetabaseResult<impl Iterator<Item = User>> { ... }
    
    // Permission-aware operations
    pub fn insert_with_permission_check(&mut self, ...) -> NetabaseResult<()> { ... }
    
    // Cross-definition operations
    pub fn insert_with_cross_validation(&mut self, ...) -> NetabaseResult<()> { ... }
}
```

### Permission System

The permission system operates on multiple levels:

#### 1. **Compile-Time Validation**
```rust
// This generates compile-time checks
#[netabase(parent = "Organization", permission_level = "department")]
struct Department {
    org_id: u64,  // Compiler ensures this field exists and is the right type
}
```

#### 2. **Runtime Permission Checks**
```rust
pub struct PermissionManager<T> {
    permission_tree: PermissionTree,
    access_control: AccessControlList,
    validation_cache: HashMap<String, bool>,
}

impl<T> PermissionManager<T> {
    pub fn check_access(&self, operation: Operation, context: &PermissionContext) -> bool {
        // Check cached permissions first
        if let Some(cached) = self.validation_cache.get(&context.cache_key()) {
            return *cached;
        }
        
        // Walk the permission tree
        let result = self.permission_tree.validate_access(operation, context);
        
        // Cache the result
        self.validation_cache.insert(context.cache_key(), result);
        
        result
    }
}
```

#### 3. **Hierarchical Structure**
```rust
pub struct PermissionTree {
    nodes: HashMap<PermissionLevel, PermissionNode>,
    relationships: Vec<PermissionRelationship>,
}

pub struct PermissionNode {
    level: PermissionLevel,
    parent: Option<PermissionLevel>,
    children: Vec<PermissionLevel>,
    permissions: Vec<Permission>,
}
```

### Cross-Definition Linking

Cross-definition linking ensures referential integrity across different stores:

#### 1. **Relationship Types**
```rust
#[derive(Debug, Clone)]
pub enum RelationshipType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

#[derive(Debug, Clone)]
pub struct CrossDefinitionLink {
    source_type: String,
    target_type: String,
    relationship: RelationshipType,
    validation_rules: Vec<ValidationRule>,
}
```

#### 2. **Validation Engine**
```rust
pub struct CrossDefinitionValidator {
    links: HashMap<String, Vec<CrossDefinitionLink>>,
    validation_cache: LruCache<String, ValidationResult>,
}

impl CrossDefinitionValidator {
    pub fn validate_cross_reference<T, U>(
        &mut self,
        source: &T,
        target_store: &dyn Backend,
        target_key: &str,
    ) -> NetabaseResult<()> {
        // Check if validation is cached
        let cache_key = format!("{}:{}:{}", 
            type_name::<T>(), target_key, hash(source));
        
        if let Some(cached) = self.validation_cache.get(&cache_key) {
            return cached.clone();
        }
        
        // Perform validation
        let result = self.perform_validation(source, target_store, target_key);
        
        // Cache result
        self.validation_cache.put(cache_key, result.clone());
        
        result
    }
}
```

### Backend Abstraction

The backend system provides a unified interface across different storage engines:

#### 1. **Backend Trait**
```rust
pub trait Backend: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    fn remove(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn scan(&self) -> Result<Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>>, Self::Error>;
    fn contains_key(&self, key: &[u8]) -> Result<bool, Self::Error>;
    
    // Batch operations
    fn batch_insert(&mut self, items: &[(&[u8], &[u8])]) -> Result<(), Self::Error>;
    fn batch_remove(&mut self, keys: &[&[u8]]) -> Result<(), Self::Error>;
}
```

#### 2. **Concrete Implementations**

**In-Memory Backend**
```rust
pub struct InMemoryBackend {
    data: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl Backend for InMemoryBackend {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        let data = self.data.read().unwrap();
        Ok(data.get(key).cloned())
    }
    // ... other methods
}
```

**Sled Backend**
```rust
pub struct SledBackend {
    db: sled::Db,
}

impl Backend for SledBackend {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.db.get(key)?.map(|v| v.to_vec()))
    }
    // ... other methods
}
```

**Redis Backend**
```rust
pub struct RedisBackend {
    connection: redis::Connection,
}

impl Backend for RedisBackend {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        let key_str = String::from_utf8_lossy(key);
        let result: Option<Vec<u8>> = redis::cmd("GET")
            .arg(key_str.as_ref())
            .query(&mut self.connection)?;
        Ok(result)
    }
    // ... other methods
}
```

## 🔄 Data Flow

### 1. **Compilation Phase**
```
User Code → Macro Processing → Generated Code → Rust Compiler → Optimized Binary
```

1. User writes `#[netabase]` struct
2. Macro processes attributes and generates store code
3. Rust compiler optimizes the generated code
4. Result: Zero-overhead runtime performance

### 2. **Runtime Phase**
```
Application → Store API → Permission Check → Cross-Validation → Backend → Storage
```

1. Application calls store method
2. Permission manager validates access
3. Cross-definition validator checks references
4. Backend performs storage operation
5. Result returned to application

## 🚀 Performance Characteristics

### Compile-Time Optimizations
- **Monomorphization**: Generic code specialized for each type
- **Dead Code Elimination**: Unused generated code is removed
- **Inlining**: Small functions are inlined for zero overhead

### Runtime Optimizations
- **Permission Caching**: Frequently checked permissions are cached
- **Validation Caching**: Cross-definition checks are cached
- **Batch Operations**: Multiple operations grouped for efficiency
- **Lock-Free Reads**: Read operations avoid locks where possible

### Memory Usage
```rust
// Typical memory layout for UserStore
struct UserStore<InMemoryBackend> {
    backend: InMemoryBackend,                    // 24 bytes
    permission_manager: PermissionManager<User>, // 64 bytes
    cross_definition_links: CrossDefinitionLinks // 32 bytes
}
// Total: ~120 bytes + data
```

## 🔧 Extension Points

### Custom Backends
Implement the `Backend` trait for custom storage solutions:

```rust
pub struct CustomBackend {
    // Your implementation
}

impl Backend for CustomBackend {
    type Error = CustomError;
    
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        // Your implementation
    }
    // ... implement other methods
}
```

### Custom Permissions
Extend the permission system with custom logic:

```rust
#[derive(Debug)]
pub struct CustomPermission {
    name: String,
    validator: Box<dyn Fn(&PermissionContext) -> bool>,
}

impl Permission for CustomPermission {
    fn validate(&self, context: &PermissionContext) -> bool {
        (self.validator)(context)
    }
}
```

### Custom Validators
Add domain-specific validation rules:

```rust
pub struct EmailValidator;

impl ValidationRule for EmailValidator {
    fn validate(&self, value: &str) -> Result<(), ValidationError> {
        if !value.contains('@') {
            return Err(ValidationError::InvalidFormat("Email must contain @".into()));
        }
        Ok(())
    }
}
```

## 📊 Comparison with Other Solutions

| Feature | NetabaseStore | Diesel | SeaORM | Sled | Redis |
|---------|---------------|---------|---------|-------|-------|
| Type Safety | ✅ Compile-time | ✅ Compile-time | ✅ Compile-time | ❌ Runtime | ❌ Runtime |
| Schema-less | ✅ | ❌ | ❌ | ✅ | ✅ |
| Permissions | ✅ Hierarchical | ❌ | ❌ | ❌ | ❌ |
| Cross-links | ✅ Type-safe | ✅ SQL | ✅ ORM | ❌ | ❌ |
| Multiple Backends | ✅ | ❌ SQL only | ❌ SQL only | ❌ | ❌ |
| Code Generation | ✅ Macros | ✅ CLI | ✅ CLI | ❌ | ❌ |

## 🎯 Design Trade-offs

### Advantages
- **Compile-time safety**: Catch errors before deployment
- **Performance**: Zero-cost abstractions
- **Flexibility**: Multiple backend support
- **Permissions**: Built-in hierarchical access control
- **Type safety**: Cross-definition relationships validated

### Limitations
- **Learning curve**: New concepts to understand
- **Compile time**: Large projects may have slower compile times
- **Schema evolution**: Changes require code updates
- **Memory usage**: In-memory backend can use significant RAM

## 🔮 Future Architecture

Planned enhancements:
- **Async support**: Non-blocking I/O operations
- **Query language**: SQL-like querying capabilities
- **Distributed permissions**: Cross-node permission synchronization
- **Schema migrations**: Automated data migration tools
- **Performance monitoring**: Built-in metrics and tracing

---

This architecture provides a solid foundation for building secure, type-safe, and performant database applications while maintaining flexibility and extensibility.