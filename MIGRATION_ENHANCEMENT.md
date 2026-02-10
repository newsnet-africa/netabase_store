# Migration Enhancement - Family Enum Implementation

## Current Gap

The existing migration system (in `netabase_macros/src/generators/model/migration.rs`) has:
- ✅ `VersionedModel` trait
- ✅ `MigrationChainExecutor` with `migrate_bytes(version, data)`
- ✅ Migration chain generation (V1 -> V2 -> V3)
- ❌ **Missing**: Automatic version detection from bytes

**Problem**: `migrate_bytes(version, data)` requires knowing the version upfront.
**Reality**: When reading from database, we don't know which version the bytes represent.

## Solution: Add Family Enum with Fallback Deserialization

### Implementation in migration.rs

Add new method to `MigrationGenerator`:

```rust
/// Generate a versioned family enum for automatic version detection.
fn generate_version_family_enum(&self, family: &ModelFamily) -> TokenStream {
    if family.versions.len() <= 1 {
        return quote! {}; // No enum needed for single version
    }
    
    let family_name = format_ident!("{}Family", family.family);
    let current_model = family.current_model();
    
    // Generate enum variants
    let variants = family.versions.iter().map(|model| {
        let version = model.version();
        let variant_name = format_ident!("V{}", version);
        let model_name = &model.name;
        quote! { #variant_name(#model_name) }
    });
    
    // Generate try_from_bytes that attempts each version
    let try_decode_arms = family.versions.iter().rev().map(|model| {
        let version = model.version();
        let variant_name = format_ident!("V{}", version);
        let model_name = &model.name;
        quote! {
            // Try this version
            if let Ok(decoded) = postcard::from_bytes::<#model_name>(bytes) {
                return Ok(#family_name::#variant_name(decoded));
            }
        }
    });
    
    // Generate to_current that migrates to current version
    let to_current_arms = family.versions.iter().enumerate().map(|(idx, model)| {
        let version = model.version();
        let variant_name = format_ident!("V{}", version);
        
        if version == family.current_version {
            quote! { #family_name::#variant_name(v) => v }
        } else {
            let chain = self.generate_migration_chain_call(family, idx);
            quote! { 
                #family_name::#variant_name(v) => {
                    let decoded = v;
                    #chain
                }
            }
        }
    });
    
    // Generate version() method
    let version_arms = family.versions.iter().map(|model| {
        let version = model.version();
        let variant_name = format_ident!("V{}", version);
        quote! { #family_name::#variant_name(_) => #version }
    });
    
    let current_model_name = &current_model.name;
    
    quote! {
        /// Enum representing all versions of the #family_name family.
        /// 
        /// This enum enables automatic version detection when deserializing
        /// from unknown binary data. It will try each version until one succeeds.
        #[derive(Debug, Clone)]
        pub enum #family_name {
            #(#variants),*
        }
        
        impl #family_name {
            /// Attempt to deserialize from bytes, trying each version.
            ///
            /// Tries versions in reverse order (newest first) for better performance,
            /// as most data in production will be the current version.
            ///
            /// # Returns
            /// - `Ok(family_variant)` if any version successfully deserializes
            /// - `Err(_)` if all versions fail
            pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
                #(#try_decode_arms)*
                
                // No version matched
                Err(postcard::Error::DeserializeUnexpectedEnd)
            }
            
            /// Convert any version to the current version.
            ///
            /// If already the current version, returns as-is.
            /// Otherwise, automatically migrates through the chain.
            pub fn to_current(self) -> #current_model_name {
                match self {
                    #(#to_current_arms),*
                }
            }
            
            /// Get the version number of this instance.
            pub fn version(&self) -> u32 {
                match self {
                    #(#version_arms),*
                }
            }
            
            /// Try to downgrade to a specific version (if supported).
            ///
            /// Returns `None` if the target version doesn't support downgrade.
            pub fn try_downgrade_to(&self, target_version: u32) -> Option<Vec<u8>> {
                use netabase_store::traits::migration::VersionedEncode;
                
                // First migrate to current if needed
                let current = match self {
                    #(#to_current_arms),*
                };
                
                // Then use encode_for_version
                current.encode_for_version(target_version)
            }
        }
        
        // Auto-implement From for ergonomic construction
        impl From<#family_name> for #current_model_name {
            fn from(family: #family_name) -> Self {
                family.to_current()
            }
        }
    }
}
```

### Integration Point

In `MigrationGenerator::generate()`:

```rust
pub fn generate(&self) -> TokenStream {
    let mut output = TokenStream::new();

    // ... existing code ...

    // NEW: Generate family enums
    output.extend(self.generate_family_enums());

    output
}

fn generate_family_enums(&self) -> TokenStream {
    let mut enums = TokenStream::new();
    
    for family in self.visitor.model_families.values() {
        // Only generate for families with versioning
        if family.versions.first().map(|m| m.version_info().is_some()).unwrap_or(false) {
            enums.extend(self.generate_version_family_enum(family));
        }
    }
    
    enums
}
```

### Update NetabaseModel to Use Family Enum

In `generators/model/serialization.rs`, update the decode implementation:

```rust
// Current (knows version):
fn from_bytes(data: &[u8]) -> Result<Self, Error> {
    let version = detect_version(data); // How?
    MigrationChain::migrate_bytes(version, data)
}

// NEW (tries all versions):
fn from_bytes(data: &[u8]) -> Result<Self, Error> {
    UserFamily::try_from_bytes(data)
        .map(|family| family.to_current())
        .map_err(Into::into)
}
```

## Example Generated Code

For:
```rust
#[netabase_version(family = "User", version = 1)]
pub struct UserV1 { id: String, name: String }

#[netabase_version(family = "User", version = 2, current)]
pub struct UserV2 { id: String, first_name: String, last_name: String }

impl MigrateFrom<UserV1> for UserV2 { ... }
```

Generates:
```rust
#[derive(Debug, Clone)]
pub enum UserFamily {
    V1(UserV1),
    V2(UserV2),
}

impl UserFamily {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
        // Try V2 first (current)
        if let Ok(decoded) = postcard::from_bytes::<UserV2>(bytes) {
            return Ok(UserFamily::V2(decoded));
        }
        
        // Try V1
        if let Ok(decoded) = postcard::from_bytes::<UserV1>(bytes) {
            return Ok(UserFamily::V1(decoded));
        }
        
        Err(postcard::Error::DeserializeUnexpectedEnd)
    }
    
    pub fn to_current(self) -> UserV2 {
        match self {
            UserFamily::V2(v) => v,
            UserFamily::V1(v) => UserV2::migrate_from(v),
        }
    }
    
    pub fn version(&self) -> u32 {
        match self {
            UserFamily::V1(_) => 1,
            UserFamily::V2(_) => 2,
        }
    }
}

impl From<UserFamily> for UserV2 {
    fn from(family: UserFamily) -> Self {
        family.to_current()
    }
}

// NetabaseModel now uses this:
impl NetabaseModel for UserV2 {
    fn from_stored_bytes(data: &[u8]) -> Result<Self, Error> {
        UserFamily::try_from_bytes(data)?
            .to_current()
            .map_err(Into::into)
    }
}
```

## Benefits

1. **Automatic Version Detection** ✅
   - No need to store version separately
   - No need to track schema externally
   - Binary data is self-describing through structure

2. **Transparent Migration** ✅
   - User code always works with current version
   - Migration happens automatically on read
   - Write always uses current version

3. **Database Flexibility** ✅
   - Can have mixed versions in same table
   - Rolling migrations supported
   - No downtime needed for schema changes

4. **P2P Compatible** ✅
   - Nodes can run different versions
   - Automatic upgrade of received data
   - Optional downgrade for older nodes

## Performance

- **Common case** (current version): Single deserialization attempt
- **Old version**: One failed attempt + one successful + migration chain
- **Very old**: Multiple failed attempts + migration (still fast)

Optimization: Could use version header if available, fall back to probing.

## Next Steps

1. Implement `generate_version_family_enum` in migration.rs
2. Update serialization.rs to use family enum
3. Add tests in example crate showing V1 → V2 migration
4. Document in tutorial
5. Add CLI tool for bulk migration (optional)

This makes the migration system robust and production-ready! 🚀
