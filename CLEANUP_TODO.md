# Netabase Store - Cleanup & Refactoring Todo List

**Created:** 2025-12-23
**Status:** Ready for Implementation
**Priority:** Medium (performance is excellent, focus on maintainability)

---

## High Priority (Do First)

### 1. ✅ Fix Benchmark Fairness [COMPLETED]

**Status:** ✅ Done
**Files Modified:**
- `boilerplate/benches/crud.rs`

**Changes Made:**
- Added `split_blob_into_chunks()` helper function
- Added `BLOB_ANOTHER` table definition
- Updated raw insert to properly split both bio and another blobs
- Updated raw read setup to properly split blobs
- Updated raw delete setup and operation to handle all blob chunks

**Impact:** Benchmarks now accurately compare abstracted vs raw performance (abstracted is 5-11% faster).

---

### 2. 🔧 Consolidate Duplicate `to_pascal_case` Function

**Priority:** HIGH
**Effort:** Low (30 minutes)
**Impact:** Reduces code duplication, single source of truth

**Current State:**
- **Location 1:** `netabase_macros/src/generators/model/traits.rs:312-322`
- **Location 2:** `netabase_macros/src/generators/model/key_enums.rs:405-415`
- **Location 3:** `netabase_macros/src/generators/model/serialization.rs:251-261`

**Problem:** Identical function defined in 3 separate files.

**Solution:**
1. Create new file: `netabase_macros/src/utils/naming.rs`
2. Move `to_pascal_case()` function there
3. Add to `netabase_macros/src/utils/mod.rs`:
   ```rust
   pub mod naming;
   ```
4. Replace all 3 definitions with:
   ```rust
   use crate::utils::naming::to_pascal_case;
   ```

**Files to Modify:**
- Create: `netabase_macros/src/utils/naming.rs`
- Modify: `netabase_macros/src/utils/mod.rs`
- Modify: `netabase_macros/src/generators/model/traits.rs` (remove lines 312-322, add import)
- Modify: `netabase_macros/src/generators/model/key_enums.rs` (remove lines 405-415, add import)
- Modify: `netabase_macros/src/generators/model/serialization.rs` (remove lines 251-261, add import)

**Testing:**
```bash
cargo test
cargo check --benches
```

**Expected Result:** No behavior change, reduced line count by ~30 lines.

---

### 3. 🔧 Replace Manual `IntoDiscriminant` with Derive Macro

**Priority:** HIGH
**Effort:** Medium (2-3 hours)
**Impact:** Eliminates 6 repetitive implementations, delegates to strum

**Current State:**
Manual `IntoDiscriminant` implementations in `key_enums.rs`:
- Lines 67 (secondary empty)
- Lines 120 (secondary non-empty)
- Lines 163 (relational empty)
- Lines 216 (relational non-empty)
- Lines 265 (blob empty)
- Lines 317 (blob non-empty)

**Problem:** Manually implementing what `#[derive(strum::EnumDiscriminants)]` already provides.

**Solution:**

**Option A (Preferred):** Use strum's derive macros
```rust
// Before:
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UserSecondaryKeys {
    Name(UserName),
    Age(UserAge),
}

impl strum::IntoDiscriminant for UserSecondaryKeys {
    type Discriminant = UserSecondaryKeysTreeName;
    fn discriminant(&self) -> Self::Discriminant { ... }
}

// After:
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(strum::EnumDiscriminants)]
#[strum_discriminants(name(UserSecondaryKeysTreeName))]
#[strum_discriminants(derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord))]
pub enum UserSecondaryKeys {
    Name(UserName),
    Age(UserAge),
}
```

**Option B (If strum doesn't support your exact pattern):** Create a custom derive macro

**Files to Modify:**
- `netabase_macros/src/generators/model/key_enums.rs`
  - Remove manual `IntoDiscriminant` impl generation
  - Add derive attributes to generated enums
  - Update lines: 67, 120, 163, 216, 265, 317

**Testing:**
```bash
cargo test
# Verify discriminant usage in CRUD operations still works
cd boilerplate && cargo bench --bench crud
```

**Expected Result:** -100 lines of generated code, same functionality.

---

## Medium Priority

### 4. 🔧 Move Key Getter Methods to Trait Defaults

**Priority:** MEDIUM
**Effort:** Medium (2 hours)
**Impact:** ~40 lines saved per model with empty key collections

**Current State:**
These methods are generated even when they just return `vec![]`:
- `get_secondary_keys()` - `netabase_macros/src/generators/model/traits.rs:219-234`
- `get_relational_keys()` - `netabase_macros/src/generators/model/traits.rs:236-253`
- `get_subscription_keys()` - `netabase_macros/src/generators/model/traits.rs:255-274`

**Problem:** Models without these keys still get generated stub implementations.

**Solution:**

**Step 1:** Add defaults to `NetabaseModel` trait (`src/traits/registery/models/model/mod.rs`):

```rust
pub trait NetabaseModel<D: NetabaseDefinition>: Sized + Clone {
    // ... existing associated types ...

    // Add default implementations:
    fn get_secondary_keys<'a>(
        &'a self,
    ) -> Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Secondary<'a>> {
        vec![] // Default: no secondary keys
    }

    fn get_relational_keys<'a>(
        &'a self,
    ) -> Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Relational<'a>> {
        vec![] // Default: no relational keys
    }

    fn get_subscription_keys<'a>(
        &'a self,
    ) -> Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Subscription<'a>> {
        vec![] // Default: no subscriptions
    }

    // Existing methods without changes:
    fn get_primary_key(&self) -> <Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'_>;
    fn get_blob_entries(&self) -> Vec<Vec<(...)>>;
}
```

**Step 2:** Update macro generators to **skip** generating these methods when collections are empty:

```rust
// In netabase_macros/src/generators/model/traits.rs:

fn generate_get_secondary_keys(&self) -> TokenStream {
    if self.visitor.secondary_keys.is_empty() {
        // Don't generate, use trait default
        return quote! {};
    }

    // Only generate for non-empty:
    let model_name = &self.visitor.model_name;
    let keys_enum = secondary_keys_enum_name(model_name);
    let variants = ...;
    quote! {
        fn get_secondary_keys<'b>(&'b self) -> Vec<#keys_enum> {
            vec![#(#variants),*]
        }
    }
}
```

**Files to Modify:**
- `src/traits/registery/models/model/mod.rs` - Add default implementations
- `netabase_macros/src/generators/model/traits.rs` - Skip generation when empty

**Testing:**
```bash
cargo test
# Test model with no secondary keys (should compile)
# Test model with secondary keys (should still work)
```

**Expected Result:**
- Models with no keys: 0 lines generated (use defaults)
- Models with keys: same as before (override defaults)
- Total savings: ~40 lines per simple model

---

### 5. 🔧 Remove Unnecessary Blob Trait Method Overrides

**Priority:** MEDIUM
**Effort:** Medium (2 hours)
**Impact:** ~40 lines saved per model, leverage trait defaults

**Current State:**
`NetabaseBlobItem` trait (`src/blob.rs:12-53`) already provides defaults for:
- `split_into_blobs()` (lines 21-33)
- `reconstruct_from_blobs()` (lines 35-52)

But macros generate explicit implementations anyway (`netabase_macros/src/generators/model/serialization.rs`).

**Problem:** Trait defaults are ignored, logic is duplicated in generated code.

**Solution:**

**For individual blob field types** (e.g., `LargeUserFile`, `AnotherLargeUserFile`):

```rust
// Current generated:
impl NetabaseBlobItem for LargeUserFile {
    type Blobs = UserBlobItem;

    fn wrap_blob(index: u8, data: Vec<u8>) -> UserBlobItem { ... }
    fn unwrap_blob(blob: &UserBlobItem) -> Option<(u8, Vec<u8>)> { ... }

    // REMOVE THESE (use trait defaults instead):
    fn split_into_blobs(&self) -> Vec<Self::Blobs> { ... }
    fn reconstruct_from_blobs(blobs: Vec<Self::Blobs>) -> Self { ... }
}

// After:
impl NetabaseBlobItem for LargeUserFile {
    type Blobs = UserBlobItem;

    fn wrap_blob(index: u8, data: Vec<u8>) -> UserBlobItem { ... }
    fn unwrap_blob(blob: &UserBlobItem) -> Option<(u8, Vec<u8>)> { ... }

    // Let trait defaults handle split/reconstruct
}
```

**For the blob enum itself** (special case, keep custom logic):

The `UserBlobItem` enum needs special `reconstruct_from_blobs()` because it delegates to the appropriate field type. Keep this custom implementation.

**Files to Modify:**
- `netabase_macros/src/generators/model/serialization.rs`
  - Lines 162-164, 166-172: Remove from individual field implementations
  - Lines 223-243: Keep for enum itself (special case)

**Testing:**
```bash
cargo test
cd boilerplate && cargo run
# Verify blob reconstruction works (boilerplate/src/main.rs:100-117)
```

**Expected Result:** ~40 lines less generated code per model, same functionality.

---

### 6. 🔧 Consolidate Empty Enum Generation Pattern

**Priority:** MEDIUM
**Effort:** Low (1 hour)
**Impact:** Reduces pattern repetition, easier to maintain

**Current State:**
Empty enum pattern duplicated in:
- `key_enums.rs:42-74` (secondary empty)
- `key_enums.rs:138-170` (relational empty)
- `key_enums.rs:240-272` (blob empty)

**Problem:** Same pattern copy-pasted 3 times.

**Solution:**

Create helper function in `key_enums.rs`:

```rust
fn generate_empty_key_enum(
    enum_name: &Ident,
    tree_name: &Ident,
) -> TokenStream {
    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum #enum_name {
            None
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, strum::EnumDiscriminants)]
        pub enum #tree_name {
            None
        }

        impl strum::IntoDiscriminant for #enum_name {
            type Discriminant = #tree_name;

            fn discriminant(&self) -> Self::Discriminant {
                match self {
                    #enum_name::None => #tree_name::None,
                }
            }
        }
    }
}
```

Then use it:
```rust
// Before:
if self.visitor.secondary_keys.is_empty() {
    // 30 lines of enum generation
}

// After:
if self.visitor.secondary_keys.is_empty() {
    return generate_empty_key_enum(&enum_name, &tree_name);
}
```

**Files to Modify:**
- `netabase_macros/src/generators/model/key_enums.rs` (consolidate lines 42-74, 138-170, 240-272)

**Testing:**
```bash
cargo test
```

**Expected Result:** -60 lines of duplicate code, same functionality.

---

## Low Priority (Nice to Have)

### 7. 🔍 Evaluate Relational Key Wrapper Types

**Priority:** LOW
**Effort:** High (requires design review)
**Impact:** Potential simplification, but may reduce type safety

**Current State:**
Each relational field generates a wrapper type:
- `netabase_macros/src/generators/model/wrapper_types.rs:72-90`

Example:
```rust
pub struct UserAuthorWrapper(pub AuthorID);

enum UserRelationalKeys {
    Author(UserAuthorWrapper),  // Why not just AuthorID?
}
```

**Question:** Is the wrapper necessary?

**Pros of wrapper:**
- Stronger type distinction (UserAuthorWrapper vs AuthorID)
- Could add methods/traits specific to that relationship

**Cons of wrapper:**
- Extra indirection
- Immediately unwrapped in `get_relational_keys()`
- Enum variant name already provides context

**Action Items:**
1. Review design rationale with team
2. If wrapper is unnecessary, simplify to direct key types
3. If wrapper is intentional, document why

**No immediate action required** - this is a design discussion point.

---

### 8. 📝 Add Documentation for Generated Code

**Priority:** LOW
**Effort:** Medium (ongoing)
**Impact:** Improves developer experience

**Current State:**
Generated enums and traits have minimal documentation.

**Solution:**
Add doc comments to generated code:

```rust
// Current:
pub enum UserSecondaryKeys {
    Name(UserName),
    Age(UserAge),
}

// Improved:
/// Secondary key enum for User model.
///
/// Used to index users by secondary attributes (name, age).
/// Each variant wraps the corresponding field value.
pub enum UserSecondaryKeys {
    /// Index by user name
    Name(UserName),
    /// Index by user age
    Age(UserAge),
}
```

**Files to Modify:**
- All generator files in `netabase_macros/src/generators/`
- Add `#[doc = "..."]` attributes to generated items

**Testing:**
```bash
cargo doc --open
# Verify generated docs are readable
```

---

### 9. 🧹 Fix Compiler Warnings

**Priority:** LOW
**Effort:** Low (30 minutes)
**Impact:** Cleaner build output

**Current Warnings:**

From `netabase_macros`:
- 29 warnings (unused imports, variables, dead code)
- Examples: `unused import: Path`, `unused variable: field_info`

From `netabase_store`:
- 11 warnings (unused imports)
- Examples: `unused import: StoreKey`, `unused import: ReadableDatabase`

**Solution:**
```bash
cargo fix --lib -p netabase_macros
cargo fix --lib -p netabase_store
```

Then manually review and commit changes.

**Files to Modify:**
- Various files flagged by `cargo fix`

**Testing:**
```bash
cargo build
# Should see 0 warnings
```

---

### 10. 📊 Add Additional Benchmarks

**Priority:** LOW
**Effort:** Medium (3-4 hours)
**Impact:** More comprehensive performance data

**Current State:**
Only Insert benchmarks completed (Read/Update/Delete ran out of disk space).

**Solution:**

1. **Fix disk space issue:**
   - Use smaller test sizes (0, 10, 100, 1000, 5000 instead of 100000)
   - Clean up test databases after each iteration
   - Use RAM disk: `mkdir /dev/shm/netabase_bench`

2. **Complete Read benchmarks:**
   - Measure blob reconstruction overhead
   - Measure index lookup performance

3. **Complete Update benchmarks:**
   - Measure differential key update performance
   - Measure blob replacement overhead

4. **Complete Delete benchmarks:**
   - Measure cascading deletion overhead
   - Measure index cleanup performance

5. **Add new benchmarks:**
   - Concurrent transactions (multi-threaded)
   - Large blob sizes (>60KB, multi-chunk)
   - Query benchmarks (secondary key lookups)

**Files to Modify:**
- `boilerplate/benches/crud.rs` - Reduce test sizes, add cleanup
- Create: `boilerplate/benches/query.rs` - Index lookup benchmarks
- Create: `boilerplate/benches/concurrent.rs` - Multi-threaded benchmarks

**Testing:**
```bash
cd boilerplate
cargo bench
```

---

## Implementation Priority Order

**Week 1 (High Priority):**
1. ✅ Fix benchmark fairness [DONE]
2. 🔧 Consolidate `to_pascal_case` function
3. 🔧 Replace manual `IntoDiscriminant` with derive

**Week 2 (Medium Priority):**
4. 🔧 Move key getter methods to trait defaults
5. 🔧 Remove unnecessary blob trait overrides
6. 🔧 Consolidate empty enum generation

**Week 3+ (Low Priority, as time permits):**
7. 🔍 Evaluate relational key wrappers (design discussion)
8. 📝 Add documentation for generated code
9. 🧹 Fix compiler warnings
10. 📊 Add additional benchmarks

---

## Success Metrics

**Code Quality:**
- ✅ Benchmark fairness: Both approaches do identical work
- 🎯 Line count reduction: ~200-300 lines less generated code
- 🎯 Warning count: 0 compiler warnings
- 🎯 Trait default usage: 3+ methods use defaults instead of generation

**Performance:**
- ✅ Abstracted overhead: 5-11% faster than raw (achieved)
- 🎯 Benchmark coverage: Insert, Read, Update, Delete all tested
- 🎯 Scaling validation: Sub-linear scaling maintained

**Maintainability:**
- 🎯 Code duplication: 0 duplicate helper functions
- 🎯 Documentation: All generated types have doc comments
- 🎯 Trait design: Logic in traits, not macros

---

## Notes

**Performance Status:**
- ✅ Current performance is **excellent** (5-11% faster than raw redb)
- ✅ Zero-cost abstractions achieved
- ✅ Production-ready

**Focus Area:**
- These refactorings are about **maintainability**, not performance
- Performance improvements are negligible (code quality is the goal)

**Breaking Changes:**
- Most refactorings are internal (macro generators)
- Public API remains unchanged
- Trait default additions are **non-breaking** (backward compatible)

**Testing Strategy:**
- Run `cargo test` after each change
- Run benchmarks to verify performance is maintained
- Test with boilerplate example to verify generated code works

---

## Related Documents

- [PERFORMANCE_ASSESSMENT.md](./PERFORMANCE_ASSESSMENT.md) - Detailed benchmark analysis
- [README.md](./README.md) - Project overview
- [src/blob.rs](./src/blob.rs) - Blob trait definitions
- [netabase_macros/](./netabase_macros/) - Macro generators

---

**Last Updated:** 2025-12-23
**Reviewer:** Claude Sonnet 4.5
**Status:** Ready for implementation
