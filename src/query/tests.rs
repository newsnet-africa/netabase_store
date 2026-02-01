//! Tests for query module.

use super::*;

#[test]
fn test_query_config_defaults() {
    let config = QueryConfig::default();
    assert_eq!(config.mode, QueryMode::Fetch);
    assert_eq!(config.pagination.limit, None);
    assert_eq!(config.pagination.offset, None);
    assert!(config.fetch_options.include_blobs);
    assert_eq!(config.fetch_options.hydration_depth, 0);
    assert!(!config.reversed);
}

#[test]
fn test_query_config_builder() {
    let config = QueryConfig::default()
        .count_only()
        .with_limit(10)
        .with_offset(5)
        .with_blobs(false)
        .with_hydration(2)
        .reversed();

    assert_eq!(config.mode, QueryMode::Count);
    assert_eq!(config.pagination.limit, Some(10));
    assert_eq!(config.pagination.offset, Some(5));
    assert!(!config.fetch_options.include_blobs);
    assert_eq!(config.fetch_options.hydration_depth, 2);
    assert!(config.reversed);
}

#[test]
fn test_query_config_helpers() {
    let config = QueryConfig::default().no_blobs().no_hydration();
    assert!(!config.fetch_options.include_blobs);
    assert_eq!(config.fetch_options.hydration_depth, 0);
}

#[test]
fn test_query_config_range_change() {
    let config = QueryConfig::default().with_limit(5);
    // Default range is RangeFull
    let config_range = config.with_range(0..10);

    assert_eq!(config_range.range, 0..10);
    assert_eq!(config_range.pagination.limit, Some(5));
}
