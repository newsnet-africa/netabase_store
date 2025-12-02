//! Introspection implementation for RedbStoreZeroCopy

use crate::error::NetabaseError;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::introspection::{DatabaseIntrospection, TreeInfo, TreeType};
use super::RedbStoreZeroCopy;
use redb::{ReadableDatabase, ReadableTable, ReadableTableMetadata};

impl<D> DatabaseIntrospection<D> for RedbStoreZeroCopy<D>
where
    D: NetabaseDefinitionTrait,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant + strum::IntoEnumIterator,
{
    fn list_all_trees(&self) -> Result<Vec<TreeInfo>, NetabaseError> {
        let mut trees = Vec::new();

        // Use existing tree_names() method to get all model discriminants
        for discriminant in self.tree_names() {
            let name = discriminant.as_ref().to_string();
            let tree_type = TreeType::PrimaryModel;

            // Get entry count for primary tree
            let entry_count = self.tree_entry_count(&name).ok();

            trees.push(TreeInfo {
                name: name.clone(),
                tree_type,
                entry_count,
                size_bytes: None,
            });

            // Add corresponding secondary index tree
            let secondary_name = format!("{}_secondary", name);
            let secondary_count = self.tree_entry_count(&secondary_name).ok();

            trees.push(TreeInfo {
                name: secondary_name,
                tree_type: TreeType::SecondaryIndex,
                entry_count: secondary_count,
                size_bytes: None,
            });
        }

        // Check for libp2p trees if they exist
        #[cfg(feature = "libp2p")]
        {
            if let Ok(count) = self.tree_entry_count("__libp2p_providers") {
                trees.push(TreeInfo {
                    name: "__libp2p_providers".to_string(),
                    tree_type: TreeType::LibP2PProviders,
                    entry_count: Some(count),
                    size_bytes: None,
                });
            }
            if let Ok(count) = self.tree_entry_count("__libp2p_provided") {
                trees.push(TreeInfo {
                    name: "__libp2p_provided".to_string(),
                    tree_type: TreeType::LibP2PProvided,
                    entry_count: Some(count),
                    size_bytes: None,
                });
            }
        }

        Ok(trees)
    }

    fn tree_entry_count(&self, tree_name: &str) -> Result<usize, NetabaseError> {
        let txn = self.db().begin_read()?;
        let table_def: redb::TableDefinition<&[u8], &[u8]> = redb::TableDefinition::new(tree_name);
        let table = txn.open_table(table_def)?;
        Ok(table.len()? as usize)
    }

    fn tree_keys_raw(&self, tree_name: &str) -> Result<Vec<Vec<u8>>, NetabaseError> {
        let txn = self.db().begin_read()?;
        let table_def: redb::TableDefinition<&[u8], &[u8]> = redb::TableDefinition::new(tree_name);
        let table = txn.open_table(table_def)?;

        let mut keys = Vec::new();
        for item in table.iter()? {
            let (key, _): (redb::AccessGuard<&[u8]>, redb::AccessGuard<&[u8]>) = item?;
            keys.push(key.value().to_vec());
        }

        Ok(keys)
    }

    fn tree_contents_raw(&self, tree_name: &str) -> Result<Vec<(Vec<u8>, Vec<u8>)>, NetabaseError> {
        let txn = self.db().begin_read()?;
        let table_def: redb::TableDefinition<&[u8], &[u8]> = redb::TableDefinition::new(tree_name);
        let table = txn.open_table(table_def)?;

        let mut contents = Vec::new();
        for item in table.iter()? {
            let (key, value): (redb::AccessGuard<&[u8]>, redb::AccessGuard<&[u8]>) = item?;
            contents.push((key.value().to_vec(), value.value().to_vec()));
        }

        Ok(contents)
    }
}

/// Classify a tree by its name
#[allow(dead_code)] // Reserved for future tree classification logic
fn classify_tree_type(name: &str) -> TreeType {
    if name == "__libp2p_providers" {
        TreeType::LibP2PProviders
    } else if name == "__libp2p_provided" {
        TreeType::LibP2PProvided
    } else if name.ends_with("_secondary") {
        TreeType::SecondaryIndex
    } else if name.starts_with("__") {
        TreeType::System
    } else {
        TreeType::PrimaryModel
    }
}
