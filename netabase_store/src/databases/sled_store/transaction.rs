//! Transaction implementations for sled backend
//!
//! This module provides read and write transaction wrappers that adapt sled's
//! API to Netabase's trait-based transaction interface.

use crate::{
    databases::sled_store::{
        SledStore,
        SledNetabaseModelTrait,
        serialize_key,
        serialize_value,
        deserialize_key,
        deserialize_value,
    },
    error::NetabaseResult,
    traits::{
        definition::{DiscriminantName, NetabaseDefinition},
        model::{NetabaseModelTrait, key::NetabaseModelKeyTrait},
    },
};
use log::{trace, debug};
use std::fmt::Debug;
use std::time::Instant;
use strum::{IntoDiscriminant, IntoEnumIterator};

// =============================================================================
// Read Transaction
// =============================================================================

/// Sled read transaction wrapper
///
/// Provides read-only access to the database with snapshot isolation semantics.
/// All reads within a transaction see a consistent view of the database.
pub struct SledReadTransaction<'db, D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Reference to the sled database
    pub(crate) db: &'db sled::Db,

    /// Reference to the parent store
    pub(crate) _sled_store: &'db SledStore<D>,
}

impl<'db, D: NetabaseDefinition> SledReadTransaction<'db, D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    pub fn get<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<Option<M>>
    where
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + 'static,
        M: bincode::Decode<()>,
    {
        // Open the main tree for this model
        let tree_name_binding = M::MODEL_TREE_NAME;
        let tree_name = tree_name_binding.name();
        let tree = self.db.open_tree(tree_name)?;

        // Serialize the key
        let key_bytes = serialize_key(&key)?;

        // Get the value
        match tree.get(key_bytes)? {
            Some(value_bytes) => {
                let model = deserialize_value(&value_bytes)?;
                Ok(Some(model))
            }
            None => Ok(None),
        }
    }

    pub fn get_pk_by_secondary_key<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    where
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: bincode::Encode + IntoDiscriminant,
    {
        // Get the discriminant to find the right tree
        let discriminant = secondary_key.discriminant();

        // Generate tree name
        let tree_name = M::secondary_key_table_name(discriminant);
        let tree = self.db.open_tree(&tree_name)?;

        // Serialize the secondary key
        let sk_bytes = serialize_key(&secondary_key)?;

        // Get the primary key
        match tree.get(sk_bytes)? {
            Some(pk_bytes) => {
                let pk = deserialize_value(&pk_bytes)?;
                Ok(Some(pk))
            }
            None => Ok(None),
        }
    }

    pub fn get_subscription_accumulator<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <<M as NetabaseModelTrait<D>>::SubscriptionEnum as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<([u8; 32], u64)>
    where
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + 'static,
    {
        // Generate tree name
        let tree_name = M::subscription_key_table_name(subscription_discriminant);
        let tree = self.db.open_tree(&tree_name)?;

        // Initialize accumulator and count
        let mut accumulator = [0u8; 32];
        let mut count = 0u64;

        // Iterate through all entries and XOR their hashes
        for entry in tree.iter() {
            let (_key, hash_bytes) = entry?;

            // Deserialize the hash
            let hash: [u8; 32] = deserialize_value(&hash_bytes)?;

            // XOR into accumulator (order-independent)
            for i in 0..32 {
                accumulator[i] ^= hash[i];
            }

            count += 1;
        }

        Ok((accumulator, count))
    }

    pub fn get_subscription_keys<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <<M as NetabaseModelTrait<D>>::SubscriptionEnum as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<Vec<M::PrimaryKey>>
    where
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + 'static,
    {
        // Generate tree name
        let tree_name = M::subscription_key_table_name(subscription_discriminant);
        let tree = self.db.open_tree(&tree_name)?;

        let mut keys = Vec::new();

        // Iterate through all entries and collect keys
        for entry in tree.iter() {
            let (key_bytes, _hash) = entry?;
            let key = deserialize_key(&key_bytes)?;
            keys.push(key);
        }

        Ok(keys)
    }
}

// =============================================================================
// Write Transaction - Operation Queue
// =============================================================================

/// Operation types for the write transaction queue
///
/// Operations are queued during the transaction and executed atomically
/// in priority order when the transaction is committed.
#[derive(Debug)]
pub enum SledOperation {
    /// Insert into main model tree
    MainTreeInsert {
        tree_name: String,
        key_bytes: Vec<u8>,
        value_bytes: Vec<u8>,
        priority: u8,
    },

    /// Insert into secondary key index
    SecondaryKeyInsert {
        tree_name: String,
        key_bytes: Vec<u8>,
        pk_bytes: Vec<u8>,
        priority: u8,
    },

    /// Insert into relational key index
    RelationalKeyInsert {
        tree_name: String,
        key_bytes: Vec<u8>,
        pk_bytes: Vec<u8>,
        priority: u8,
    },

    /// Insert into hash tree
    HashTreeInsert {
        tree_name: String,
        pk_bytes: Vec<u8>,
        hash: [u8; 32],
        priority: u8,
    },

    /// Insert into subscription tree
    SubscriptionInsert {
        tree_name: String,
        pk_bytes: Vec<u8>,
        hash: [u8; 32],
        priority: u8,
    },

    /// Delete from tree
    Delete {
        tree_name: String,
        key_bytes: Vec<u8>,
        priority: u8,
    },
}

impl SledOperation {
    /// Get the priority of this operation
    ///
    /// Lower numbers execute first:
    /// - 0: Main tree inserts
    /// - 1: Secondary key inserts
    /// - 2: Relational key inserts
    /// - 3: Hash tree inserts
    /// - 4: Subscription inserts
    /// - 5: Deletes
    pub fn priority(&self) -> u8 {
        match self {
            SledOperation::MainTreeInsert { priority, .. } => *priority,
            SledOperation::SecondaryKeyInsert { priority, .. } => *priority,
            SledOperation::RelationalKeyInsert { priority, .. } => *priority,
            SledOperation::HashTreeInsert { priority, .. } => *priority,
            SledOperation::SubscriptionInsert { priority, .. } => *priority,
            SledOperation::Delete { priority, .. } => *priority,
        }
    }

    /// Execute this operation against the database
    pub fn execute(&self, db: &sled::Db) -> NetabaseResult<()> {
        match self {
            SledOperation::MainTreeInsert {
                tree_name,
                key_bytes,
                value_bytes,
                ..
            } => {
                let tree = db.open_tree(tree_name)?;
                tree.insert(key_bytes, value_bytes.as_slice())?;
            }
            SledOperation::SecondaryKeyInsert {
                tree_name,
                key_bytes,
                pk_bytes,
                ..
            } => {
                let tree = db.open_tree(tree_name)?;
                tree.insert(key_bytes, pk_bytes.as_slice())?;
            }
            SledOperation::RelationalKeyInsert {
                tree_name,
                key_bytes,
                pk_bytes,
                ..
            } => {
                let tree = db.open_tree(tree_name)?;
                tree.insert(key_bytes, pk_bytes.as_slice())?;
            }
            SledOperation::HashTreeInsert {
                tree_name,
                pk_bytes,
                hash,
                ..
            } => {
                let tree = db.open_tree(tree_name)?;
                let hash_bytes = serialize_value(hash)?;
                tree.insert(pk_bytes, hash_bytes.as_slice())?;
            }
            SledOperation::SubscriptionInsert {
                tree_name,
                pk_bytes,
                hash,
                ..
            } => {
                let tree = db.open_tree(tree_name)?;
                let hash_bytes = serialize_value(hash)?;
                tree.insert(pk_bytes, hash_bytes.as_slice())?;
            }
            SledOperation::Delete {
                tree_name,
                key_bytes,
                ..
            } => {
                let tree = db.open_tree(tree_name)?;
                tree.remove(key_bytes)?;
            }
        }
        Ok(())
    }
}

// =============================================================================
// Write Transaction
// =============================================================================

/// Sled write transaction wrapper
///
/// Provides write access to the database with operation queueing.
/// All operations are queued and executed atomically in priority order
/// when the transaction is committed.
pub struct SledWriteTransaction<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// The sled database (Arc internally, cheap to clone)
    pub(crate) db: sled::Db,

    /// Queue of pending operations
    pub(crate) operation_queue: Vec<SledOperation>,

    /// Pointer to the parent store (for lifetime management)
    pub(crate) _sled_store: *const SledStore<D>,
}

impl<D: NetabaseDefinition> SledWriteTransaction<D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Create a new write transaction
    pub fn new(db: &sled::Db, sled_store: &SledStore<D>) -> Self {
        Self {
            db: db.clone(), // Sled Db is Arc internally
            operation_queue: Vec::new(),
            _sled_store: sled_store as *const SledStore<D>,
        }
    }

    /// Add an operation to the queue
    pub fn add_operation(&mut self, operation: SledOperation) {
        self.operation_queue.push(operation);
    }

    /// Process the operation queue atomically using sled::Batch
    fn process_queue(&mut self) -> NetabaseResult<()> {
        let start = Instant::now();
        let ops_count = self.operation_queue.len();
        debug!("SledWriteTransaction: Processing queue with {} operations", ops_count);

        // Sort operations by priority
        let sort_start = Instant::now();
        self.operation_queue.sort_by_key(|op| op.priority());
        trace!("  Sorted operations in {:?}", sort_start.elapsed());

        let prep_start = Instant::now();
        let mut batches: std::collections::HashMap<String, sled::Batch> = std::collections::HashMap::new();
        let mut batch_order: Vec<String> = Vec::new();

        // Execute all operations in order
        let operations = std::mem::take(&mut self.operation_queue);

        for operation in operations {
            let (tree_name, key, value_op) = match operation {
                SledOperation::MainTreeInsert { tree_name, key_bytes, value_bytes, .. } => {
                    (tree_name, key_bytes, Some(value_bytes))
                }
                SledOperation::SecondaryKeyInsert { tree_name, key_bytes, pk_bytes, .. } => {
                    (tree_name, key_bytes, Some(pk_bytes))
                }
                SledOperation::RelationalKeyInsert { tree_name, key_bytes, pk_bytes, .. } => {
                    (tree_name, key_bytes, Some(pk_bytes))
                }
                SledOperation::HashTreeInsert { tree_name, pk_bytes, hash, .. } => {
                    let hash_bytes = serialize_value(&hash)?;
                    (tree_name, pk_bytes, Some(hash_bytes))
                }
                SledOperation::SubscriptionInsert { tree_name, pk_bytes, hash, .. } => {
                    let hash_bytes = serialize_value(&hash)?;
                    (tree_name, pk_bytes, Some(hash_bytes))
                }
                SledOperation::Delete { tree_name, key_bytes, .. } => {
                    (tree_name, key_bytes, None)
                }
            };

            if !batches.contains_key(&tree_name) {
                batches.insert(tree_name.clone(), sled::Batch::default());
                batch_order.push(tree_name.clone());
            }

            let batch = batches.get_mut(&tree_name).unwrap();
            match value_op {
                Some(value) => batch.insert(key, value),
                None => batch.remove(key),
            }
        }
        trace!("  Prepared {} batches in {:?}", batch_order.len(), prep_start.elapsed());

        // Apply batches in order
        let apply_start = Instant::now();
        for tree_name in batch_order {
            if let Some(batch) = batches.remove(&tree_name) {
                let tree_start = Instant::now();
                let tree = self.db.open_tree(&tree_name)?;
                tree.apply_batch(batch)?;
                trace!("    Applied batch to tree '{}' in {:?}", tree_name, tree_start.elapsed());
            }
        }
        trace!("  Applied all batches in {:?}", apply_start.elapsed());

        // Flush to ensure durability
        let flush_start = Instant::now();
        self.db.flush()?;
        trace!("  Flushed DB in {:?}", flush_start.elapsed());

        debug!("SledWriteTransaction: Completed in {:?}", start.elapsed());
        Ok(())
    }

    // Read methods (read during write)
    pub fn get<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<Option<M>>
    where
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + 'static,
        M: bincode::Decode<()>,
    {
        let tree_name_binding = M::MODEL_TREE_NAME;
        let tree_name = tree_name_binding.name();
        let tree = self.db.open_tree(tree_name)?;
        let key_bytes = serialize_key(&key)?;

        match tree.get(key_bytes)? {
            Some(value_bytes) => Ok(Some(deserialize_value(&value_bytes)?)),
            None => Ok(None),
        }
    }

    pub fn get_pk_by_secondary_key<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    where
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: bincode::Encode + IntoDiscriminant,
    {
        let discriminant = secondary_key.discriminant();
        let tree_name = M::secondary_key_table_name(discriminant);
        let tree = self.db.open_tree(&tree_name)?;
        let sk_bytes = serialize_key(&secondary_key)?;

        match tree.get(sk_bytes)? {
            Some(pk_bytes) => Ok(Some(deserialize_value(&pk_bytes)?)),
            None => Ok(None),
        }
    }

    pub fn get_subscription_accumulator<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <<M as NetabaseModelTrait<D>>::SubscriptionEnum as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<([u8; 32], u64)>
    where
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + 'static,
    {
        let tree_name = M::subscription_key_table_name(subscription_discriminant);
        let tree = self.db.open_tree(&tree_name)?;

        let mut accumulator = [0u8; 32];
        let mut count = 0u64;

        for entry in tree.iter() {
            let (_key, hash_bytes) = entry?;
            let hash: [u8; 32] = deserialize_value(&hash_bytes)?;

            for i in 0..32 {
                accumulator[i] ^= hash[i];
            }
            count += 1;
        }

        Ok((accumulator, count))
    }

    pub fn get_subscription_keys<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <<M as NetabaseModelTrait<D>>::SubscriptionEnum as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<Vec<M::PrimaryKey>>
    where
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + 'static,
    {
        let tree_name = M::subscription_key_table_name(subscription_discriminant);
        let tree = self.db.open_tree(&tree_name)?;

        let mut keys = Vec::new();

        for entry in tree.iter() {
            let (key_bytes, _hash) = entry?;
            let key = deserialize_key(&key_bytes)?;
            keys.push(key);
        }

        Ok(keys)
    }

    // Write methods
    pub fn put<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D> + Send + Clone>(
        &mut self,
        model: M,
    ) -> NetabaseResult<()>
    where
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + Send + Clone + 'static,
        M: bincode::Encode,
        M::Hash: Clone + Into<[u8; 32]>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: bincode::Encode + IntoDiscriminant,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: bincode::Encode + IntoDiscriminant,
    {
        // Extract primary key
        let pk = model.primary_key();
        let pk_bytes = serialize_key(&pk)?;

        // Serialize the model
        let model_bytes = serialize_value(&model)?;

        // 1. Queue main tree operation (priority 0)
        let tree_name = M::MODEL_TREE_NAME.name().to_string();
        self.add_operation(SledOperation::MainTreeInsert {
            tree_name,
            key_bytes: pk_bytes.clone(),
            value_bytes: model_bytes,
            priority: 0,
        });

        // 2. Queue secondary key operations (priority 1)
        let secondary_keys: Vec<_> =
            <M::Keys as NetabaseModelKeyTrait<D, M>>::secondary_keys(&model);
        for sk in secondary_keys {
            let sk_discriminant = sk.discriminant();
            let sk_bytes = serialize_key(&sk)?;
            let tree_name = M::secondary_key_table_name(sk_discriminant);

            self.add_operation(SledOperation::SecondaryKeyInsert {
                tree_name,
                key_bytes: sk_bytes,
                pk_bytes: pk_bytes.clone(),
                priority: 1,
            });
        }

        // 3. Queue relational key operations (priority 2)
        let relational_keys: Vec<_> =
            <M::Keys as NetabaseModelKeyTrait<D, M>>::relational_keys(&model);
        for rk in relational_keys {
            let rk_discriminant = rk.discriminant();
            let rk_bytes = serialize_key(&rk)?;
            let tree_name = M::relational_key_table_name(rk_discriminant);

            self.add_operation(SledOperation::RelationalKeyInsert {
                tree_name,
                key_bytes: rk_bytes,
                pk_bytes: pk_bytes.clone(),
                priority: 2,
            });
        }

        // 4. Queue hash tree operation (priority 3)
        let hash: [u8; 32] = model.compute_hash().into();
        let hash_tree_name = M::hash_tree_table_name();

        self.add_operation(SledOperation::HashTreeInsert {
            tree_name: hash_tree_name,
            pk_bytes: pk_bytes.clone(),
            hash,
            priority: 3,
        });

        // 5. Queue subscription operations (priority 4)
        let subscriptions = model.get_subscriptions();
        for subscription in subscriptions {
            let sub_discriminant = subscription.discriminant();
            let tree_name = M::subscription_key_table_name(sub_discriminant);

            self.add_operation(SledOperation::SubscriptionInsert {
                tree_name,
                pk_bytes: pk_bytes.clone(),
                hash,
                priority: 4,
            });
        }

        Ok(())
    }

    pub fn delete<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D> + Clone + Send>(
        &mut self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<()>
    where
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + Send + Clone + 'static,
    {
        let tree_name = M::MODEL_TREE_NAME.name().to_string();
        let key_bytes = serialize_key(&key)?;

        self.add_operation(SledOperation::Delete {
            tree_name,
            key_bytes,
            priority: 5, // Delete has highest priority
        });

        Ok(())
    }

    pub fn commit(mut self) -> NetabaseResult<()> {
        self.process_queue()?;
        Ok(())
    }
}
