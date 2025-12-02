//! Introspection implementation for SledStore

use crate::error::NetabaseError;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::introspection::{DatabaseIntrospection, TreeInfo, TreeType};
use super::SledStore;

impl<D> DatabaseIntrospection<D> for SledStore<D>
where
    D: NetabaseDefinitionTrait + crate::traits::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    fn list_all_trees(&self) -> Result<Vec<TreeInfo>, NetabaseError> {
        let mut trees = Vec::new();

        // Get all tree names from sled
        for name_result in self.db.tree_names() {
            let name_bytes = name_result;
            let name = String::from_utf8_lossy(&name_bytes).to_string();

            // Determine tree type based on name
            let tree_type = classify_tree_type(&name);

            // Try to get entry count and size
            let (entry_count, size_bytes) = if let Ok(tree) = self.db.open_tree(&name_bytes) {
                let count = tree.len();
                // Note: size_on_disk() is only available on sled::Db, not individual trees
                (Some(count), None)
            } else {
                (None, None)
            };

            trees.push(TreeInfo {
                name,
                tree_type,
                entry_count,
                size_bytes,
            });
        }

        Ok(trees)
    }

    fn tree_entry_count(&self, tree_name: &str) -> Result<usize, NetabaseError> {
        let tree = self.db.open_tree(tree_name)?;
        Ok(tree.len())
    }

    fn tree_keys_raw(&self, tree_name: &str) -> Result<Vec<Vec<u8>>, NetabaseError> {
        let tree = self.db.open_tree(tree_name)?;
        let mut keys = Vec::new();

        for item in tree.iter() {
            let (key, _) = item?;
            keys.push(key.to_vec());
        }

        Ok(keys)
    }

    fn tree_contents_raw(&self, tree_name: &str) -> Result<Vec<(Vec<u8>, Vec<u8>)>, NetabaseError> {
        let tree = self.db.open_tree(tree_name)?;
        let mut contents = Vec::new();

        for item in tree.iter() {
            let (key, value) = item?;
            contents.push((key.to_vec(), value.to_vec()));
        }

        Ok(contents)
    }
}

/// Classify a tree by its name
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_tree_type() {
        assert_eq!(classify_tree_type("User"), TreeType::PrimaryModel);
        assert_eq!(classify_tree_type("Post"), TreeType::PrimaryModel);
        assert_eq!(classify_tree_type("User_secondary"), TreeType::SecondaryIndex);
        assert_eq!(classify_tree_type("Post_secondary"), TreeType::SecondaryIndex);
        assert_eq!(classify_tree_type("__libp2p_providers"), TreeType::LibP2PProviders);
        assert_eq!(classify_tree_type("__libp2p_provided"), TreeType::LibP2PProvided);
        assert_eq!(classify_tree_type("__sled__default"), TreeType::System);
    }
}
