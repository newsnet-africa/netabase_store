# Strategy for fixing redb::Key trait bounds

The issue: We have `redb::Key` in trait bounds which breaks Sled-only builds.

## Solution: Use conditional impl blocks

```rust
// For Sled only (or both)
#[cfg(not(feature = "redb"))]
impl<'txn, D, M, Mode> TreeView<'txn, D, M, Mode>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    M::PrimaryKey: Ord,
    M::SecondaryKeys: Ord,
{
    // All methods here
}

// For Redb (when enabled)
#[cfg(feature = "redb")]
impl<'txn, D, M, Mode> TreeView<'txn, D, M, Mode>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    M::PrimaryKey: Ord + redb::Key,
    M::SecondaryKeys: Ord,
{
    // Exact same methods here
}
```

This way:
- When only sled is enabled, we get the first impl
- When redb is enabled (with or without sled), we get the second impl with the Key bound
