# Netabase Store Refactoring Plan

This document outlines the planned changes to improve the ergonomics, flexibility, and documentation of the netabase_store crate.

## Overview

The following changes aim to make the API more intuitive, flexible, and well-documented while maintaining backward compatibility where possible.

---

## 1. Convert Secondary Keys from Vec to HashMap

**Current State:**
- Secondary keys are stored as `Vec<SecondaryKeys>` returned by `secondary_keys()` method
- Finding a specific secondary key requires iterating through the vector
- Index-based access is not intuitive

**Desired State:**
- Secondary keys stored as `HashMap<SecondaryKeyDiscriminant, SecondaryKeys>`
- Direct access by key name/discriminant
- More intuitive API: `model.secondary_keys().get(&KeyType::Email)`

**Changes Required:**

### Files to Modify:
- `netabase_macros/src/generators/model_key.rs:233` - Update `secondary_keys()` generation to return HashMap
- `src/traits/model.rs:86` - Change trait signature from `Vec<SecondaryKeys>` to `HashMap<...>`
- `src/databases/sled_store.rs` - Update secondary key handling logic
- `src/databases/redb_store.rs` - Update secondary key handling logic
- `src/databases/redb_zerocopy.rs` - Update if zerocopy uses secondary keys
- `src/databases/indexeddb_store.rs` - Update for consistency
- `src/databases/memory_store.rs` - Update for consistency
- All tests and examples that use `secondary_keys()[0]` pattern
- `README.md` - Update examples
- `ARCHITECTURE.md` - Update architecture documentation

**Implementation Notes:**
- Need to determine discriminant type for HashMap key (likely a string-based enum discriminant)
- Consider adding convenience methods: `get_secondary_key(&str)` or similar
- Ensure bincode encoding/decoding works with HashMap

**Breaking Change:** Yes - API change requires version bump

**Priority:** Medium (improves ergonomics significantly)

---

## 2. Add `new()` Constructor to Generic NetabaseStore

**Current State:**
```rust
// Users must use backend-specific constructors:
let store = NetabaseStore::<Definition, _>::sled("path")?;
let store = NetabaseStore::<Definition, _>::redb("path")?;
```

**Desired State:**
```rust
// Generic new() that works with turbofish syntax:
let store = NetabaseStore::<Definition, SledStore<Definition>>::new("path")?;
let store = NetabaseStore::<Definition, RedbStore<Definition>>::new("path")?;
```

**Changes Required:**

### Files to Modify:
- `src/store.rs:72-106` - Add generic `new()` implementation:
  ```rust
  impl<D, Backend> NetabaseStore<D, Backend>
  where
      D: NetabaseDefinitionTrait,
      Backend: BackendFor<D> + BackendConstructor,
  {
      pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
          Ok(Self::from_backend(Backend::new(path)?))
      }
  }
  ```
- Need to create `BackendConstructor` trait for common constructor pattern
- Update documentation in `src/lib.rs` to show both patterns
- Update `README.md` with new constructor examples

**Implementation Notes:**
- This is additive, no breaking changes
- Keeps existing `sled()`, `redb()`, etc. constructors for convenience
- More flexible for generic code and abstractions
- May need separate trait implementations for backends with different constructor signatures (temp, memory, etc.)

**Breaking Change:** No - purely additive

**Priority:** High (frequently requested feature)

---

## 3. Fix or Remove Ignored Doctests

**Current State:**
- Multiple doctests use `ignore` directive in:
  - `netabase_macros/src/lib.rs` (lines 175, 226, 239)
  - `netabase_macros/src/generators/type_utils.rs` (lines 23, 67, 151, 243)
  - `netabase_macros/src/generators/zerocopy.rs` (lines 47, 60, 69, 113, 158, 302, 322)
  - `netabase_macros/src/generators/module_definition.rs` (lines 302, 334)
  - `netabase_macros/src/generators/table_definitions.rs` (line 11)
  - `src/databases/redb_zerocopy.rs` (lines 68, 76, 611)
  - `src/lib.rs` (lines 77, 193, 262, 319)
  - `src/store.rs` (lines 9, 142, 168, 283, 300, 332)
  - `src/transaction.rs`
  - Other files

**Desired State:**
- All doctests either:
  1. Run successfully (remove `ignore`)
  2. Properly marked with reason (e.g., `ignore: requires filesystem`)
  3. Converted to `no_run` if they're demonstration-only
  4. Removed if redundant/outdated

**Changes Required:**

### Process:
1. Audit all `ignore` directives - classify by reason:
   - Feature-gated (redb-zerocopy, etc.)
   - Requires filesystem/environment
   - Incomplete/broken examples
   - Demonstration-only

2. For each category:
   - Feature-gated: Change to `#[cfg_attr(...)]` or `no_run`
   - Filesystem: Change to `no_run` if valid, fix if invalid
   - Broken: Fix or remove
   - Demo: Change to `no_run`

3. Run `cargo test --doc` to verify

**Files to Check:**
- All files found in grep search above

**Breaking Change:** No

**Priority:** High (documentation quality)

---

## 4. Fix Malformed Documentation in redb_store Module

**Current State:**
- `src/databases/redb_store.rs` has documentation issues (need to review specific issues)

**Desired State:**
- Well-formatted, complete module documentation
- Proper doc examples
- Clear explanations of redb-specific features

**Changes Required:**

### Files to Modify:
- `src/databases/redb_store.rs` - Full documentation review:
  - Fix any malformed doc comments
  - Complete incomplete sentences/sections
  - Add missing examples
  - Ensure all public items have docs
  - Add module-level overview
  - Document differences from SledStore
  - Document redb-specific methods (check_integrity, compact, etc.)

**Implementation Notes:**
- Review against sled_store.rs documentation as reference
- Ensure consistency with overall crate documentation style
- Add comparison table: when to use Redb vs Sled

**Breaking Change:** No

**Priority:** High (documentation quality)

---

## 5. Add zerocopy-redb to Main API

**Current State:**
- zerocopy-redb exists in `src/databases/redb_zerocopy.rs`
- Guards module exists in `src/guards.rs` (behind feature flag)
- Not exposed in main public API or unified store interface

**Desired State:**
- Expose zerocopy-redb through main `NetabaseStore` API
- Document performance benefits
- Provide examples in main lib.rs

**Changes Required:**

### Files to Modify:
- `src/store.rs` - Add zerocopy-specific methods:
  ```rust
  #[cfg(all(feature = "redb", feature = "redb-zerocopy"))]
  impl<D> NetabaseStore<D, RedbStore<D>> {
      pub fn open_tree_zerocopy<M>(&self) -> ZeroCopyTree<...> {
          // Implementation
      }
  }
  ```

- `src/lib.rs` - Add documentation section:
  - Explain what zerocopy is
  - Show performance comparison
  - Provide usage examples
  - Document when to use it vs standard API

- Re-export guards module publicly:
  ```rust
  #[cfg(all(feature = "redb", feature = "redb-zerocopy"))]
  pub use guards::*;
  ```

- `README.md` - Add zerocopy section with:
  - Feature flag explanation
  - Performance benchmarks
  - Usage examples
  - Limitations (supported types)

**Implementation Notes:**
- Already implemented in Phase 4, just needs exposure
- Link to existing `REDB_ZEROCOPY.md` documentation
- Ensure examples work with `redb-zerocopy` feature enabled

**Breaking Change:** No - additive feature

**Priority:** Medium (powerful feature, needs visibility)

---

## 6. Expand Macro Documentation

**Current State:**
- `netabase_macros/src/lib.rs` has basic documentation for:
  - `NetabaseModel` derive macro (lines 16-118)
  - `netabase` attribute macro (lines 138-162)
  - `netabase_definition_module` attribute macro (lines 288-388)
- Documentation explains WHAT is generated, but not enough detail on WHY and HOW to use it

**Desired State:**
- Comprehensive macro documentation explaining:
  - **What gets generated** (detailed breakdown of each type/impl)
  - **Why each piece exists** (purpose, use cases)
  - **How to use the generated code** (patterns, best practices)
  - **Common pitfalls** and how to avoid them
  - **Advanced patterns** (multiple definitions, relationships)

**Changes Required:**

### Files to Modify:

#### `netabase_macros/src/lib.rs`:

1. **Expand `NetabaseModel` documentation (line 16)**:
   ```rust
   /// # What Gets Generated
   ///
   /// For a model like:
   /// ```rust
   /// #[derive(NetabaseModel, ...)]
   /// #[netabase(MyDef)]
   /// struct User {
   ///     #[primary_key] id: u64,
   ///     name: String,
   ///     #[secondary_key] email: String,
   /// }
   /// ```
   ///
   /// The macro generates:
   ///
   /// ## 1. Primary Key Newtype
   /// ```rust
   /// pub struct UserPrimaryKey(pub u64);
   /// ```
   /// **Why:** Type safety - prevents using wrong key type
   /// **How to use:** `tree.get(UserPrimaryKey(1))?`
   ///
   /// ## 2. Secondary Key Newtypes (one per #[secondary_key])
   /// ```rust
   /// pub struct UserEmailSecondaryKey(pub String);
   /// ```
   /// **Why:** Model-prefixed to avoid conflicts when multiple models have same field name
   /// **How to use:** Part of SecondaryKeys enum
   ///
   /// ## 3. Secondary Keys Enum
   /// ```rust
   /// pub enum UserSecondaryKeys {
   ///     Email(UserEmailSecondaryKey),
   /// }
   /// ```
   /// **Why:** Allows querying by any secondary key through unified type
   /// **How to use:** `tree.get_by_secondary_key(UserSecondaryKeys::Email(...))?`
   ///
   /// ## 4. Combined Keys Enum
   /// ```rust
   /// pub enum UserKey {
   ///     Primary(UserPrimaryKey),
   ///     Secondary(UserSecondaryKeys),
   /// }
   /// ```
   /// **Why:** Unified type for any key - used in batch operations
   /// **How to use:** `tree.get_by_key(UserKey::Primary(...))?`
   ///
   /// ## 5. NetabaseModelTrait Implementation
   /// ```rust
   /// impl NetabaseModelTrait<MyDef> for User { ... }
   /// ```
   /// **Why:** Provides runtime access to keys from model instances
   /// **How to use:** Automatically used by tree operations
   ///
   /// ## 6. Borrow Implementations
   /// Allows efficient key lookups without allocating new key instances.
   ///
   /// # Why This Architecture?
   ///
   /// - **Type Safety:** Can't accidentally use PostPrimaryKey with User tree
   /// - **Ergonomics:** Single trait covers all model types
   /// - **Performance:** Zero-cost abstractions, keys are newtypes
   /// - **Flexibility:** Easy to add new key types or models
   ///
   /// # Common Patterns
   /// [...]
   /// ```

2. **Expand `netabase_definition_module` documentation (line 288)**:
   - Add detailed "What Gets Generated" section
   - Explain discriminant enum generation
   - Explain Keys enum generation
   - Document trait implementations
   - Show how models are grouped
   - Explain table definitions struct (redb)

3. **Add troubleshooting section** to each macro doc:
   - Common compile errors and solutions
   - Import requirements
   - Feature flag interactions

#### New Documentation Files:
- `docs/MACRO_REFERENCE.md` - Comprehensive macro reference
- `docs/GENERATED_CODE.md` - Full example of generated code with annotations

#### Examples:
- `examples/macro_exploration.rs` - Example showing all generated types

**Implementation Notes:**
- Use `cargo expand` output to show exact generated code
- Link between related concepts
- Add diagrams/ASCII art for relationships
- Cross-reference with architecture docs

**Breaking Change:** No - documentation only

**Priority:** High (improves onboarding, reduces confusion)

---

## 7. Generate Convenience Functions for Secondary Keys

**Current State:**
```rust
// Current verbose API:
tree.get_by_secondary_key(
    UserSecondaryKeys::Email(UserEmailSecondaryKey("user@example.com".to_string()))
)?;
```

**Desired State:**
```rust
// Ergonomic extension trait API:
tree.get_by_secondary_key("user@example.com".as_user_email_key())?;
```

**Changes Required:**

### Files to Modify:

#### `netabase_macros/src/generators/model_key.rs`:

Add generation of extension traits for each secondary key type:

```rust
// For each secondary key, generate:
pub trait AsUserEmailKey {
    fn as_user_email_key(self) -> UserSecondaryKeys;
}

impl AsUserEmailKey for String {
    fn as_user_email_key(self) -> UserSecondaryKeys {
        UserSecondaryKeys::Email(UserEmailSecondaryKey(self))
    }
}

impl AsUserEmailKey for &str {
    fn as_user_email_key(self) -> UserSecondaryKeys {
        UserSecondaryKeys::Email(UserEmailSecondaryKey(self.to_string()))
    }
}

impl<'a> AsUserEmailKey for &'a String {
    fn as_user_email_key(self) -> UserSecondaryKeys {
        UserSecondaryKeys::Email(UserEmailSecondaryKey(self.clone()))
    }
}
```

**Implementation Strategy:**

1. **Macro Generation Phase:**
   - In `generate_secondary_keys_newtypes()`, also generate extension traits
   - Create trait name: `As{Model}{Field}Key` (e.g., `AsUserEmailKey`)
   - Implement for inner type + common conversions (&str, &String, etc.)

2. **Type Mapping:**
   - String → implement for String, &str, &String
   - u32/u64/etc → implement for that type + references
   - bool → implement for bool + reference
   - Custom types → implement for that type + reference

3. **Generated Pattern:**
   ```rust
   // For each field with #[secondary_key]:
   fn generate_key_extension_trait(
       model_name: &Ident,
       field_name: &Ident,
       field_type: &Type,
   ) -> TokenStream {
       let trait_name = format_ident!("As{}{}Key", model_name, uppercase(field_name));
       let secondary_keys = format_ident!("{}SecondaryKeys", model_name);
       let variant = uppercase(field_name);
       let key_type = format_ident!("{}{}SecondaryKey", model_name, uppercase(field_name));

       quote! {
           pub trait #trait_name {
               fn #method_name(self) -> #secondary_keys;
           }

           impl #trait_name for #field_type {
               fn #method_name(self) -> #secondary_keys {
                   #secondary_keys::#variant(#key_type(self))
               }
           }

           // Add reference implementations based on field_type
       }
   }
   ```

4. **Additional Convenience:**
   - Also generate `From<InnerType>` implementations:
     ```rust
     impl From<String> for UserSecondaryKeys {
         fn from(email: String) -> Self {
             Self::Email(UserEmailSecondaryKey(email))
         }
     }
     ```
   - This allows: `tree.get_by_secondary_key("email@example.com".into())?`

5. **Documentation Generation:**
   - Add examples to trait docs showing usage
   - Document which types have convenience methods

### Files to Update:
- `netabase_macros/src/generators/model_key.rs:8` - Update `generate_keys()` to include extension traits
- `netabase_macros/src/lib.rs:76-105` - Update NetabaseModel docs to show convenience API
- `src/lib.rs` - Update examples to use new ergonomic API
- `README.md` - Add section showing both APIs (verbose + ergonomic)
- `ARCHITECTURE.md` - Document the extension trait pattern

**Implementation Notes:**
- Consider naming: `as_user_email_key()` vs `to_user_email_key()` vs `user_email_key()`
- Follow Rust naming conventions (as_ for cheap, to_ for expensive)
- Since we're creating newtype, `as_` is more appropriate
- Could also generate type-specific methods on the tree itself (more advanced)

**Alternative/Additional Approach:**
Generate builder methods on SecondaryKeys enum:
```rust
impl UserSecondaryKeys {
    pub fn email(value: impl Into<String>) -> Self {
        Self::Email(UserEmailSecondaryKey(value.into()))
    }
}

// Usage:
tree.get_by_secondary_key(UserSecondaryKeys::email("user@example.com"))?;
```

**Breaking Change:** No - purely additive

**Priority:** High (major ergonomics improvement)

---

## Implementation Order

Recommended implementation sequence:

1. **Phase 1 - Documentation (Low Risk)**
   - Task 3: Fix/remove ignored doctests
   - Task 4: Fix redb_store documentation
   - Task 6: Expand macro documentation
   - **Rationale:** Improves understanding before code changes

2. **Phase 2 - Additive API Improvements (Medium Risk)**
   - Task 2: Add `new()` constructor to NetabaseStore
   - Task 7: Generate convenience functions for secondary keys
   - Task 5: Expose zerocopy-redb in main API
   - **Rationale:** Non-breaking, high value additions

3. **Phase 3 - Breaking Changes (High Risk)**
   - Task 1: Convert secondary keys Vec → HashMap
   - **Rationale:** Requires version bump, coordinate with users

---

## Testing Strategy

For each task:

1. **Unit Tests**
   - Test generated code compiles
   - Test macro expansion (use `cargo expand`)
   - Test new functionality in isolation

2. **Integration Tests**
   - Test with all backends (sled, redb, memory)
   - Test with/without feature flags
   - Test migration paths for breaking changes

3. **Documentation Tests**
   - All `cargo test --doc` must pass
   - Examples in README must work
   - Examples must be tested in CI

4. **Benchmark Tests**
   - Verify HashMap doesn't regress performance (Task 1)
   - Verify convenience functions have zero overhead (Task 7)

---

## Success Criteria

- [ ] All doctests pass (no `ignore` without good reason)
- [ ] API is more ergonomic (convenience functions work)
- [ ] Generic `new()` constructor available
- [ ] HashMap secondary keys improve lookup ergonomics
- [ ] Zerocopy redb is discoverable and documented
- [ ] Macro documentation is comprehensive
- [ ] All tests pass
- [ ] Benchmarks show no regressions
- [ ] Documentation is clear and complete

---

## Notes

- Consider creating a `MIGRATION.md` for Task 1 (HashMap change)
- May want to do Task 1 behind a feature flag initially
- Coordinate Task 1 with any downstream users
- Consider adding to CHANGELOG as tasks complete
- Update version number appropriately (SemVer)
