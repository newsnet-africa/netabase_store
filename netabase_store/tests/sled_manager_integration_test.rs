// Integration tests for Sled-backed definition manager
//
// These tests verify that the Sled manager implementation works correctly
// with multi-definition transactions and permission checking.
//
// ============================================================================
// ⚠️  TEMPORARILY DISABLED ⚠️
// ============================================================================
// These tests require full trait implementations for TestDefinitions,
// TestManager, and TestPermissions, which would require thousands of lines
// of boilerplate code (the exact problem we're solving!).
//
// These tests will be re-enabled in Phase 9 (Testing) once the proc macros
// are functional and can generate the required boilerplate.
//
// TODO: Re-enable these tests after implementing:
//   - Phase 3: Per-Model Structure Generation
//   - Phase 4: Per-Model Trait Implementations
//   - Phase 5: Per-Definition Structures
//   - Phase 6: Backend-Specific Implementations (Sled)
//   - Phase 7: TreeManager Implementation
//
// The tests will then use #[derive(NetabaseModel)] and
// #[netabase_definition_module(...)] to generate the required code.
// ============================================================================

// Original tests preserved but commented out - will be restored when macros are ready
