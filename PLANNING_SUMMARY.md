# Netabase Store Refactoring - Planning Summary

## Overview

This document provides a quick reference to all planning documentation for the comprehensive reorganization of netabase_store.

## Planning Documents

### 1. [REFACTOR_PLAN.md](./REFACTOR_PLAN.md) - Master Refactoring Plan
**Purpose:** Complete reorganization strategy for both `netabase_store` and `netabase_macros`

**Key Sections:**
- Current state analysis (file sizes, module organization)
- Proposed architecture (module restructure)
- 6-phase implementation strategy
- Detailed implementation rules
- Success criteria and timeline

**Timeline:** 16-23 days for complete reorganization

**Priority Changes:**
1. Break down 1600+ line files into ~400 line modules
2. Fix `traits::registery` → `traits::registry` typo
3. Add comprehensive feature gates
4. Reorganize test structure
5. Improve documentation coverage

---

### 2. [ARCHITECTURE.md](./ARCHITECTURE.md) - System Architecture
**Purpose:** High-level overview of system design and components

**Key Sections:**
- Design philosophy and principles
- Core concepts (Definition, Model, Repository)
- Architecture layers (User → Backend)
- Type system and guarantees
- Feature system matrix
- Backend abstraction
- Code generation pipeline
- Performance model

**Use Cases:**
- Onboarding new contributors
- Understanding design decisions
- Planning new features
- System-level refactoring

---

### 3. [EXAMPLES_ARCHITECTURE.md](./EXAMPLES_ARCHITECTURE.md) - Composable Examples
**Purpose:** Restructure examples to be modular, feature-gated, and educational

**Key Innovations:**
- **Base models** shared across examples
- **Feature-gated examples** (`#![cfg(feature = "...")]`)
- **Progressive learning** (00_minimal → 07_libp2p)
- **Combination tests** for feature interaction

**Structure:**
```
example/
├── src/models/          # Shared base models
│   ├── base.rs          # No features
│   ├── secondary.rs     # + secondary_keys
│   └── complete.rs      # All features
├── examples/
│   ├── 00_minimal.rs    # No features
│   ├── 01_secondary_keys.rs
│   └── combinations/    # Feature interaction
└── benches/             # Feature-gated benchmarks
```

**Benefits:**
- Users see features incrementally
- CI tests all feature combinations
- Examples validate feature interaction
- Clear documentation path

---

### 4. [BENCHMARK_ARCHITECTURE.md](./BENCHMARK_ARCHITECTURE.md) - Performance Testing
**Purpose:** Feature-gated, composable benchmarking system

**Key Features:**
- **Modular benchmarks** measuring specific operations
- **Feature-gated compilation** (`required-features = [...]`)
- **Shared baseline** for comparison
- **CI integration** for regression detection

**Structure:**
```
benches/
├── core/
│   ├── crud.rs              # No features
│   └── serialization.rs     # Base operations
├── features/
│   ├── secondary_keys.rs    # #[cfg(feature = "secondary_keys")]
│   └── blobs.rs             # #[cfg(feature = "blobs")]
└── combinations/
    └── full_featured.rs     # Overhead measurement
```

**Running:**
```bash
# Base benchmarks
cargo bench --bench crud --no-default-features

# Feature-specific
cargo bench --bench secondary_keys --features secondary_keys

# Compare performance
cargo bench -- --save-baseline main
```

---

### 5. [CONDITIONAL_CODEGEN_ANALYSIS.md](./CONDITIONAL_CODEGEN_ANALYSIS.md) - Smart Code Generation
**Purpose:** Reduce generated code size by detecting unused features

**Question Answered:** Can macros conditionally generate code based on actual feature usage?

**Answer:** ✅ **Yes, recommended approach:**

**Phase 1 (Immediate):**
- Macros detect which features each model uses
- Generate minimal placeholders for unused features
- Backward compatible (no API changes)

```rust
// Before: Always generates full infrastructure
pub enum UserBlob { Field1(Vec<u8>), Field2(Vec<u8>) }
pub enum UserSecondaryKeys { Email(String), Name(String) }

// After: Detects no blobs/secondary keys
pub enum UserBlob { __NoBlobs(()) }
pub enum UserSecondaryKeys { __NoSecondaryKeys(()) }
```

**Phase 2 (Long-term):**
- Trait redesign with opt-in features
- True optional associated types

**Impact:**
- 10-40% faster compilation for simple models
- 30% reduction in generated code size
- Zero runtime overhead
- Minor security improvement (smaller attack surface)

**Decision:** ✅ Implement in Phase 2 of refactor

---

### 6. [MEMORY_BACKEND_DESIGN.md](./MEMORY_BACKEND_DESIGN.md) - Backend Abstraction
**Purpose:** Validate architecture decoupling and enable fast testing

**Answer:** ✅ **Yes, highly recommended!**

**Implementations:**

1. **ByteVecBackend** - Generic byte storage
   ```rust
   // Pure byte-level storage
   BTreeMap<String, BTreeMap<Vec<u8>, Vec<u8>>>
   ```

2. **TypedHashMapBackend** - Type-safe storage (redb-like)
   ```rust
   // Type-erased hashmaps with runtime type checking
   HashMap<(TableName, TypeId), Box<dyn Any>>
   ```

3. **Adapter Layer** - Bridge to existing traits
   ```rust
   impl NBStore<D> for MemoryStore<D> { /* ... */ }
   ```

**Benefits:**
- **Fast tests** - No disk I/O (100x faster)
- **Decoupling proof** - Validates trait abstraction
- **Development speed** - Instant startup
- **Future backends** - Path to IndexedDB, SQLite, etc.

**Architecture:**
```
User Code
    ↓
NBStore trait
    ↓
┌──────────┬──────────┬──────────┐
│   Redb   │  Memory  │ IndexedDB│
└──────────┴──────────┴──────────┘
```

**Decision:** ✅ Implement memory backend (High Priority)

---

## Test Organization Strategy

### Sequential Test Dependencies

**Problem:** Some tests depend on others (e.g., import schema → export schema)

**Solution: Fixture-Based Approach**

```rust
// tests/common/fixtures.rs
pub fn ensure_schema_toml() -> PathBuf {
    let path = PathBuf::from("test_data/schema.toml");
    if !path.exists() {
        generate_schema_toml(&path);
    }
    path
}

// tests/schema_import.rs
#[test]
fn test_import() {
    let schema = ensure_schema_toml(); // Idempotent!
    // test import
}
```

**Benefits:**
- Parallel-safe (fixtures are idempotent)
- No single-threaded requirement
- Fast test execution
- Deterministic results

**Alternative: Build Script**
```rust
// build.rs
fn main() {
    generate_test_fixtures();
}
```

---

## Implementation Roadmap

### Phase 1: Preparation (2-3 days)
✅ **Decisions Made:**
- Fix `registery` → `registry`
- Keep example crate name
- Use `utils/` for internal code
- Add macro-level feature detection
- Create ARCHITECTURE.md

**Tasks:**
1. Add feature gates throughout codebase
2. Create fixture infrastructure for tests
3. Set up doc_examples module
4. Create new directory structure (parallel to old)

### Phase 2: Core Migration (5-7 days)
**Tasks:**
1. Move core types (error, key, primitives)
2. Split large files (transaction, crud, query)
3. Reorganize model/definition/repository traits
4. Add comprehensive documentation
5. Implement memory backend

### Phase 3: Macro Reorganization (3-4 days)
**Tasks:**
1. Split large generator files
2. Add feature detection logic
3. Implement conditional code generation
4. Add feature gates to macro crate

### Phase 4: Test Migration (2-3 days)
**Tasks:**
1. Reorganize tests by feature
2. Create composable test fixtures
3. Add feature-gated test execution
4. Consolidate duplicate tests

### Phase 5: Documentation (3-4 days)
**Tasks:**
1. Document all public APIs
2. Create reusable doc examples
3. Add module-level documentation
4. Update README and guides

### Phase 6: Cleanup (1-2 days)
**Tasks:**
1. Remove old code
2. Final validation (all feature combos)
3. Run clippy and fix warnings
4. Performance benchmarks

**Total: 16-23 days**

---

## Success Metrics

### Code Quality
- ✅ No file >400 lines (excluding test fixtures)
- ✅ All public items documented
- ✅ Zero clippy warnings
- ✅ <5% doc examples use `ignore`

### Functionality
- ✅ All tests pass
- ✅ All feature combinations compile
- ✅ No performance regressions
- ✅ Examples work

### Organization
- ✅ Clear module hierarchy
- ✅ Feature-gated code isolated
- ✅ Tests match source structure
- ✅ No circular dependencies

---

## Quick Reference

### Run Examples
```bash
# Minimal
cargo run --example 00_minimal --no-default-features

# With feature
cargo run --example 01_secondary_keys --features secondary_keys

# Combination
cargo run --example relational_blob --features relational_keys,blobs
```

### Run Benchmarks
```bash
# Base
cargo bench --bench crud --no-default-features

# Feature-specific
cargo bench --bench secondary_keys --features secondary_keys

# All
cargo bench
```

### Run Tests by Feature
```bash
# No features
cargo test --no-default-features

# Specific feature
cargo test --features secondary_keys

# All features
cargo test
```

### Check Feature Combinations
```bash
# Test all combinations
for features in "" "secondary_keys" "relational_keys,blobs" "migration,repository"; do
    echo "Testing: $features"
    cargo check --no-default-features --features "$features"
done
```

---

## Key Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Fix typo | `registry` | Correct spelling |
| Example name | Keep as `example` | Clarify in docs |
| Internal utils | Use `utils/` | Mark clearly |
| Feature detection | ✅ Macro-level | 10-40% compile time savings |
| Backend abstraction | ✅ Memory backend | Fast tests, decoupling |
| Test dependencies | Fixture-based | Parallel-safe |
| Benchmark gating | ✅ Feature-gated | Isolate overhead |
| Code generation | Conditional placeholders | Backward compatible |

---

## Files Generated

1. `REFACTOR_PLAN.md` - Master plan
2. `ARCHITECTURE.md` - System overview
3. `EXAMPLES_ARCHITECTURE.md` - Example organization
4. `BENCHMARK_ARCHITECTURE.md` - Benchmark structure
5. `CONDITIONAL_CODEGEN_ANALYSIS.md` - Code generation optimization
6. `MEMORY_BACKEND_DESIGN.md` - Backend abstraction
7. `PLANNING_SUMMARY.md` - This document

---

## Next Steps

1. **Review plans** - Get team/community feedback
2. **Create tracking issue** - GitHub issue with checklist
3. **Set up branches** - Feature branches for each phase
4. **Begin Phase 1** - Infrastructure setup
5. **Incremental PRs** - Small, reviewable changes

---

## Questions?

- Architecture questions → See `ARCHITECTURE.md`
- Implementation details → See `REFACTOR_PLAN.md`
- Example structure → See `EXAMPLES_ARCHITECTURE.md`
- Performance concerns → See `BENCHMARK_ARCHITECTURE.md`
- Code generation → See `CONDITIONAL_CODEGEN_ANALYSIS.md`
- Backend design → See `MEMORY_BACKEND_DESIGN.md`

---

*Planning completed: 2026-02-01*
*Ready for implementation*
