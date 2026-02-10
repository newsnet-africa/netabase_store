//! Range query operations for intersecting primary and secondary key ranges.
//!
//! This module provides efficient, type-safe range query functionality that
//! leverages redb's `Range` and `MultimapValue` iterators to intersect
//! constraints across primary and secondary key spaces.
//!
//! # Algorithm
//!
//! The query algorithm is designed for minimal memory overhead:
//!
//! 1. **Primary key collection**: First, collect all primary keys that fall
//!    within the optional primary key range. This forms the initial candidate set.
//!
//! 2. **Secondary key filtering**: For each secondary range constraint, identify
//!    the matching secondary table by discriminant, then iterate through that
//!    specific multimap table and filter the candidate set in-place.
//!
//! 3. **Final fetch**: Fetch the actual model values for the remaining primary
//!    keys from the main table.
//!
//! # Performance
//!
//! - Uses `HashSet` for O(1) membership testing during intersection
//! - Iterates secondary tables lazily via redb iterators
//! - Only queries the relevant secondary table (by discriminant)
//! - Applies limit/offset after all filtering for correct pagination

use std::collections::HashSet;
use std::hash::Hash;

use redb::{self, AccessGuard, ReadableMultimapTable, ReadableTable};

use super::tables::{ModelOpenTables, ReadWriteTableType, TablePermission, TableType};
use super::options::CrudOptions;
use crate::errors::{NetabaseError, NetabaseResult};
use crate::traits::registry::{
    definition::redb_definition::RedbDefinition,
    models::{
        keys::{ModelKeyRange, NetabaseModelKeys, SimpleKeyRange, blob::NetabaseModelBlobKey},
        model::{NetabaseModel, redb_model::RedbNetbaseModel},
    },
};
use strum::IntoDiscriminant;

/// Extracts a reference value from a SimpleKeyRange for discriminant matching.
/// Returns None for `All` variant which has no bounds.
fn get_range_reference<K>(range: &SimpleKeyRange<K>) -> Option<&K> {
    match range {
        SimpleKeyRange::All => None,
        SimpleKeyRange::From { start } => Some(start),
        SimpleKeyRange::To { end, .. } => Some(end),
        SimpleKeyRange::Between { start, .. } => Some(start),
    }
}

/// Executes a range query with primary and secondary key constraints.
///
/// This is the main entry point for `ModelKeyRange`-based queries.
///
/// # Algorithm
///
/// 1. Collect all primary keys within the primary range (or all if unbounded)
/// 2. For each secondary range:
///    - Extract the discriminant from the range to identify the correct secondary table
///    - Query only that specific secondary table (not all secondary tables)
///    - Intersect the found primary keys with the candidate set
/// 3. Fetch final models for remaining candidates with pagination
///
/// # Secondary Table Matching
///
/// Secondary keys are stored in separate multimap tables, one per field variant
/// (e.g., `Age`, `FirstName`, `LastName`). When filtering by a secondary range,
/// the function identifies the correct table by matching the discriminant from
/// the range bounds against the table name suffix.
///
/// # Type Requirements
///
/// This function requires all the standard redb key bounds that are already
/// present on the `RedbModelCrud` impl block. The key types must support
/// `Borrow<SelfType>` for redb's range operations, and importantly must have
/// `SelfType<'a> = Self` so that `.value()` returns the actual key type.
/// Secondary keys must implement `IntoDiscriminant` for table matching.
pub fn execute_range_query<'db, 'txn, 'a, D, M>(
    tables: &'a ModelOpenTables<'txn, 'db, D, M>,
    ranges: &ModelKeyRange<D, M>,
    config: CrudOptions,
) -> NetabaseResult<Vec<AccessGuard<'a, M::TableV>>>
where
    D: RedbDefinition + Clone,
    D::Discriminant: 'static + std::fmt::Debug,
    M: RedbNetbaseModel<'db, D> + redb::Key,
    'db: 'txn,
    // Primary key bounds - must have SelfType = Self for .value() to return Primary
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key + 'static + Clone + Eq + Hash + Ord,
    for<'v> <M::Keys as NetabaseModelKeys<D, M>>::Primary:
        std::borrow::Borrow<<<M::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'v>>,
    for<'v> &'v <M::Keys as NetabaseModelKeys<D, M>>::Primary:
        std::borrow::Borrow<<<M::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'v>>,
    for<'v> <M::Keys as NetabaseModelKeys<D, M>>::Primary:
        redb::Value<SelfType<'v> = <M::Keys as NetabaseModelKeys<D, M>>::Primary>,
    // Secondary key bounds - must implement IntoDiscriminant for table matching
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key + 'static + Clone + IntoDiscriminant,
    for<'v> <M::Keys as NetabaseModelKeys<D, M>>::Secondary:
        std::borrow::Borrow<<<M::Keys as NetabaseModelKeys<D, M>>::Secondary as redb::Value>::SelfType<'v>>,
    // Other key bounds (required by ModelOpenTables)
    <M::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key + 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
    <<M::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
    // Discriminant bounds
    <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
    D::SubscriptionKeys: redb::Key + 'static,
{
    let limit = config.list.limit;
    let offset = config.list.offset.unwrap_or(0);

    // Step 1: Collect primary keys within the primary range
    let primary_range = ranges
        .primary
        .clone()
        .unwrap_or(SimpleKeyRange::All);

    let mut candidates: HashSet<<M::Keys as NetabaseModelKeys<D, M>>::Primary> = HashSet::new();

    match &tables.main {
        TablePermission::ReadOnly(TableType::Table(table)) => {
            let iter = table.range(primary_range).map_err(|e| NetabaseError::RedbError(e.into()))?;
            for item in iter {
                let (k_guard, _v_guard) = item.map_err(|e| NetabaseError::RedbError(e.into()))?;
                candidates.insert(k_guard.value());
            }
        }
        TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
            let iter = table.range(primary_range).map_err(|e| NetabaseError::RedbError(e.into()))?;
            for item in iter {
                let (k_guard, _v_guard) = item.map_err(|e| NetabaseError::RedbError(e.into()))?;
                candidates.insert(k_guard.value());
            }
        }
        _ => return Err(NetabaseError::Other),
    }

    // Early exit if no candidates
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    // Step 2: Filter by each secondary range
    for sec_range in &ranges.secondary {
        // Get a reference value from the range to determine the discriminant
        let range_ref = get_range_reference(sec_range);
        
        // For All range, we skip secondary filtering (returns all)
        // For specific ranges, we find the matching table by discriminant
        if range_ref.is_none() {
            continue; // All range doesn't filter
        }
        
        let ref_value = range_ref.unwrap();
        let discriminant = ref_value.discriminant();
        let discriminant_str = format!("{:?}", discriminant);
        
        // Collect all primary keys present in the matching secondary table
        let mut found_in_secondary: HashSet<<M::Keys as NetabaseModelKeys<D, M>>::Primary> = HashSet::new();

        for (table_perm, table_name) in &tables.secondary {
            // Only query the table that matches our discriminant
            // Table names end with the discriminant name (e.g., "MyApp:User:Secondary:Age")
            if !table_name.ends_with(&discriminant_str) {
                continue;
            }
            
            match table_perm {
                TablePermission::ReadOnly(TableType::MultimapTable(table)) => {
                    let iter = table
                        .range(sec_range.clone())
                        .map_err(|e| NetabaseError::RedbError(e.into()))?;

                    for item in iter {
                        let (_sec_guard, values) = item.map_err(|e| NetabaseError::RedbError(e.into()))?;
                        for value in values {
                            let prim_guard = value.map_err(|e| NetabaseError::RedbError(e.into()))?;
                            found_in_secondary.insert(prim_guard.value());
                        }
                    }
                }
                TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table))
                | TablePermission::ReadOnlyWrite(ReadWriteTableType::MultimapTable(table)) => {
                    let iter = table
                        .range(sec_range.clone())
                        .map_err(|e| NetabaseError::RedbError(e.into()))?;

                    for item in iter {
                        let (_sec_guard, values) = item.map_err(|e| NetabaseError::RedbError(e.into()))?;
                        for value in values {
                            let prim_guard = value.map_err(|e| NetabaseError::RedbError(e.into()))?;
                            found_in_secondary.insert(prim_guard.value());
                        }
                    }
                }
                _ => {}
            }
        }

        // Intersect: keep only candidates that are also in found_in_secondary
        candidates.retain(|k| found_in_secondary.contains(k));

        // Early exit if intersection is empty
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
    }

    // Step 3: Fetch models with pagination
    // Sort keys for deterministic ordering
    let mut sorted_keys: Vec<_> = candidates.into_iter().collect();
    sorted_keys.sort();

    let mut result = Vec::new();
    let mut seen = 0usize;
    let mut taken = 0usize;

    for key in sorted_keys {
        // Apply offset
        if seen < offset {
            seen += 1;
            continue;
        }

        // Apply limit
        if let Some(lim) = limit {
            if taken >= lim {
                break;
            }
        }

        // Fetch from main table
        match &tables.main {
            TablePermission::ReadOnly(TableType::Table(table)) => {
                if let Ok(Some(guard)) = table.get(&key) {
                    result.push(guard);
                    taken += 1;
                }
            }
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                if let Ok(Some(guard)) = table.get(&key) {
                    result.push(guard);
                    taken += 1;
                }
            }
            _ => {}
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests would go here, but require a full model setup.
    // Integration tests are in tests/integration_list.rs
}
