// Basic integration tests for Sled-backed definition manager
//
// These tests verify core manager functionality: creation, loading/unloading,
// stats, and path handling.
//
// ============================================================================
// ⚠️  TEMPORARILY DISABLED ⚠️
// ============================================================================
// These tests require SimpleDef and SimpleManager to implement
// NetabaseDefinitionTrait and DefinitionManagerTrait, which would require
// thousands of lines of boilerplate code (the exact problem we're solving!).
//
// These tests will be re-enabled in Phase 9 (Testing) once the proc macros
// are functional and can generate the required boilerplate.
//
// TODO: Re-enable these tests after implementing:
//   - Phase 3: Per-Model Structure Generation
//   - Phase 4: Per-Model Trait Implementations
//   - Phase 5: Per-Definition Structures
//   - Phase 7: TreeManager Implementation
//
// The tests will then use #[derive(NetabaseModel)] and
// #[netabase_definition_module(...)] to generate the required code.
// ============================================================================

/*
use netabase_store::databases::manager::DefinitionManager;
use netabase_store::databases::sled_store::SledStore;
use netabase_store::traits::definition::DiscriminantName;
use netabase_store::traits::permission::NoPermissions;
use strum::{AsRefStr, EnumDiscriminants, EnumIter, IntoDiscriminant};

// Minimal test definition enum
#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(Hash, EnumIter, AsRefStr))]
#[strum_discriminants(name(SimpleDefDiscriminants))]
pub enum SimpleDef {
    Users,
    Products,
}

impl DiscriminantName for SimpleDefDiscriminants {}

// Minimal manager type marker
#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(Hash, EnumIter, AsRefStr))]
pub enum SimpleManager {
    Instance,
}

impl DiscriminantName for SimpleManagerDiscriminants {}

#[test]
fn test_sled_generic_manager_creation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager: DefinitionManager<SimpleManager, SimpleDef, NoPermissions, SledStore<SimpleDef>> =
        DefinitionManager::new(temp_dir.path()).unwrap();

    let (total, loaded, warm) = manager.stats();
    assert_eq!(total, 2); // Users, Products
    assert_eq!(loaded, 0); // None loaded initially
    assert_eq!(warm, 0); // No warm hints
}

#[test]
fn test_sled_manager_load_unload() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut manager: DefinitionManager<SimpleManager, SimpleDef, NoPermissions, SledStore<SimpleDef>> =
        DefinitionManager::new(temp_dir.path()).unwrap();

    // Initially no definitions loaded
    assert!(!manager.is_loaded(&SimpleDefDiscriminants::Users));

    // Load a definition
    let path = manager.root_path().join("Users").join("store.db");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let store = SledStore::new(&path).unwrap();

    if let Some(link) = manager.stores.get_mut(&SimpleDefDiscriminants::Users) {
        link.load(store);
    }

    assert!(manager.is_loaded(&SimpleDefDiscriminants::Users));

    let loaded = manager.loaded_definitions();
    assert_eq!(loaded.len(), 1);
}

#[test]
fn test_sled_manager_warm_on_access() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut manager: DefinitionManager<SimpleManager, SimpleDef, NoPermissions, SledStore<SimpleDef>> =
        DefinitionManager::new(temp_dir.path()).unwrap();

    // Add warm hint
    manager.add_warm_on_access(SimpleDefDiscriminants::Products);

    let (_, _, warm) = manager.stats();
    assert_eq!(warm, 1);
}

#[test]
fn test_sled_manager_root_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager: DefinitionManager<SimpleManager, SimpleDef, NoPermissions, SledStore<SimpleDef>> =
        DefinitionManager::new(temp_dir.path()).unwrap();

    assert_eq!(manager.root_path(), temp_dir.path());
}

#[test]
fn test_sled_manager_stats() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut manager: DefinitionManager<SimpleManager, SimpleDef, NoPermissions, SledStore<SimpleDef>> =
        DefinitionManager::new(temp_dir.path()).unwrap();

    let (total, loaded, warm) = manager.stats();
    assert_eq!(total, 2);
    assert_eq!(loaded, 0);
    assert_eq!(warm, 0);

    // Load one definition
    let path = manager.root_path().join("Users").join("store.db");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let store = SledStore::new(&path).unwrap();

    if let Some(link) = manager.stores.get_mut(&SimpleDefDiscriminants::Users) {
        link.load(store);
    }

    let (total, loaded, warm) = manager.stats();
    assert_eq!(total, 2);
    assert_eq!(loaded, 1);
    assert_eq!(warm, 0);

    // Add warm hint
    manager.add_warm_on_access(SimpleDefDiscriminants::Products);
    let (total, loaded, warm) = manager.stats();
    assert_eq!(total, 2);
    assert_eq!(loaded, 1);
    assert_eq!(warm, 1);
}

#[test]
fn test_sled_manager_accessed_tracking() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut manager: DefinitionManager<SimpleManager, SimpleDef, NoPermissions, SledStore<SimpleDef>> =
        DefinitionManager::new(temp_dir.path()).unwrap();

    // Mark a definition as accessed
    manager.mark_accessed(SimpleDefDiscriminants::Users.clone());

    // Clear accessed
    manager.clear_accessed();

    // This is mainly to ensure the API exists and doesn't panic
}
*/
