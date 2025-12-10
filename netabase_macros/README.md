# Netabase Macros

Procedural macros for generating netabase boilerplate code.

## Overview

This crate provides derive macros and attribute macros that eliminate ~94% of boilerplate code required for netabase definitions. Instead of manually writing thousands of lines of wrapper types, enums, and trait implementations, you can define your models with simple annotations.

## Status

🚧 **Under Development** - Phase 1 Complete (Workspace Setup)

## Planned Features

- `#[derive(NetabaseModel)]` - Generate all boilerplate for a model
- `#[netabase_definition_module(...)]` - Generate definition-level structures
- Support for primary keys, secondary keys, and relational keys
- Automatic generation of Redb and Sled backend implementations
- Nested definitions with permission inference
- Cross-definition linking

## Usage (Planned)

```rust
use netabase_macros::{NetabaseModel, netabase_definition_module};

#[netabase_definition_module(UserDefinition, UserDefinitionKeys)]
pub mod user_definition {
    #[derive(NetabaseModel)]
    pub struct User {
        #[primary_key]
        id: u64,
        #[secondary_key]
        email: String,
        #[secondary_key]
        username: String,
        name: String,
        age: u32,
    }
}
```

## Implementation Phases

See [IMPLEMENTATION_PLAN.md](../IMPLEMENTATION_PLAN.md) for the complete implementation roadmap.

- [x] Phase 1: Workspace Setup
- [ ] Phase 2: Parsing Infrastructure
- [ ] Phase 3: Per-Model Structure Generation
- [ ] Phase 4: Per-Model Trait Implementations
- [ ] Phase 5: Per-Definition Structures
- [ ] Phase 6: Backend-Specific Implementations
- [ ] Phase 7: TreeManager Implementation
- [ ] Phase 8: Nested Definitions & Permissions
- [ ] Phase 9: Testing
- [ ] Phase 10: Error Handling & Diagnostics
- [ ] Phase 11: Documentation
- [ ] Phase 12: Integration & Migration

## Development

```bash
# Build the workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Check macro expansion (requires cargo-expand)
cargo expand -p netabase_macros
```

## License

MIT OR Apache-2.0
