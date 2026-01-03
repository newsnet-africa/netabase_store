//! Comprehensive migration tests with state inspection before and after operations.
//!
//! This test suite ensures migration features work correctly by:
//! 1. Checking initial state before operations
//! 2. Performing migration operations
//! 3. Verifying final state matches expectations
//! 4. Testing error conditions and edge cases

#![allow(dead_code)]
#![allow(unused_imports)]

mod common;

use netabase_store::query::QueryResult;
use netabase_store::traits::migration::{VersionContext, VersionHeader};

#[test]
fn test_version_header_roundtrip() {
    // Test that version header encoding/decoding is lossless
    for version in [0, 1, 100, u32::MAX] {
        let header = VersionHeader::new(version);
        let bytes = header.to_bytes();

        // Verify size
        assert_eq!(bytes.len(), VersionHeader::SIZE);

        // Verify magic bytes
        assert_eq!(bytes[0], b'N');
        assert_eq!(bytes[1], b'V');

        // Roundtrip
        let decoded = VersionHeader::from_bytes(&bytes).expect("Failed to decode header");
        assert_eq!(decoded.version, version);
        assert_eq!(decoded.magic, VersionHeader::MAGIC);
    }
}

#[test]
fn test_version_header_detection() {
    // Valid versioned data
    let versioned = VersionHeader::new(1).to_bytes();
    assert!(VersionHeader::is_versioned(&versioned));

    // Too short
    let too_short = vec![b'N', b'V', 0, 0];
    assert!(!VersionHeader::is_versioned(&too_short));

    // Wrong magic
    let wrong_magic = vec![b'X', b'Y', 1, 0, 0, 0];
    assert!(!VersionHeader::is_versioned(&wrong_magic));

    // Legacy unversioned
    let legacy = vec![0u8; 20];
    assert!(!VersionHeader::is_versioned(&legacy));
}

#[test]
fn test_version_context_creation() {
    // Default context
    let default_ctx = VersionContext::default();
    assert_eq!(default_ctx.expected_version, 0);
    assert!(default_ctx.auto_migrate);
    assert!(!default_ctx.strict);

    // Custom context
    let custom_ctx = VersionContext::new(3);
    assert_eq!(custom_ctx.expected_version, 3);
    assert!(custom_ctx.auto_migrate);

    // Strict context
    let strict_ctx = VersionContext::strict(2);
    assert_eq!(strict_ctx.expected_version, 2);
    assert!(!strict_ctx.auto_migrate);
    assert!(strict_ctx.strict);
}

#[test]
fn test_version_context_needs_migration() {
    let mut ctx = VersionContext::new(3);

    // No actual version yet
    assert!(!ctx.needs_migration());

    // Same version - no migration needed
    ctx.actual_version = Some(3);
    assert!(!ctx.needs_migration());

    // Different version - migration needed
    ctx.actual_version = Some(2);
    assert!(ctx.needs_migration());
}

#[test]
fn test_version_context_delta() {
    let mut ctx = VersionContext::new(5);

    // No actual version
    assert_eq!(ctx.version_delta(), 0);

    // Same version
    ctx.actual_version = Some(5);
    assert_eq!(ctx.version_delta(), 0);

    // Upgrade needed (actual < expected)
    ctx.actual_version = Some(3);
    assert_eq!(ctx.version_delta(), 2);

    // Downgrade (actual > expected) - rare
    ctx.actual_version = Some(7);
    assert_eq!(ctx.version_delta(), -2);
}

#[test]
fn test_query_result_utilities() {
    // Test unwrap_single
    let single = QueryResult::Single(Some(42));
    assert_eq!(single.unwrap_single(), 42);

    // Test expect_single
    let single2 = QueryResult::Single(Some(100));
    assert_eq!(single2.expect_single("should have value"), 100);

    // Test as_single
    let single3 = QueryResult::Single(Some(200));
    assert_eq!(single3.as_single(), Some(&200));

    // Test as_multiple
    let multiple = QueryResult::Multiple(vec![1, 2, 3]);
    assert_eq!(multiple.as_multiple(), Some(&vec![1, 2, 3]));

    // Test into_vec
    let multi2 = QueryResult::Multiple(vec![10, 20, 30]);
    assert_eq!(multi2.into_vec(), vec![10, 20, 30]);
}

#[test]
#[should_panic(expected = "called `QueryResult::unwrap_single()` on a `None` value")]
fn test_query_result_unwrap_single_panics_on_none() {
    let empty: QueryResult<i32> = QueryResult::Single(None);
    empty.unwrap_single();
}

#[test]
#[should_panic(expected = "called `QueryResult::unwrap_single()` on a non-Single variant")]
fn test_query_result_unwrap_single_panics_on_multiple() {
    let multiple = QueryResult::Multiple(vec![1, 2, 3]);
    multiple.unwrap_single();
}
