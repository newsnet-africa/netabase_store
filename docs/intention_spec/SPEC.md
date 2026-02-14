# Intention-Driven TODO Language (IDTL) v3.5: Formal Specification

## 1. Grammar Specification
The grammar is designed to be parseable by both humans and machines in a single pass.

`TODO: [INTENTION:] <Category><Action>[Target][domain](<Subject>) <State><SideEffect> [<Reason>(<Description>)]`

### Components (in order):
- **[INTENTION:]**: High-level context (e.g., `SYNC:`, `FIX:`, `AUDIT:`) (Optional)
- **Category+Action**: `TR`, `MD`, `QL`, etc. (Required)
- **Target**: `N` (New) or `E` (Existing). (Optional for Queries/Audits)
- **domain**: `a` (API), `c` (Concrete), `s` (Schema). (Optional for Queries/Audits)
- **Subject**: `(Symbol/Context)` (Required)
- **State**: `!`, `?`, `*` (Optional)
- **SideEffect**: `^M`, `^C` (Optional)
- **Reason+Description**: `[P](Reduce allocs)` (Optional)

---

## 2. Core Property Enums

### [INTENTION] (The High-Level "Why")
- **SYNC**: Aligning code across crate boundaries (e.g., Core vs. Macro).
- **FIX**: Repairing a bug or incorrect implementation.
- **FEAT**: Adding new functionality or types.
- **REFAC**: Structural cleanup without changing logic.
- **AUDIT**: Review, discovery, or security investigation. Requires high semantic nuance.

### [Target] (Uppercase Suffix)
- **N: New**. The item is being created. (LSP: Collision detection).
- **E: Existing**. The item already exists and is being modified. (LSP: Symbol resolution).
- *(Optional for Queries/Audits)*

### [domain] (lowercase Suffix)

- **a: API Shape / Syntax Construct**. Changing code structure (e.g., `struct -> enum`, `fn -> const`, `trait method signature`).

- **c: Concrete Implementation / Type Instance**. Changing specific types or logic (e.g., `String -> Cow`, `u32 -> i64`).

- **s: Serialization Schema / Data Layer**. Changing persisted structures (TOML/JSON layouts).

- **r: Reference / Relationship**. Interactions, call graphs, or module dependencies.

- *(Optional for Queries/Audits)*



---



## 3. Category Tree & Usage Examples



### T: Type/Signature (API Contracts)

- **TR: Return Type** [Target, domain]

  - *Ex*: `TRNa(Result<T>)^M` (Update Macro to emit a New API Shape: Result wrapper).

- **TA: Argument Type** [Target, domain]

  - *Ex*: `TANa(&str)` (Add a New API argument of type &str).

  - *Ex*: `AUDIT: TAEa(u32?(u64))` (Query: Why use u32 instead of u64 for this Existing API argument?).

- **TL: Lifetimes & Generics**

  - *Ex*: `TLa('a, T: Archive)` (Add a lifetime and bound to a generated AST).

- **TO: Ownership & Memory**

  - *Ex*: `TOc(String -> Cow<'a>)*` (Change Concrete ownership to Cow; Verified).

- **TW: Wrapper Type**

  - *Ex*: `TWNc(ModelId)` (Create a New Concrete wrapper type).



### M: Move/Relocate (Refactoring)

- **MD: Destination** [Target, domain]

  - *Ex*: `MDNa(struct -> enum)^M` (Macro: Move logic from struct to a New Enum shape).

  - *Ex*: `MDEc(utils::db)` (Move this Concrete item to the Existing utils::db module).

- **MN: Rename** [Target: N, domain]

  - *Ex*: `MNNc(Registry)` (Rename this Concrete item to Registry).



### U: Update/Fix (Internal Refinement)

- **UR: Reason**. General logic updates or fixes.

  - *Ex*: `UR[m](Fix field emission)!` (Critical: Fix Macro Bug in code emission).

- **UI: Interface**. Internal-only trait or method changes.

  - *Ex*: `UI[i](Align with v2)` (Update internal logic to match new interface).



### Q: Quality & Audit (Semantic Nuance)

Used to flag specific qualities of code for review. These are highly composable with the `AUDIT:` intention.

- **QL: Logic & Flow**. Control flow, algorithmic correctness, edge cases.

  - *Ex*: `AUDIT: QLc(Pool)? [B]` (Is this Pool logic sound? Seems like a bad implementation).

- **QS: Security & Safety**. `unsafe` usage, race conditions, memory safety, or bounds.

  - *Ex*: `AUDIT: QSc(Send + Sync)?` (Verify if these bounds are actually safe here).

- **QP: Performance & Scale**. Latency, allocations, complexity, or resource usage.

  - *Ex*: `AUDIT: QPc(BTreeMap)? [P]` (Is BTreeMap the right choice for high-frequency inserts?).

- **QD: Data & State**. State transitions, serialization integrity, or persistence.

  - *Ex*: `AUDIT: QDs(Version)?` (Ensure state doesn't corrupt during migration).

- **QT: Type & Contract**. Ergonomics, trait bounds, or signature design "vibe".

  - *Ex*: `AUDIT: QTa(TR)? [i]` (Query why the Return Type has this specific API shape).

- **QU: Usage & Ergonomics**. Auditing how an API or type is used by consumers.

  - *Ex*: `AUDIT: QUc(Builder)?` (Is this Builder pattern actually ergonomic for users?).

- **QR: Relationships**. Call graphs, dependencies, and interactions.

  - *Ex*: `AUDIT: QRc(User calls? db.save)` (Why is this call happening here?).



### G: Generator Logic (Macro Internals)

- **GE: Emission** | **GI: Implementation** | **GP: Parsing** | **GV: Validation**

  - *Ex*: `GIa(NetabaseModel)` (Refactor how the macro implements the NetabaseModel trait).

  - *Decision*: If output changes, use **U**; if only internal logic changes, use **G**.



---



## 4. Lifecycle & Reasoning



### TODO Lifecycle States

- `?` → **Inquiry**: "Why is this here?" (Discovery/Audit phase).

- `*` → **Vetted**: Logic approved; ready for implementation or verified safe.

- `!` → **Critical**: Urgent priority; blocks release or indicates a failed audit.

- `✓` → **Resolved**: Task complete (removed from code).



### Audit Semantics & Composability

Audits are the most nuanced part of IDTL. They can be composed in two ways:

1. **Structural Audit**: Use `AUDIT:` + a structural category (e.g., `TR`, `MD`) to query *structure*.

   - *Ex*: `AUDIT: TR(T)?` (Query the Return Type structure).

2. **Quality Audit**: Use `AUDIT:` + a `Q` category (e.g., `QS`, `QP`) to query *properties*.

   - *Ex*: `AUDIT: QP(TR)?` (Audit the Performance of the Return Type).



### Targeted Question Syntax (The "Moving ?")



For complex queries, the `?` can be moved inside the subject to pinpoint the inquiry. This shorthand is available for all categories.



- **Subject Query**: `Subject?(Alternative)` queries the choice of the subject itself.



  - *Ex*: `AUDIT: TREa(M?(impl SomeTrait))`: "Why use generic M here instead of impl SomeTrait?"

  - *Ex*: `AUDIT: TAEa(u32?(u64))`: "Why use u32 here instead of u64 for this argument?"



- **Action Query**: `(Subject ->? Target)` queries the *relationship/action* itself.



  - *Ex*: `QRc(Builder call? quick_sort)`: "Why is it calling this?"



- **Target Query**: `(Subject -> Target?)` queries the *target* choice.



  - *Ex*: `QRc(Builder call quick_sort?)`: "Why quick_sort specifically?"



- **Alternative Query**: `(Subject -> Target?(Alternative))` proposes an alternative.



  - *Ex*: `QRc(Builder call bubble_sort?(quick_sort))`: "Why bubble_sort instead of quick_sort?"







> **Note on Composability**: You do not need special "Question Categories" for everything. Use the `AUDIT:` intention with standard structural categories.



> - `AUDIT: TREa(...)` is the standard way to query a Return Type.



> - `Q` categories (like `QT`, `QP`) are for auditing *qualities* (Performance, Ergonomics) that don't map to a single structural element.







### Reason Codes & Descriptions
Reason codes provide justification and can include an optional **Description**: `[Code](Description)`.

- `[L]`: Logical Refinement. | `[B]`: Bad Implementation.
- `[P]`: Performance. | `[S]`: Security/Safety.
- `[m]`: Macro Bug. | `[i]`: Interface Lag/Design.
- `[M]`: Moved to Macro. | `[C]`: Moved to Core.

---

## 5. IDTL Blocks (Technical Roadmaps)

```rust
/* IDTL_START: REFAC: MDNa(struct -> enum)^M [L]
 * Status: IN_PROGRESS
 * Goal: Refactor hardcoded Metadata struct into flexible Enum.
 * Pre:  struct Metadata { title: String, ... }
 * Post: enum Metadata { Article {...}, Video {...} }
 * Dep:  netabase_store/src/traits/mod.rs:TODO_ID_123
 * Verify: cargo test test_metadata_variants
 * IDTL_END
 */
```

---

## 6. Real-World Comparison Table

| IDTL Code | Plain English Equivalent |
| :--- | :--- |
| `TODO: SYNC: TRNa(Result<T>)^M!` | Add a new Result wrapper to the return type; update macro; breaking change. |
| `TODO: FEAT: MNNc(UserRegistry)` | Add feature by renaming this concrete type to UserRegistry. |
| `TODO: SYNC: TRNa(Wrapper<Self>)^M [i]` | Trait changed in Core; update Macro to emit `Wrapper` to fix interface lag. |
| `TODO: AUDIT: QLc(Pool)? [B]` | Reviewing concrete Pool logic; seems like a bad implementation. |

---

## 7. The "TODO Compiler" & LSP Vision
- **Ghost TODOs**: When a trait changes (`TRNa`), the LSP injects virtual TODOs into all implementors.
- **Pre/Post Check**: Compiler fails if `Post:` condition (e.g., `enum`) isn't met upon resolution.
- **Dependency Graph**: The compiler ensures that a TODO chain (Core -> Macro -> Consumer) is completed in order.
- **Symbol Guard**: Detects if a `New (N)` target already exists or an `Existing (E)` target is missing.

---

## 8. Implementation Roadmap
- **Phase 1 (Manual)**: Use `grep` patterns.
- **Phase 2 (CLI)**: `cargo-todo` tool for syntax validation.
- **Phase 3 (LSP)**: Autocomplete, hover docs, and "Side-Effect Teleportation."
- **Phase 4 (Compiler)**: CI enforcement of Pre/Post conditions and Graph completion.
