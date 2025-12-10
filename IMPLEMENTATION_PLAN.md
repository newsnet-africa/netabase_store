# Netabase Proc-Macro Crate Implementation Plan

## Overview

Implement a proc-macro crate to generate netabase boilerplate code, eliminating 5000+ lines of manual code per definition (94% reduction).

### User-Requested Syntax

```rust
#[netabase_definition_module(DefinitionName, DefinitionKeys, subscriptions(Topic1, Topic2))]
pub mod definitions {
    #[derive(NetabaseModel)]
    #[subscribe(Topic1)]
    pub struct SomeModel {
        #[primary_key]
        this_id: Uuid,
        #[secondary_key]
        email: String,
        #[relation]
        some_other: RelationalLink<SomeOther>,
    }

    // Nested definitions (independent but namespaced)
    #[netabase_definition_module(InnerDefinition, InnerDefinitionKeys)]
    pub mod inner_definition {
        #[derive(NetabaseModel)]
        pub struct InnerModel {
            #[primary_key]
            id: String,
        }
    }
}
```

### Design Decisions (from user clarifications)

1. **Cross-definition linking**: Support BOTH explicit types AND `#[cross_definition_link(path)]` attribute
2. **Nested definitions**: Independent definitions, namespaced together. Parent/child relationships determine cross-definition permissions
3. **Subscriptions**: Module-level declaration required (models can only subscribe to declared topics)
4. **Backend support**: Always generate both Redb and Sled implementations

## Boilerplate Generated Per Model

### Structures (13+ items):
1. Primary key wrapper (e.g., `UserId(u64)`)
2. Secondary key wrappers (one per `#[secondary_key]` field)
3. `ModelSecondaryKeys` enum
4. `ModelSecondaryKeysDiscriminants` (via strum)
5. `ModelSecondaryTreeNames` enum
6. `ModelSecondaryKeysIter` iterator
7. `ModelRelationalKeys` enum
8. `ModelRelationalKeysDiscriminants`
9. `ModelRelationalKeysIter` iterator
10. `ModelSubscriptions` enum
11. `ModelSubscriptionsDiscriminants`
12. `ModelSubscriptionTreeNames` enum
13. `ModelKeys` unified wrapper

### Trait Implementations (4 per model):
1. `NetabaseModelKeyTrait<Definition, Model>`
2. `NetabaseModelTrait<Definition>`
3. `RedbNetabaseModelTrait<Definition>`
4. `SledNetabaseModelTrait<Definition>`

### Per Definition:
1. `Definition` enum (wraps all models)
2. `DefinitionKeys` enum
3. `DefinitionModelAssociatedTypes` mega-enum (8 variants per model)
4. `ModelAssociatedTypesExt<Definition>` impl
5. `RedbModelAssociatedTypesExt<Definition>` impl (massive pattern matches)
6. `SledModelAssociatedTypesExt<Definition>` impl
7. `NetabaseDefinitionTrait` impl
8. `TreeManager<Definition>` impl

## Implementation Phases

### Phase 1: Workspace Setup (Week 1) ✅ COMPLETED

**Goal**: Convert single package to workspace with proc-macro crate

**Tasks**:
1. Create workspace root `Cargo.toml`
2. Move `netabase_store` to subdirectory
3. Create `netabase_macros/` proc-macro crate
4. Set up dependencies: `syn`, `quote`, `proc-macro2`, `darling`

**Directory Structure**:
```
netabase_store/
├── Cargo.toml              # Workspace manifest
├── netabase_store/         # Main library
│   └── Cargo.toml
└── netabase_macros/        # Proc-macro crate
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── parse/          # Parsing modules
        ├── generate/       # Code generation
        └── utils/
```

**Dependencies**:
```toml
[dependencies]
syn = { version = "2.0", features = ["full", "extra-traits"] }
quote = "1.0"
proc-macro2 = "1.0"
darling = "0.20"  # Simplifies attribute parsing
```

### Phase 2: Parsing Infrastructure with Syn Visitors (Weeks 2-3) ✅ COMPLETED

**Goal**: Parse module and model structures using syn visitors for maintainable metadata collection

**Status**: ✅ All parsing infrastructure implemented with visitor pattern (17 tests passing)

**Files**:
- `src/parse/module.rs` - Module visitor and context builder
- `src/parse/model.rs` - Model visitor for struct analysis
- `src/parse/visitors.rs` - Syn visitor implementations
- `src/parse/attributes.rs` - Parse field attributes using darling
- `src/parse/metadata.rs` - Metadata structures

**Key Design: Syn Visitor Pattern**

Use `syn::visit::Visit` to walk the AST and collect metadata:

```rust
use syn::visit::{self, Visit};

// Metadata structures
#[derive(Debug, Default)]
struct ModuleMetadata {
    definition_name: syn::Ident,
    keys_name: syn::Ident,
    available_subscriptions: Vec<syn::Ident>,
    models: Vec<ModelMetadata>,
    nested_modules: Vec<ModuleMetadata>,
}

#[derive(Debug)]
struct ModelMetadata {
    name: syn::Ident,
    vis: syn::Visibility,
    fields: Vec<FieldMetadata>,
    subscriptions: Vec<syn::Ident>,
}

#[derive(Debug)]
struct FieldMetadata {
    name: syn::Ident,
    ty: syn::Type,
    is_primary_key: bool,
    is_secondary_key: bool,
    is_relation: bool,
    cross_definition_link: Option<syn::Path>,
}

// Visitor for collecting model metadata
struct ModelVisitor {
    metadata: ModelMetadata,
    errors: Vec<syn::Error>,
}

impl<'ast> Visit<'ast> for ModelVisitor {
    fn visit_field(&mut self, field: &'ast syn::Field) {
        let mut field_meta = FieldMetadata {
            name: field.ident.clone().unwrap(),
            ty: field.ty.clone(),
            is_primary_key: false,
            is_secondary_key: false,
            is_relation: false,
            cross_definition_link: None,
        };

        // Parse attributes
        for attr in &field.attrs {
            if attr.path().is_ident("primary_key") {
                field_meta.is_primary_key = true;
            } else if attr.path().is_ident("secondary_key") {
                field_meta.is_secondary_key = true;
            } else if attr.path().is_ident("relation") {
                field_meta.is_relation = true;
            } else if attr.path().is_ident("cross_definition_link") {
                // Parse cross_definition_link path
                if let Ok(path) = attr.parse_args::<syn::Path>() {
                    field_meta.cross_definition_link = Some(path);
                }
            }
        }

        // Validate: only one of primary/secondary/relation
        let count = [
            field_meta.is_primary_key,
            field_meta.is_secondary_key,
            field_meta.is_relation,
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        if count > 1 {
            self.errors.push(syn::Error::new_spanned(
                field,
                "Field can only have one of: #[primary_key], #[secondary_key], #[relation]",
            ));
        }

        self.metadata.fields.push(field_meta);
        visit::visit_field(self, field);
    }
}

// Visitor for collecting module metadata
struct ModuleVisitor {
    metadata: ModuleMetadata,
    errors: Vec<syn::Error>,
}

impl<'ast> Visit<'ast> for ModuleVisitor {
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        // Check if struct has #[derive(NetabaseModel)]
        let has_netabase_model = item.attrs.iter().any(|attr| {
            if attr.path().is_ident("derive") {
                if let Ok(meta) = attr.parse_args::<syn::Path>() {
                    return meta.is_ident("NetabaseModel");
                }
            }
            false
        });

        if has_netabase_model {
            // Use ModelVisitor to collect field metadata
            let mut model_visitor = ModelVisitor {
                metadata: ModelMetadata {
                    name: item.ident.clone(),
                    vis: item.vis.clone(),
                    fields: Vec::new(),
                    subscriptions: Vec::new(),
                },
                errors: Vec::new(),
            };

            model_visitor.visit_item_struct(item);

            // Collect errors
            self.errors.extend(model_visitor.errors);

            // Validate: exactly one primary key
            let primary_key_count = model_visitor
                .metadata
                .fields
                .iter()
                .filter(|f| f.is_primary_key)
                .count();

            if primary_key_count == 0 {
                self.errors.push(syn::Error::new_spanned(
                    item,
                    "Model must have exactly one field marked with #[primary_key]",
                ));
            } else if primary_key_count > 1 {
                self.errors.push(syn::Error::new_spanned(
                    item,
                    "Model can only have one #[primary_key] field",
                ));
            }

            self.metadata.models.push(model_visitor.metadata);
        }

        visit::visit_item_struct(self, item);
    }

    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        // Check if nested module has #[netabase_definition_module]
        let has_definition_attr = module.attrs.iter().any(|attr| {
            attr.path().is_ident("netabase_definition_module")
        });

        if has_definition_attr {
            // Recursively parse nested module
            // Store in self.metadata.nested_modules
        }

        visit::visit_item_mod(self, module);
    }
}
```

**Benefits of Visitor Pattern**:
1. **Maintainability**: Add new features by extending visitor methods
2. **Separation of concerns**: Parsing logic separate from generation logic
3. **Composability**: Can combine multiple visitors
4. **Error collection**: Accumulate all errors before failing
5. **Tree walking**: Automatic traversal of nested structures

**Validation Strategy**:
- Collect all metadata first using visitors
- Validate after collection (allows reporting multiple errors at once)
- Build ModuleMetadata tree with all necessary information
- Pass metadata to code generators

**Error Handling with Visitors**:
```rust
// Collect errors during visitation
struct ErrorCollector {
    errors: Vec<syn::Error>,
}

impl ErrorCollector {
    fn validate_primary_keys(&mut self, model: &ModelMetadata, span: Span) {
        let count = model.fields.iter().filter(|f| f.is_primary_key).count();
        if count != 1 {
            self.errors.push(syn::Error::new(
                span,
                format!("Expected exactly 1 primary key, found {}", count),
            ));
        }
    }

    fn validate_subscriptions(
        &mut self,
        model: &ModelMetadata,
        available: &[syn::Ident],
    ) {
        for sub in &model.subscriptions {
            if !available.contains(sub) {
                self.errors.push(syn::Error::new_spanned(
                    sub,
                    format!(
                        "Subscription '{}' not declared. Available: {:?}",
                        sub, available
                    ),
                ));
            }
        }
    }

    fn into_result(self) -> Result<(), syn::Error> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            // Combine all errors
            let mut combined = self.errors.into_iter();
            let mut first = combined.next().unwrap();
            for err in combined {
                first.combine(err);
            }
            Err(first)
        }
    }
}
```

### Phase 3: Per-Model Structure Generation (Weeks 4-5) ✅ COMPLETED

**Goal**: Generate all wrapper types and enums for each model

**Status**: ✅ All model structure generators implemented (31 tests passing)

**Files**:
- `src/generate/model/primary_key.rs` - Primary key wrapper + traits
- `src/generate/model/secondary_keys.rs` - Secondary key wrappers, enum, discriminants, iterator
- `src/generate/model/relational_keys.rs` - Relational key enum, discriminants, iterator
- `src/generate/model/subscription_keys.rs` - Subscription enum, discriminants
- `src/generate/model/value_impl.rs` - `redb::Value` implementation for model

**Code Generation Pattern** (based on actual boilerplate):
```rust
fn generate_primary_key_wrapper(
    model_name: &syn::Ident,
    field_type: &syn::Type,
) -> TokenStream {
    let wrapper_name = format_ident!("{}Id", model_name);
    let type_name_str = wrapper_name.to_string();

    quote! {
        #[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
        pub struct #wrapper_name(pub #field_type);

        // Redb Value implementation (delegates to inner type)
        impl redb::Value for #wrapper_name {
            type SelfType<'a> = #wrapper_name;
            type AsBytes<'a> = <#field_type as redb::Value>::AsBytes<'a>;

            fn fixed_width() -> Option<usize> {
                <#field_type as redb::Value>::fixed_width()
            }

            fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> {
                Self(<#field_type as redb::Value>::from_bytes(data))
            }

            fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
                <#field_type as redb::Value>::as_bytes(&value.0)
            }

            fn type_name() -> redb::TypeName {
                redb::TypeName::new(#type_name_str)
            }
        }

        // Redb Key implementation (delegates to inner type)
        impl redb::Key for #wrapper_name {
            fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                <#field_type as redb::Key>::compare(data1, data2)
            }
        }

        // Bincode conversions for Sled backend
        impl TryFrom<Vec<u8>> for #wrapper_name {
            type Error = bincode::error::DecodeError;
            fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
                let (value, _): (#field_type, usize) =
                    bincode::decode_from_slice(&data, bincode::config::standard())?;
                Ok(#wrapper_name(value))
            }
        }

        impl TryFrom<#wrapper_name> for Vec<u8> {
            type Error = bincode::error::EncodeError;
            fn try_from(value: #wrapper_name) -> Result<Self, Self::Error> {
                bincode::encode_to_vec(value.0, bincode::config::standard())
            }
        }
    }
}
```

**For each model generates**:
- Primary key wrapper (e.g., `UserId`) with redb::Value, redb::Key, and TryFrom implementations
- Secondary key wrappers (e.g., `UserEmail`, `UserName`) with same trait implementations
- `ModelSecondaryKeys` enum with:
  - Derives: `Debug, Clone, EnumDiscriminants, Encode, Decode`
  - Variants containing wrapped values (e.g., `Email(UserEmail)`)
  - `redb::Value` and `redb::Key` implementations
  - TryFrom conversions for Sled backend
- `ModelSecondaryKeysDiscriminants` (via strum EnumDiscriminants):
  - Derives: `Hash, AsRefStr, Encode, Decode, EnumIter`
  - `DiscriminantName` trait implementation (delegates to AsRefStr)
- `ModelSecondaryTreeNames` enum:
  - No inner data, purely for tree identification
  - Derives: `Debug, Clone, Copy, Hash, Eq, PartialEq, EnumIter, AsRefStr`
  - `DiscriminantName` trait implementation
- `ModelSecondaryKeysIter` iterator wrapper
- Same pattern for relational keys (RelationalKeys, RelationalKeysDiscriminants, RelationalTreeNames, RelationalKeysIter)
- Same pattern for subscriptions (Subscriptions, SubscriptionsDiscriminants, SubscriptionTreeNames)

### Phase 4: Per-Model Trait Implementations (Weeks 6-7) ✅ COMPLETED

**Goal**: Generate trait implementations for each model

**Status**: ✅ NetabaseModelTrait generator implemented with hash computation

**Files**:
- `src/generate/model/key_trait.rs` - `NetabaseModelKeyTrait` impl
- `src/generate/model/model_trait.rs` - `NetabaseModelTrait` impl (8 wrapping methods + hash computation)
- `src/generate/model/redb_trait.rs` - `RedbNetabaseModelTrait` impl
- `src/generate/model/sled_trait.rs` - `SledNetabaseModelTrait` impl

**NetabaseModelTrait Implementation**:
```rust
impl NetabaseModelTrait<Definitions> for User {
    type Keys = UserKeys;
    const MODEL_TREE_NAME: DefinitionsDiscriminants = DefinitionsDiscriminants::User;
    type SecondaryKeys = UserSecondaryKeysIter;
    type RelationalKeys = UserRelationalKeysIter;
    type SubscriptionEnum = UserSubscriptions;
    type Hash = [u8; 32];

    fn primary_key(&self) -> UserId {
        UserId(self.id)
    }

    fn compute_hash(&self) -> Self::Hash {
        // Generate blake3 hash of all fields
    }

    // 8 wrapping methods (wrap_primary_key, wrap_model, etc.)
}
```

**Hash Computation Logic**:
- Detect field types and generate appropriate serialization
- Primitives: `to_le_bytes()`
- Strings: `as_bytes()`
- Complex types: bincode serialization
- Hash all fields in order using blake3

**Model redb::Value Implementation** (Phase 3):
The model struct itself needs a custom `redb::Value` implementation that serializes all fields:
```rust
impl Value for User {
    type SelfType<'a> = User;
    type AsBytes<'a> = Cow<'a, [u8]>;

    fn fixed_width() -> Option<usize> {
        None  // Variable-width for structs with String fields
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> {
        // Custom deserialization: read fields in order
        // Example: id(8 bytes) + age(4 bytes) + email_len(8 bytes) + email + name
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
        // Custom serialization: write fields in order
        // Use to_le_bytes() for primitives, length-prefixed strings
    }

    fn type_name() -> TypeName {
        TypeName::new("ModelName")
    }
}
```
Note: Fixed-width types can optimize with `fixed_width() -> Some(N)`. Variable-width types (with String/Vec) must use `None` and length prefixes.

### Phase 5: Per-Definition Structures (Weeks 8-9) ✅ COMPLETED

**Goal**: Generate definition-level enums and mega-structures

**Status**: ✅ All definition-level structures implemented including tree naming (38 tests passing)

**Files**:
- `src/generate/definition/enum.rs` - Definition enum
- `src/generate/definition/keys.rs` - DefinitionKeys enum
- `src/generate/definition/associated_types.rs` - DefinitionModelAssociatedTypes mega-enum (8 variants per model)
- `src/generate/definition/associated_types_ext.rs` - ModelAssociatedTypesExt impl

**DefinitionModelAssociatedTypes**:
```rust
#[derive(Debug, Clone)]
pub enum DefinitionModelAssociatedTypes {
    // 8 variants per model:
    UserPrimaryKey(UserId),
    UserModel(User),
    UserSecondaryKey(UserSecondaryKeys),
    UserRelationalKey(UserRelationalKeys),
    UserSubscriptionKey(UserSubscriptions),
    UserSecondaryKeyDiscriminant(UserSecondaryKeysDiscriminants),
    UserRelationalKeyDiscriminant(UserRelationalKeysDiscriminants),
    UserSubscriptionKeyDiscriminant(UserSubscriptionsDiscriminants),

    // Repeat for each model
    DefinitionKey(DefinitionKeys),
}
```

### Phase 6: Backend-Specific Implementations (Weeks 10-11) ✅ COMPLETED

**Goal**: Generate massive pattern-matching implementations for Redb and Sled

**Status**: ✅ Both Redb and Sled extension trait generators fully implemented and tested (42/42 tests passing)

**Files**:
- `src/generate/definition/backend_extensions.rs` - Both `RedbModelAssociatedTypesExt` and `SledModelAssociatedTypesExt` implementations

**Challenge**: Generate 5+ methods with N×M match arms (N models, M operations)

**RedbModelAssociatedTypesExt Methods**:
1. `insert_model_into_redb` - N match arms
2. `insert_secondary_key_into_redb` - N match arms
3. `insert_relational_key_into_redb` - N match arms
4. `insert_hash_into_redb` - N match arms
5. `insert_subscription_into_redb` - N match arms
6. `delete_model_from_redb` - N match arms
7. `delete_subscription_from_redb` - N match arms

**Code Generation Strategy**:
```rust
fn generate_redb_insert_model_arms(models: &[ModelMetadata]) -> TokenStream {
    let arms = models.iter().map(|model| {
        let model_name = &model.name;
        let pk_type = format_ident!("{}Id", model_name);
        quote! {
            (
                DefinitionModelAssociatedTypes::#model_name Model(model),
                DefinitionModelAssociatedTypes::#model_name PrimaryKey(pk),
            ) => {
                let table_def: TableDefinition<#pk_type, #model_name> =
                    TableDefinition::new(table_name);
                let mut table = txn.open_table(table_def)?;
                table.insert(pk, model)?;
                Ok(())
            }
        }
    });
    quote! {
        match (self, key) {
            #(#arms,)*
            _ => Err(NetabaseError::Other("Type mismatch".into())),
        }
    }
}
```

### Phase 7: TreeManager Implementation (Week 12) ✅ COMPLETED

**Goal**: Generate TreeManager trait implementation

**Status**: ✅ TreeManager trait generator fully implemented and tested (45/45 tests passing)

**File**: `src/generate/definition/tree_manager.rs`

**Generates**:
```rust
impl TreeManager<Definitions> for Definitions {
    fn all_trees() -> AllTrees<Definitions> {
        AllTrees::new()
    }

    fn get_tree_name(discriminant: &DefinitionsDiscriminants) -> Option<String> {
        match discriminant {
            DefinitionsDiscriminants::User => Some("User".to_string()),
            // ... for each model
        }
    }

    fn get_secondary_tree_names(discriminant: &DefinitionsDiscriminants) -> Vec<String> {
        match discriminant {
            DefinitionsDiscriminants::User => vec!["User_Email".to_string(), /* ... */],
            // ... for each model
        }
    }

    fn get_relational_tree_names(/*...*/) -> Vec<String> { /* ... */ }
    fn get_subscription_tree_names(/*...*/) -> Vec<String> { /* ... */ }
}
```

### Phase 8: Nested Definitions & Permissions (Weeks 13-14)

**Goal**: Support nested modules and infer permissions from hierarchy

**Nested Module Processing**:
- Each nested module generates its own complete definition
- Parent can access child by default
- Sibling access requires explicit permission
- Generate permission enums based on module structure

**Cross-Definition Linking**:

**Explicit approach** (user provides full type):
```rust
#[relation]
inner_link: OtherDefinitionLinkEnum<InnerDef, InnerModel, Store>
```

**Attribute approach** (macro generates):
```rust
#[cross_definition_link(inner::InnerModel)]
inner_link: OuterModelInnerModelLink  // Generated wrapper type
```

**Generated wrapper**:
```rust
pub struct OuterModelInnerModelLink {
    pub definition: InnerDef,
    pub model: InnerModel,
}
```

### Phase 9: Testing (Weeks 15-16)

**Goal**: Comprehensive test coverage

**Test Files**:
- `tests/unit_tests.rs` - Test each code generator in isolation
- `tests/integration_tests.rs` - Test full macro expansion
- `tests/edge_cases.rs` - Empty models, no secondary keys, nested definitions
- `tests/comparison_tests.rs` - Compare with manual boilerplate behavior

**Example Integration Test**:
```rust
#[test]
fn test_simple_definition() {
    #[netabase_definition_module(TestDef, TestDefKeys)]
    mod test_def {
        #[derive(NetabaseModel)]
        pub struct TestModel {
            #[primary_key]
            id: u64,
            #[secondary_key]
            name: String,
        }
    }

    let model = test_def::TestModel { id: 1, name: "test".to_string() };
    let pk = model.primary_key();
    assert_eq!(pk.0, 1);
}
```

**Edge Cases**:
- Model without secondary keys
- Model without relations
- Nested definitions
- Invalid subscription topics (should error)
- Multiple primary keys (should error)

### Phase 10: Error Handling & Diagnostics (Week 17)

**Goal**: Clear, actionable error messages with proper spans

**Error Message Examples**:
```rust
// Missing primary key
"Model must have exactly one field marked with #[primary_key]\n\
 \n\
 Example:\n\
 #[derive(NetabaseModel)]\n\
 pub struct User {\n\
     #[primary_key]\n\
     id: u64,\n\
 }"

// Invalid subscription
"Subscription topic 'Unknown' not declared in module attribute\n\
 \n\
 Declared topics: [\"Updates\", \"Premium\"]\n\
 \n\
 Fix: Add 'Unknown' to subscriptions(...) or remove from #[subscribe(...)]"
```

### Phase 11: Documentation (Week 18)

**Goal**: Comprehensive rustdoc and examples

**Documentation**:
- Macro documentation with examples
- Field attribute documentation
- Migration guide from manual boilerplate
- Troubleshooting guide

**Example Projects**:
1. `examples/simple.rs` - Basic single-model definition
2. `examples/multi_model.rs` - Multiple models with relations
3. `examples/nested_definitions.rs` - Nested module structure
4. `examples/cross_definition.rs` - Cross-definition linking
5. `examples/subscriptions.rs` - Subscription features

### Phase 12: Integration & Migration (Weeks 19-20)

**Goal**: Integrate with existing codebase and provide migration path

**Integration Testing**:
- Test macros with actual `RedbStore` and `SledStore`
- Verify CRUD operations work correctly
- Validate hash computation matches manual implementation
- Ensure generated code compiles with existing traits

**Migration Guide** (`MIGRATION.md`):
- Step-by-step conversion from manual boilerplate
- Before/after code examples
- Breaking changes (if any)
- Testing recommendations

## Critical Files

### Reference Files (to understand requirements):
1. `/home/rusta/Projects/NewsNet/netabase_store/examples/boilerplate.rs` - 5181 lines of manual boilerplate
2. `/home/rusta/Projects/NewsNet/netabase_store/src/traits/model/mod.rs` - `NetabaseModelTrait` definition
3. `/home/rusta/Projects/NewsNet/netabase_store/src/traits/definition/mod.rs` - `NetabaseDefinitionTrait`, `ModelAssociatedTypesExt`
4. `/home/rusta/Projects/NewsNet/netabase_store/src/databases/redb_store/traits.rs` - `RedbNetabaseModelTrait`, `RedbModelAssociatedTypesExt`
5. `/home/rusta/Projects/NewsNet/netabase_store/src/databases/sled_store/traits.rs` - `SledNetabaseModelTrait`, `SledModelAssociatedTypesExt`
6. `/home/rusta/Projects/NewsNet/netabase_store/src/traits/store/tree_manager.rs` - `TreeManager` trait

### Files to Create:
1. `/home/rusta/Projects/NewsNet/Cargo.toml` - Workspace manifest
2. `/home/rusta/Projects/NewsNet/netabase_macros/Cargo.toml` - Proc-macro crate
3. `/home/rusta/Projects/NewsNet/netabase_macros/src/lib.rs` - Macro entry points

## Success Metrics

1. **Code Reduction**: 94% reduction (from ~860 lines/model to ~50 lines/model)
2. **Correctness**: Generated code behavior matches manual boilerplate
3. **Developer Experience**: Clear errors, comprehensive docs
4. **Performance**: Generated code performance equals manual code

## Timeline

**Total**: 20 weeks (5 months)

**Milestones**:
- **Week 4**: MVP - Basic single model generation works
- **Week 8**: Alpha - All single-definition features work
- **Week 14**: Beta - Nested definitions and permissions work
- **Week 18**: RC - Full test coverage and documentation
- **Week 20**: Release - Production ready

## Implementation Priority

### Must Have (MVP - Weeks 1-4):
1. Workspace setup
2. Module & model parsing
3. Primary key wrapper generation
4. Secondary keys generation
5. Basic trait implementations
6. Definition enum generation
7. Simple integration tests

### Should Have (Alpha - Weeks 5-11):
8. Relational keys
9. Subscription keys
10. Redb backend implementation
11. Sled backend implementation
12. TreeManager implementation

### Nice to Have (Beta - Weeks 12-20):
13. Nested definitions
14. Permission inference
15. Cross-definition linking
16. Comprehensive error messages
17. Documentation & examples
18. Migration guide
