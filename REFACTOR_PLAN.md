# Netabase Store - Comprehensive Reorganization Plan

## Executive Summary

This plan reorganizes `netabase_store` and `netabase_macros` to improve:
- **Maintainability**: Break down 1600+ line files into focused modules
- **Ergonomics**: Clearer import paths and better prelude organization
- **Consistency**: Unified patterns across similar features
- **Documentation**: Comprehensive, tested docs with minimal `ignore` attributes
- **Feature Gating**: Proper conditional compilation for all optional features
- **Testing**: Organized test structure with clear naming

## Current State Analysis

### Problems Identified

#### 1. **File Size Issues**
- `src/databases/redb/transaction/mod.rs`: 1648 lines
- `src/databases/redb/transaction/crud.rs`: 1322 lines
- `src/databases/redb/mod.rs`: 777 lines
- `src/query.rs`: 651 lines
- `src/relational.rs`: 640 lines
- `netabase_macros/src/generators/definition/traits.rs`: 1923 lines

#### 2. **Module Organization**
- Flat structure in `src/` with many top-level files
- `traits/registery` has confusing naming (typo: should be "registry")
- Small utility modules scattered across codebase
- Inconsistent use of `mod.rs` vs file modules

#### 3. **Import Ergonomics**
- Deep nested imports required: `traits::registery::definition::schema::...`
- Prelude includes everything but path is cumbersome
- No clear separation between public API and internals

#### 4. **Feature Gates**
- Features defined in Cargo.toml but minimal conditional compilation
- No feature gates in macro crate despite preparation needed
- Blob, repository, migration, libp2p features not consistently gated

#### 5. **Testing Structure**
- 29 integration test files with unclear organization
- Names like `all_tables_comprehensive.rs`, `comprehensive_functionality.rs` overlap
- `common/` module underutilized
- Example crate used for testing - confusing purpose

#### 6. **Documentation**
- Many doc examples use `ignore` or `no_run`
- Boilerplate not reused across similar features
- Generated code lacks user-facing documentation

#### 7. **Pattern Inconsistencies**
- CRUD operations spread across multiple traits
- Query building inconsistent between features
- Error handling patterns vary

---

## Proposed Architecture

### 1. Core Module Restructure - `netabase_store/src/`

```
src/
├── lib.rs                          # Slim public API + feature gates
├── prelude.rs                      # Carefully curated re-exports
│
├── core/                           # Core types (no feature gates)
│   ├── mod.rs
│   ├── key.rs                      # Primary key abstractions
│   ├── primitives.rs               # Base types and constants
│   ├── error.rs                    # Error types
│   └── capabilities.rs             # Capability bitflags
│
├── model/                          # Model trait system
│   ├── mod.rs
│   ├── traits.rs                   # NetabaseModel trait
│   ├── keys.rs                     # Key trait hierarchy
│   │   ├── primary.rs
│   │   ├── secondary.rs (feature: secondary_keys)
│   │   ├── relational.rs (feature: relational_keys)
│   │   ├── subscription.rs (feature: subscriptions)
│   │   └── blob.rs (feature: blobs)
│   └── metadata.rs                 # Model metadata traits
│
├── definition/                     # Definition trait system
│   ├── mod.rs
│   ├── traits.rs                   # NetabaseDefinition trait
│   ├── schema.rs                   # Schema export/import
│   ├── tree_names.rs               # Table name enums
│   └── redb_definition.rs          # Redb-specific definition trait
│
├── repository/                     # Repository pattern (feature: repository)
│   ├── mod.rs
│   ├── traits.rs                   # NetabaseRepository trait
│   └── marker.rs                   # Repository marker types
│
├── store/                          # Database store implementations
│   ├── mod.rs
│   ├── traits.rs                   # NBStore trait
│   ├── hash.rs                     # Content hashing
│   └── backends/
│       ├── mod.rs
│       ├── redb/
│       │   ├── mod.rs
│       │   ├── store.rs            # RedbStore impl (~400 lines max)
│       │   ├── transaction/
│       │   │   ├── mod.rs          # Transaction types (~300 lines)
│       │   │   ├── read.rs         # Read-only operations
│       │   │   ├── write.rs        # Write operations
│       │   │   ├── crud/
│       │   │   │   ├── mod.rs      # CRUD trait (~150 lines)
│       │   │   │   ├── create.rs   # Create operations
│       │   │   │   ├── read.rs     # Read operations
│       │   │   │   ├── update.rs   # Update operations
│       │   │   │   ├── delete.rs   # Delete operations
│       │   │   │   └── list.rs     # List operations
│       │   │   ├── tables.rs       # Table management
│       │   │   └── options.rs      # Transaction options
│       │   ├── migration.rs (feature: migration, ~150 lines)
│       │   ├── repository.rs (feature: repository, ~200 lines)
│       │   └── libp2p.rs (feature: libp2p, ~150 lines)
│       └── indexeddb/              # Future: Browser support
│           └── mod.rs
│
├── query/                          # Query system
│   ├── mod.rs
│   ├── config.rs                   # QueryConfig, Pagination
│   ├── builder.rs                  # Query builder pattern
│   ├── executor.rs                 # Query execution
│   ├── result.rs                   # QueryResult iterator
│   └── filters.rs                  # Filter operations
│
├── relational/                     # Relational links (feature: relational_keys)
│   ├── mod.rs
│   ├── link.rs                     # RelationalLink type
│   ├── hydration.rs                # Link hydration logic
│   └── fetch_options.rs            # FetchOptions for queries
│
├── blob/                           # Blob storage (feature: blobs)
│   ├── mod.rs
│   ├── types.rs                    # Blob types and traits
│   ├── chunking.rs                 # Chunk management
│   └── item.rs                     # BlobItem trait
│
├── subscription/                   # Subscription system (feature: subscriptions)
│   ├── mod.rs
│   ├── hash.rs                     # Subscription hashing
│   ├── topics.rs                   # Topic management
│   └── registry.rs                 # Subscription registry
│
├── migration/                      # Migration system (feature: migration)
│   ├── mod.rs
│   ├── traits.rs                   # MigrateFrom, MigrateTo
│   ├── context.rs                  # MigrationContext (~200 lines)
│   ├── chain.rs                    # Migration chain (~150 lines)
│   └── versioning.rs               # Version tracking
│
├── libp2p/                         # P2P networking (feature: libp2p)
│   ├── mod.rs
│   ├── traits.rs                   # Libp2pModel, Libp2pStore
│   ├── conversion.rs               # Type conversions
│   └── schema_negotiation.rs       # Schema sync
│
├── node/                           # Node metadata
│   ├── mod.rs
│   └── metadata.rs                 # Node metadata types
│
├── utils/                          # Internal utilities
│   ├── mod.rs
│   └── serde.rs                    # Serde helpers
│
└── doc_examples/                   # Reusable doc test examples
    ├── mod.rs
    ├── simple.rs                   # Basic CRUD examples
    ├── query.rs                    # Query examples
    ├── relational.rs (feature: relational_keys)
    ├── migration.rs (feature: migration)
    └── repository.rs (feature: repository)
```

### 2. Macro Crate Restructure - `netabase_macros/src/`

```
src/
├── lib.rs                          # Public macro exports (~200 lines)
│
├── macros/                         # Macro entry points
│   ├── mod.rs
│   ├── model.rs                    # #[derive(NetabaseModel)]
│   ├── definition.rs               # #[netabase_definition]
│   ├── repository.rs (feature: repository) # #[netabase_repository]
│   ├── blob.rs (feature: blobs)    # #[derive(NetabaseBlobItem)]
│   ├── libp2p.rs (feature: libp2p) # #[netabase_libp2p]
│   ├── cli.rs                      # #[netabase_cli]
│   └── convenience.rs              # #[netabase]
│
├── parsing/                        # AST parsing and validation
│   ├── mod.rs
│   ├── model.rs                    # Parse model structs
│   ├── definition.rs               # Parse definition modules
│   ├── repository.rs               # Parse repository modules
│   ├── field.rs                    # Field parsing
│   ├── attributes.rs               # Attribute extraction (~300 lines)
│   └── validation.rs               # Validation logic
│
├── generators/                     # Code generation
│   ├── mod.rs
│   ├── model/
│   │   ├── mod.rs
│   │   ├── traits.rs               # Model trait impl (~300 lines)
│   │   ├── keys.rs                 # Key enum generation (~300 lines)
│   │   ├── wrappers.rs             # Wrapper type generation
│   │   ├── constructor.rs          # Constructor generation
│   │   ├── serialization.rs        # Serde impl (~250 lines)
│   │   ├── migration.rs (feature: migration) # Migration impl (~250 lines)
│   │   └── libp2p.rs (feature: libp2p) # Libp2p conversion
│   ├── definition/
│   │   ├── mod.rs
│   │   ├── enums.rs                # Definition enum gen (~300 lines)
│   │   ├── traits.rs               # Definition trait impl (~400 lines each file, split)
│   │   │   ├── core.rs
│   │   │   ├── schema.rs
│   │   │   └── conversion.rs
│   │   ├── tree_names.rs           # TreeNames enum
│   │   ├── keys.rs                 # DefKeys enum
│   │   └── subscription.rs (feature: subscriptions) # Subscription capability
│   ├── repository/
│   │   ├── mod.rs
│   │   ├── traits.rs               # Repository trait impl
│   │   ├── store.rs                # Store implementation
│   │   ├── schema.rs               # Schema generation
│   │   ├── discriminant.rs         # Discriminant types
│   │   └── marker.rs               # Marker traits
│   ├── blob/                       # Blob code generation
│   │   ├── mod.rs
│   │   └── item.rs
│   ├── cli/                        # CLI code generation
│   │   ├── mod.rs
│   │   └── commands.rs
│   └── common/                     # Shared generation utilities
│       ├── mod.rs
│       ├── naming.rs               # Naming conventions
│       └── structure.rs            # Common structure helpers
│
└── utils/                          # Shared utilities
    ├── mod.rs
    ├── error.rs                    # Error handling
    ├── schema.rs                   # Schema utilities (~250 lines)
    └── quote.rs                    # Quote helpers
```

### 3. Test Organization - `tests/`

```
tests/
├── common/                         # Shared test utilities
│   ├── mod.rs
│   ├── fixtures.rs                 # Test data fixtures
│   ├── helpers.rs                  # Helper functions
│   └── models.rs                   # Reusable test models
│
├── unit/                           # Unit tests (if needed for integration)
│   └── ...
│
├── integration/                    # Integration tests
│   ├── crud/
│   │   ├── basic.rs                # Basic CRUD operations
│   │   ├── batch.rs                # Batch operations
│   │   └── edge_cases.rs           # Edge cases
│   │
│   ├── query/
│   │   ├── pagination.rs           # Pagination tests
│   │   ├── filtering.rs            # Filter tests
│   │   └── sorting.rs              # Sort tests
│   │
│   ├── indexes/                    # Index tests (feature: secondary_keys)
│   │   ├── secondary_key.rs
│   │   └── lookup_performance.rs
│   │
│   ├── relational/                 # Relational tests (feature: relational_keys)
│   │   ├── links.rs
│   │   ├── hydration.rs
│   │   └── performance.rs
│   │
│   ├── blob/                       # Blob tests (feature: blobs)
│   │   ├── chunking.rs
│   │   ├── complex_types.rs
│   │   └── mixed_content.rs
│   │
│   ├── migration/                  # Migration tests (feature: migration)
│   │   ├── version_chain.rs
│   │   ├── context.rs
│   │   └── comprehensive.rs
│   │
│   ├── repository/                 # Repository tests (feature: repository)
│   │   ├── isolation.rs
│   │   ├── nested_pattern.rs
│   │   ├── toml_generation.rs
│   │   └── comprehensive.rs
│   │
│   ├── subscription/               # Subscription tests (feature: subscriptions)
│   │   ├── selective.rs
│   │   ├── hash.rs
│   │   └── debug.rs
│   │
│   ├── libp2p/                     # P2P tests (feature: libp2p)
│   │   └── schema_sync.rs
│   │
│   ├── database/
│   │   ├── comprehensive.rs        # Full database tests
│   │   ├── growth_analysis.rs      # Growth patterns
│   │   └── size_analysis.rs        # Storage size
│   │
│   └── macros/
│       ├── definition_iter.rs
│       └── record_iter.rs
│
└── doc_tests/                      # Documentation accuracy tests
    ├── readme.rs                   # README examples
    └── migration_docs.rs           # Migration doc examples
```

### 4. Example Crate Organization - `example/`

Rename to `examples_workspace/` or keep as `example/` but clarify purpose:

```
example/
├── Cargo.toml                      # Update description: "Development examples and benchmarks"
├── README.md                       # Document usage for contributors
│
├── src/
│   ├── lib.rs                      # Reusable test models and utilities
│   ├── main.rs                     # Main CLI for exploration
│   └── models/                     # Example model definitions
│       ├── mod.rs
│       ├── basic.rs                # Simple models
│       ├── complex.rs              # Complex relationships
│       └── versioned.rs            # Migration examples
│
├── examples/                       # Runnable examples
│   ├── basic_crud.rs               # Basic usage
│   ├── query_patterns.rs           # Query examples
│   ├── migration_workflow.rs      # Migration example
│   ├── repository_pattern.rs      # Repository example
│   ├── merkle_sync.rs              # Existing
│   └── selective_subscriptions.rs # Existing
│
├── benches/                        # Benchmarks
│   ├── crud.rs
│   ├── blob_chunks.rs
│   ├── stress.rs
│   └── query_performance.rs
│
└── tests/                          # Example-specific tests
    ├── macro_expansion.rs
    ├── schema_export.rs
    ├── schema_import.rs
    └── migration_logic.rs
```

---

## Implementation Strategy

### Phase 1: Preparation (No Breaking Changes)

**Goal**: Set up infrastructure without changing functionality

1. **Add Feature Gates** (~30 files)
   - Add `#[cfg(feature = "...")]` to all optional code
   - Update prelude to conditionally export
   - Ensure crate compiles with all feature combinations

2. **Create New Module Structure** (Keep old)
   - Create new directory structure
   - Add placeholder `mod.rs` files
   - Mark old modules with deprecation notices (internal)

3. **Documentation Infrastructure**
   - Create `doc_examples/` module with reusable examples
   - Set up doc test boilerplate templates
   - Create documentation style guide

4. **Test Infrastructure**
   - Create new `tests/common/` with helpers
   - Create `tests/integration/` directories
   - Add feature-gated test execution

### Phase 2: Core Module Migration

**Goal**: Move code to new structure, one subsystem at a time

1. **Core Types** (Error, Key, Primitives)
   - Move to `core/` module
   - Update imports incrementally
   - Keep old re-exports

2. **Model System**
   - Move model traits to `model/`
   - Split keys into submodules with feature gates
   - Add comprehensive docs

3. **Definition System**
   - Move definition traits to `definition/`
   - Split large files (schema.rs)
   - Add examples

4. **Store System**
   - Reorganize redb into `store/backends/redb/`
   - Split transaction/mod.rs (1648 lines → 5 files)
   - Split crud.rs (1322 lines → 6 files)

5. **Query System**
   - Split query.rs (651 lines → 5 files)
   - Consistent builder pattern

6. **Relational System**
   - Move to `relational/` with feature gate
   - Split into logical submodules

7. **Optional Features**
   - blob/ with feature gate
   - subscription/ with feature gate
   - migration/ with feature gate
   - libp2p/ with feature gate
   - repository/ with feature gate

### Phase 3: Macro Reorganization

1. **Macro Entry Points**
   - Keep lib.rs as thin dispatch layer
   - Move parsing to `parsing/`
   - Move generation to `generators/`

2. **Split Large Files**
   - `generators/definition/traits.rs` (1923 lines → 3 files)
   - `generators/model/traits.rs` (505 lines → 2 files)
   - `utils/attributes.rs` (585 lines → 3 files)

3. **Feature Gates**
   - Add conditional compilation
   - Match main crate features

### Phase 4: Test Migration

1. **Move Integration Tests**
   - One feature area at a time
   - Update imports
   - Add feature gates

2. **Consolidate Similar Tests**
   - Merge `comprehensive_*` tests
   - Remove duplicate coverage
   - Keep only meaningful tests

3. **Example Crate Cleanup**
   - Move benches that should be in main crate
   - Clarify purpose
   - Update documentation

### Phase 5: Documentation

1. **API Documentation**
   - Document all public items
   - Use reusable doc_examples
   - Minimize `ignore` attributes

2. **Module Documentation**
   - Add module-level docs to all modules
   - Include examples in module docs
   - Cross-reference related modules

3. **User Guides**
   - Update README with new structure
   - Create migration guide for users
   - Add architecture documentation

### Phase 6: Cleanup

1. **Remove Old Code**
   - Delete old module locations
   - Remove deprecation warnings
   - Update all imports

2. **Final Validation**
   - Run all tests with all feature combinations
   - Check documentation builds
   - Run clippy with strict lints

3. **Polish**
   - Consistent formatting
   - Consistent naming
   - Final review

---

## Detailed Implementation Rules

### 1. File Size Limits

- **Maximum 400 lines** per file (excluding tests)
- If approaching limit, split into logical submodules
- Exception: Generated code in test fixtures

### 2. Module Organization Patterns

```rust
// Standard module pattern
pub mod submodule;

mod internal;
pub use internal::PublicType;

// Feature-gated modules
#[cfg(feature = "feature_name")]
pub mod feature_module;

// Re-exports in mod.rs
pub use self::submodule::{PublicType, PublicTrait};
```

### 3. Import Conventions

```rust
// In library code - use absolute paths
use crate::core::error::NetabaseError;
use crate::model::traits::NetabaseModel;

// In prelude - group by category
pub use crate::core::{
    error::{NetabaseError, NetabaseResult},
    primitives::*,
};

pub use crate::model::traits::{NetabaseModel, NetabaseModelKeys};

#[cfg(feature = "relational_keys")]
pub use crate::relational::{RelationalLink, FetchOptions};
```

### 4. Feature Gate Patterns

```rust
// In Cargo.toml
[features]
default = ["secondary_keys", "relational_keys"]
secondary_keys = []
relational_keys = []
blobs = []
repository = []
migration = ["dep:toml"]
libp2p = ["dep:libp2p"]
subscriptions = []

// In code - gate entire modules
#[cfg(feature = "relational_keys")]
pub mod relational;

// Gate specific items
#[cfg(feature = "relational_keys")]
pub use crate::relational::RelationalLink;

// Gate trait methods
pub trait NetabaseModel<D> {
    // Always available
    type Keys: NetabaseModelKeys<D, Self>;
    
    #[cfg(feature = "migration")]
    fn version() -> u32;
}

// Gate imports in prelude
#[cfg(feature = "relational_keys")]
pub use crate::relational::*;
```

### 5. Documentation Standards

```rust
//! Module-level documentation.
//!
//! Describe the module's purpose, key concepts, and relationships
//! to other modules.
//!
//! # Examples
//!
//! ```rust
//! use netabase_store::prelude::*;
//! // Minimal working example
//! ```
//!
//! # Feature Flags
//!
//! This module requires the `feature_name` feature.

/// Public item documentation.
///
/// Describe what it does, when to use it, and important details.
///
/// # Examples
///
/// ```rust
/// use netabase_store::prelude::*;
/// # use netabase_store::doc_examples::*;
/// # let (store, _temp) = RedbStore::<ExampleDef>::new_temporary().unwrap();
/// // Use the item
/// ```
///
/// # Errors
///
/// Describe error conditions (for fallible functions).
///
/// # Panics
///
/// Describe panic conditions (if any).
pub fn example_function() {}
```

### 6. Test Organization

```rust
// Integration test structure
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::common::*;

    #[test]
    fn test_specific_behavior() {
        // Arrange
        let fixture = setup_test_store();
        
        // Act
        let result = perform_operation(&fixture);
        
        // Assert
        assert_eq!(result, expected);
    }
}

// Feature-gated tests
#[cfg(all(test, feature = "relational_keys"))]
mod relational_tests {
    // ...
}
```

### 7. Error Handling Patterns

```rust
// Consistent error variants
#[derive(Debug, thiserror::Error)]
pub enum NetabaseError {
    #[error("Model not found: {0}")]
    NotFound(String),
    
    #[error("Serialization failed: {0}")]
    Serialization(#[from] postcard::Error),
    
    #[cfg(feature = "migration")]
    #[error("Migration failed: {0}")]
    Migration(String),
}

// Result type alias
pub type NetabaseResult<T> = Result<T, NetabaseError>;

// Usage
pub fn operation() -> NetabaseResult<Model> {
    // Use ? operator
    let data = read_data()?;
    Ok(process(data)?)
}
```

### 8. Trait Consistency

```rust
// Trait naming
pub trait NetabaseModel<D> { }      // Core trait
pub trait RedbNetbaseModel { }       // Implementation-specific

// Trait bounds consistency
where
    D: NetabaseDefinition,
    Self: NetabaseModel<D>,

// Associated types
type Keys: NetabaseModelKeys<D, Self>;
type Output;

// Methods
fn operation(&self) -> Result<Self::Output, Error>;
```

---

## Migration Checklist

### For Each Module Moved:

- [ ] Create new module structure
- [ ] Add module-level documentation
- [ ] Move code with minimal changes
- [ ] Add feature gates if applicable
- [ ] Update internal imports
- [ ] Add re-export in parent mod.rs
- [ ] Update prelude if public API
- [ ] Add documentation examples
- [ ] Update or create tests
- [ ] Update related documentation
- [ ] Mark old location as deprecated (temporarily)
- [ ] Verify compilation
- [ ] Run tests

### For Each Large File Split:

- [ ] Identify logical boundaries (200-400 lines each)
- [ ] Create submodule directory
- [ ] Create mod.rs with re-exports
- [ ] Move code to separate files
- [ ] Ensure no duplication
- [ ] Update imports
- [ ] Verify functionality unchanged
- [ ] Add tests if missing

### Feature Gate Checklist:

- [ ] Identify all code for feature
- [ ] Add `#[cfg(feature = "...")]` to modules
- [ ] Add `#[cfg(feature = "...")]` to items
- [ ] Update prelude re-exports
- [ ] Add feature to macro crate if needed
- [ ] Update documentation
- [ ] Test with feature enabled
- [ ] Test with feature disabled
- [ ] Test feature combinations

### Documentation Checklist:

- [ ] Module-level docs for all pub modules
- [ ] Item docs for all pub items
- [ ] Examples in module docs
- [ ] Examples in item docs
- [ ] Use doc_examples module for boilerplate
- [ ] Minimize `ignore` attributes
- [ ] Test examples with `cargo test --doc`
- [ ] Cross-reference related items

---

## Success Criteria

### Code Quality
- ✅ No file exceeds 400 lines (excluding test fixtures)
- ✅ All public items documented
- ✅ All modules have module-level docs
- ✅ Consistent patterns across similar features
- ✅ No warnings with `cargo clippy`
- ✅ No unused imports or dead code

### Functionality
- ✅ All tests pass
- ✅ All feature combinations compile
- ✅ No performance regressions
- ✅ Existing examples still work

### Documentation
- ✅ `cargo doc` builds without warnings
- ✅ `cargo test --doc` passes
- ✅ README updated with new structure
- ✅ Migration guide for users (if needed)
- ✅ Less than 5% of examples use `ignore`

### Organization
- ✅ Clear module hierarchy
- ✅ Feature-gated code properly isolated
- ✅ Test organization matches source structure
- ✅ Examples organized by purpose
- ✅ No circular dependencies

---

## Timeline Estimate

- **Phase 1 (Preparation)**: 2-3 days
- **Phase 2 (Core Migration)**: 5-7 days
- **Phase 3 (Macro Reorganization)**: 3-4 days
- **Phase 4 (Test Migration)**: 2-3 days
- **Phase 5 (Documentation)**: 3-4 days
- **Phase 6 (Cleanup)**: 1-2 days

**Total**: 16-23 days for complete reorganization

---

## Risk Mitigation

### Risks:
1. **Breaking changes** - Users may have internal dependencies
2. **Performance regression** - Module reorganization could affect inlining
3. **Lost functionality** - Code could be accidentally removed
4. **Test coverage gaps** - Tests might not cover all paths

### Mitigations:
1. Keep old re-exports during transition with deprecation warnings
2. Run benchmarks before and after each phase
3. Use git branches for each phase, extensive review before merge
4. Measure test coverage before and after (use `cargo tarpaulin` or similar)
5. Create comprehensive integration tests before starting

---

## Notes

- This is a **non-functional refactor** - behavior must not change
- Focus on **maintainability** and **developer experience**
- All changes should be **incremental** and **reviewable**
- Each phase should leave the codebase in a working state
- Documentation is **first-class** - not an afterthought

---

## Decisions Made

1. **`traits::registery` → `traits::registry`**
   - ✅ **DECISION**: Fix typo during refactor
   
2. **Example crate naming**
   - ✅ **DECISION**: Keep name, clarify in documentation
   
3. **Internal utilities module**
   - ✅ **DECISION**: Use `utils/` for internal, mark with clear docs
   
4. **Feature flag granularity**
   - ✅ **DECISION**: Keep crate-level features, add **macro-level detection**
   - Macros will detect unused features per-model and generate minimal code
   - Users can opt-out of granular features via attributes (default: status quo)
   - See: `CONDITIONAL_CODEGEN_ANALYSIS.md`
   
5. **Architecture documentation**
   - ✅ **DECISION**: Add `ARCHITECTURE.md` to workspace root

## Additional Enhancements

6. **Test Dependencies** (See: `EXAMPLES_ARCHITECTURE.md`)
   - Use fixture-based approach for sequential tests
   - Build scripts for test data generation
   - Idempotent setup functions
   
7. **Composable Examples** (See: `EXAMPLES_ARCHITECTURE.md`)
   - Base models shared across examples
   - Feature-gated examples (`#![cfg(feature = "...")]`)
   - Combination tests for feature interaction
   - Progressive learning path (00_minimal → 07_libp2p)
   
8. **Feature-Gated Benchmarks** (See: `BENCHMARK_ARCHITECTURE.md`)
   - Benchmarks require matching features via `required-features`
   - Separate benchmarks for each feature
   - Combination benchmarks for overhead measurement
   - CI integration for regression tracking
   
9. **Conditional Code Generation** (See: `CONDITIONAL_CODEGEN_ANALYSIS.md`)
   - Phase 1: Macro detection with placeholder types (backward compatible)
   - Phase 2: Trait refinement for true optional features (v2.0)
   - 10-40% compile time improvement for simple models
   - Zero runtime overhead
   
10. **Memory Backend** (See: `MEMORY_BACKEND_DESIGN.md`)
    - `ByteVecBackend` for generic byte storage
    - `TypedHashMapBackend` for type-safe storage (redb-like)
    - Adapter layer to bridge to existing traits
    - Validates architecture decoupling
    - Fast tests without disk I/O
