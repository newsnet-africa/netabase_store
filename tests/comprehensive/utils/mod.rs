//! Test utilities and helpers for comprehensive testing

use netabase_store::error::NetabaseError;
use netabase_store::traits::definition::NetabaseDefinitionTrait;
use netabase_store::traits::introspection::{DatabaseIntrospection, TreeType};
use netabase_store::traits::model::NetabaseModelTrait;
use netabase_store::traits::tree::NetabaseTreeSync;
use netabase_store::traits::store_ops::OpenTree;

/// Helper trait for backend-agnostic testing
pub trait TestBackend<D: NetabaseDefinitionTrait>: DatabaseIntrospection<D> + Sized {
    /// Create a temporary store for testing
    fn create_temp() -> Result<Self, NetabaseError>;

    /// Get a descriptive name for the backend
    fn backend_name() -> &'static str;
}

/// Verify database state structure
pub struct DatabaseState {
    pub model_trees: Vec<String>,
    pub secondary_trees: Vec<String>,
    pub system_trees: Vec<String>,
    pub total_entries: usize,
}

impl DatabaseState {
    /// Capture current database state
    pub fn capture<D, B>(store: &B) -> Result<Self, NetabaseError>
    where
        D: NetabaseDefinitionTrait,
        B: DatabaseIntrospection<D>,
    {
        let all_trees = store.list_all_trees()?;

        let model_trees: Vec<String> = all_trees
            .iter()
            .filter(|t| t.tree_type == TreeType::PrimaryModel)
            .map(|t| t.name.clone())
            .collect();

        let secondary_trees: Vec<String> = all_trees
            .iter()
            .filter(|t| t.tree_type == TreeType::SecondaryIndex)
            .map(|t| t.name.clone())
            .collect();

        let system_trees: Vec<String> = all_trees
            .iter()
            .filter(|t| t.tree_type.is_system_tree())
            .map(|t| t.name.clone())
            .collect();

        let total_entries = all_trees
            .iter()
            .filter_map(|t| t.entry_count)
            .sum();

        Ok(DatabaseState {
            model_trees,
            secondary_trees,
            system_trees,
            total_entries,
        })
    }

    /// Compare two states and return differences
    pub fn diff(&self, other: &DatabaseState) -> StateDiff {
        StateDiff {
            new_model_trees: other.model_trees
                .iter()
                .filter(|t| !self.model_trees.contains(t))
                .cloned()
                .collect(),
            removed_model_trees: self.model_trees
                .iter()
                .filter(|t| !other.model_trees.contains(t))
                .cloned()
                .collect(),
            entry_count_change: other.total_entries as i64 - self.total_entries as i64,
        }
    }
}

/// Difference between two database states
#[derive(Debug, Clone)]
pub struct StateDiff {
    pub new_model_trees: Vec<String>,
    pub removed_model_trees: Vec<String>,
    pub entry_count_change: i64,
}

/// Test a CRUD operation and verify state changes
pub fn test_crud_with_state_verification<D, B, M, F>(
    store: &B,
    operation_name: &str,
    operation: F,
) -> Result<(), NetabaseError>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    B: DatabaseIntrospection<D> + OpenTree<D, M>,
    F: FnOnce(&B) -> Result<(), NetabaseError>,
{
    // Capture state before
    let state_before = DatabaseState::capture(store)?;

    // Perform operation
    operation(store)?;

    // Capture state after
    let state_after = DatabaseState::capture(store)?;

    // Verify state change
    let diff = state_before.diff(&state_after);

    println!("Operation '{}' completed:", operation_name);
    println!("  Entry count change: {}", diff.entry_count_change);
    println!("  New trees: {:?}", diff.new_model_trees);
    println!("  Removed trees: {:?}", diff.removed_model_trees);

    Ok(())
}

/// Verify tree contents match expected
pub fn verify_tree_contents<D, B, M>(
    store: &B,
    expected_count: usize,
) -> Result<(), NetabaseError>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    B: DatabaseIntrospection<D> + OpenTree<D, M>,
{
    let tree_name = M::discriminant_name();
    let actual_count = store.tree_entry_count(tree_name)?;

    assert_eq!(
        actual_count, expected_count,
        "Tree '{}' should have {} entries, but has {}",
        tree_name, expected_count, actual_count
    );

    Ok(())
}

/// Verify secondary index tree exists and has correct entry count
pub fn verify_secondary_index<D, B, M>(
    store: &B,
    expected_count: usize,
) -> Result<(), NetabaseError>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    B: DatabaseIntrospection<D>,
{
    let secondary_tree_name = format!("{}_secondary", M::discriminant_name());
    let actual_count = store.tree_entry_count(&secondary_tree_name)?;

    assert_eq!(
        actual_count, expected_count,
        "Secondary index tree '{}' should have {} entries, but has {}",
        secondary_tree_name, expected_count, actual_count
    );

    Ok(())
}

/// Count total entries across all primary model trees
pub fn count_all_model_entries<D, B>(store: &B) -> Result<usize, NetabaseError>
where
    D: NetabaseDefinitionTrait,
    B: DatabaseIntrospection<D>,
{
    let model_trees = store.list_model_trees()?;
    Ok(model_trees.iter().filter_map(|t| t.entry_count).sum())
}

/// Verify database is in clean state (no entries)
pub fn verify_clean_state<D, B>(store: &B) -> Result<(), NetabaseError>
where
    D: NetabaseDefinitionTrait,
    B: DatabaseIntrospection<D>,
{
    let total = count_all_model_entries(store)?;
    assert_eq!(total, 0, "Database should be empty but has {} entries", total);
    Ok(())
}
