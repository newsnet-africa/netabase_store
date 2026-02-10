# Migration Architecture - Gap Analysis and Solution

## Current State - Problems Identified

### 1. **No Runtime Version Detection**
Currently, when reading from the database:
- Deserializes directly into the "current" version
- If the structure doesn't match → deserialization fails
- No fallback to try older versions
- **Result:** Can't read data from older schema versions

### 2. **No Version Family Enum**
The `#[netabase_version(family = "User", version = 2)]` attribute exists but doesn't generate:
- An enum wrapping all versions: `enum UserFamily { V1(UserV1), V2(UserV2) }`
- Version discrimination logic
- Automatic version detection from binary data

### 3. **Migration Path Gaps**
- No way to detect which version is stored in database
- No automatic routing through migration chain
- User must manually implement migration for each read

## Proposed Solution

### Architecture: Version Family Enum Pattern

For each model family, generate:

```rust
// User code defines:
#[netabase_version(family = "User", version = 1)]
pub struct UserV1 { ... }

#[netabase_version(family = "User", version = 2, current)]
pub struct UserV2 { ... }

impl MigrateFrom<UserV1> for UserV2 { ... }

// Macro generates:
#[derive(Serialize, Deserialize)]
pub enum UserFamily {
    V1(UserV1),
    V2(UserV2),
    // Future versions...
}

impl UserFamily {
    /// Try to deserialize from bytes, attempting each version
    pub fn from_bytes(data: &[u8]) -> Result<Self, Error> {
        // Try current version first (optimization)
        if let Ok(v2) = postcard::from_bytes::<UserV2>(data) {
            return Ok(UserFamily::V2(v2));
        }
        
        // Fall back to older versions
        if let Ok(v1) = postcard::from_bytes::<UserV1>(data) {
            return Ok(UserFamily::V1(v1));
        }
        
        Err(Error::NoVersionMatched)
    }
    
    /// Migrate to current version
    pub fn to_current(self) -> UserV2 {
        match self {
            UserFamily::V2(v) => v,
            UserFamily::V1(v) => UserV2::migrate_from(v),
        }
    }
    
    /// Get version number
    pub fn version(&self) -> u32 {
        match self {
            UserFamily::V1(_) => 1,
            UserFamily::V2(_) => 2,
        }
    }
}

// The NetabaseModel impl uses UserFamily internally
impl NetabaseModel for UserV2 {
    // Internal: deserialize uses family enum
    fn from_stored_bytes(data: &[u8]) -> Result<Self, Error> {
        UserFamily::from_bytes(data)?.to_current()
    }
}
```

### Benefits

1. **Automatic Version Detection**
   - Try deserialization with each version
   - Fall back gracefully
   - No manual version tracking needed

2. **Transparent Migration**
   - User reads `UserV2`
   - System automatically migrates from V1 if needed
   - Migration chain handled internally

3. **Schema Evolution Safety**
   - Database can contain mixed versions
   - All reads succeed (migrate on read)
   - Write always uses current version

4. **P2P Compatibility**
   - Can serialize to any version in family
   - Downgrade for older nodes using `MigrateTo`
   - Upgrade incoming data automatically

## Implementation Plan

### Phase 1: Generate Version Family Enum (Macro Changes)
```rust
// In netabase_macros/src/generators/model.rs
fn generate_version_family_enum(family_name: &str, versions: &[(u32, Type)]) -> TokenStream {
    let variant_defs = versions.iter().map(|(ver, ty)| {
        let variant = format_ident!("V{}", ver);
        quote! { #variant(#ty) }
    });
    
    let family_enum = format_ident!("{}Family", family_name);
    
    quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub enum #family_enum {
            #(#variant_defs),*
        }
    }
}
```

### Phase 2: Generate Deserialization Logic
```rust
fn generate_family_deserialize(family: &str, versions: &[(u32, Type)]) -> TokenStream {
    let family_enum = format_ident!("{}Family", family);
    
    // Try versions in reverse order (newest first for perf)
    let try_deserialize = versions.iter().rev().map(|(ver, ty)| {
        let variant = format_ident!("V{}", ver);
        quote! {
            if let Ok(data) = postcard::from_bytes::<#ty>(bytes) {
                return Ok(#family_enum::#variant(data));
            }
        }
    });
    
    quote! {
        impl #family_enum {
            pub fn from_bytes(bytes: &[u8]) -> Result<Self, postcard::Error> {
                #(#try_deserialize)*
                Err(postcard::Error::DeserializeUnexpectedEnd)
            }
        }
    }
}
```

### Phase 3: Generate Migration Chain
```rust
fn generate_to_current(family: &str, versions: &[(u32, Type)], current_ver: u32) -> TokenStream {
    let current_ty = versions.iter()
        .find(|(v, _)| *v == current_ver)
        .map(|(_, ty)| ty)
        .unwrap();
        
    let migration_arms = versions.iter().map(|(ver, ty)| {
        let variant = format_ident!("V{}", ver);
        if *ver == current_ver {
            quote! {
                #family_enum::#variant(v) => v
            }
        } else {
            // Generate migration chain: V1 -> V2 -> V3 -> Current
            quote! {
                #family_enum::#variant(v) => {
                    // Chain migrations: v -> V{ver+1} -> ... -> Current
                    #current_ty::migrate_from(v)
                }
            }
        }
    });
    
    quote! {
        impl #family_enum {
            pub fn to_current(self) -> #current_ty {
                match self {
                    #(#migration_arms),*
                }
            }
        }
    }
}
```

### Phase 4: Integrate with NetabaseModel
```rust
// Modify the generated NetabaseModel impl
impl NetabaseModel for UserV2 {
    type Keys = UserKeys;
    
    fn from_stored_bytes(bytes: &[u8]) -> Result<Self, Error> {
        // Use the family enum for deserialization
        UserFamily::from_bytes(bytes)
            .map_err(Into::into)?
            .to_current()
    }
    
    fn to_stored_bytes(&self) -> Result<Vec<u8>, Error> {
        // Always serialize as current version
        postcard::to_allocvec(self).map_err(Into::into)
    }
}
```

## Migration Guarantees

With this implementation:

1. **Forward Compatibility**: Old database + new binary = ✅ (auto-migrate on read)
2. **Backward Compatibility**: New database + old binary = ❌ (expected, need version pinning)
3. **Mixed Versions**: Database with V1 and V2 records = ✅ (both readable)
4. **P2P Sync**: Different node versions = ✅ (with MigrateTo implemented)

## Performance Considerations

1. **Optimization**: Try current version first (most common case)
2. **Lazy Migration**: Migrate on read, write back as current version
3. **Batch Migration**: Separate tool to migrate entire database at once
4. **Cache**: Version detection result can be cached per-table

## Breaking Changes

None - this is additive:
- Existing code without `#[netabase_version]` works as before
- Family enum is opt-in by using the attribute
- Migration is transparent to user code

## Next Steps

1. Implement macro generation for family enum
2. Add runtime deserialization fallback
3. Test with example showing V1 → V2 migration
4. Document the pattern in tutorial
5. Add CLI tool for bulk migration
