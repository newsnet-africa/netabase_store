//! Test migration system with family enum fallback
//!
//! This test verifies that the family enum correctly handles:
//! 1. Data with version headers
//! 2. Legacy data without version headers
//! 3. Mixed scenarios in the same database

use netabase_macros::{netabase_definition, NetabaseModel};
use netabase_store::traits::migration::{MigrateFrom, VersionHeader};
use serde::{Deserialize, Serialize};

// Define a versioned model family
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, NetabaseModel)]
#[netabase_model(definition = "TestDef", version = 1, family = "User")]
struct UserV1 {
    #[primary_key]
    id: u64,
    name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, NetabaseModel)]
#[netabase_model(definition = "TestDef", version = 2, family = "User")]
struct UserV2 {
    #[primary_key]
    id: u64,
    name: String,
    email: String,
}

impl MigrateFrom<UserV1> for UserV2 {
    fn migrate_from(old: UserV1) -> Self {
        UserV2 {
            id: old.id,
            name: old.name,
            email: String::from("unknown@example.com"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, NetabaseModel)]
#[netabase_model(definition = "TestDef", version = 3, family = "User", current)]
struct User {
    #[primary_key]
    id: u64,
    name: String,
    email: String,
    verified: bool,
}

impl MigrateFrom<UserV2> for User {
    fn migrate_from(old: UserV2) -> Self {
        User {
            id: old.id,
            name: old.name,
            email: old.email,
            verified: false,
        }
    }
}

#[netabase_definition]
enum TestDef {
    User(User),
}

#[test]
fn test_migration_with_version_header() {
    // Create a V1 user and serialize with version header
    let v1_user = UserV1 {
        id: 1,
        name: "Alice".to_string(),
    };
    
    // Manually create versioned bytes
    let mut v1_bytes = VersionHeader::new(1).to_bytes().to_vec();
    v1_bytes.extend(postcard::to_allocvec(&v1_user).unwrap());
    
    // Deserialize using family enum (should detect version from header)
    let family = UserFamily::from_bytes_auto(&v1_bytes).expect("Failed to deserialize with header");
    assert_eq!(family.version(), 1);
    
    // Migrate to current
    let current: User = family.to_current();
    assert_eq!(current.id, 1);
    assert_eq!(current.name, "Alice");
    assert_eq!(current.email, "unknown@example.com");
    assert!(!current.verified);
}

#[test]
fn test_migration_without_version_header() {
    // Create a V1 user and serialize WITHOUT version header (legacy format)
    let v1_user = UserV1 {
        id: 2,
        name: "Bob".to_string(),
    };
    
    let v1_bytes = postcard::to_allocvec(&v1_user).unwrap();
    
    // Deserialize using family enum (should try each version)
    let family = UserFamily::from_bytes_auto(&v1_bytes).expect("Failed to deserialize legacy data");
    assert_eq!(family.version(), 1);
    
    // Migrate to current
    let current: User = family.to_current();
    assert_eq!(current.id, 2);
    assert_eq!(current.name, "Bob");
}

#[test]
fn test_migration_v2_to_v3() {
    // Create a V2 user
    let v2_user = UserV2 {
        id: 3,
        name: "Charlie".to_string(),
        email: "charlie@test.com".to_string(),
    };
    
    // Serialize with version header
    let mut v2_bytes = VersionHeader::new(2).to_bytes().to_vec();
    v2_bytes.extend(postcard::to_allocvec(&v2_user).unwrap());
    
    // Deserialize and migrate
    let family = UserFamily::from_bytes_auto(&v2_bytes).expect("Failed to deserialize V2");
    assert_eq!(family.version(), 2);
    
    let current: User = family.to_current();
    assert_eq!(current.id, 3);
    assert_eq!(current.name, "Charlie");
    assert_eq!(current.email, "charlie@test.com");
    assert!(!current.verified);
}

#[test]
fn test_current_version_no_migration() {
    // Create current version user
    let current_user = User {
        id: 4,
        name: "Diana".to_string(),
        email: "diana@test.com".to_string(),
        verified: true,
    };
    
    // Serialize with version header
    let mut bytes = VersionHeader::new(3).to_bytes().to_vec();
    bytes.extend(postcard::to_allocvec(&current_user).unwrap());
    
    // Deserialize (no migration needed)
    let family = UserFamily::from_bytes_auto(&bytes).expect("Failed to deserialize current");
    assert_eq!(family.version(), 3);
    
    let result: User = family.to_current();
    assert_eq!(result, current_user);
}

#[test]
fn test_try_from_bytes_with_version() {
    // Test the direct version-based deserialization
    let v1_user = UserV1 {
        id: 5,
        name: "Eve".to_string(),
    };
    
    let v1_bytes = postcard::to_allocvec(&v1_user).unwrap();
    
    // Directly specify version
    let family = UserFamily::try_from_bytes_with_version(&v1_bytes, 1)
        .expect("Failed to deserialize with known version");
    assert_eq!(family.version(), 1);
    
    let current: User = family.to_current();
    assert_eq!(current.id, 5);
    assert_eq!(current.name, "Eve");
}

#[test]
fn test_unknown_version_error() {
    let v1_user = UserV1 {
        id: 6,
        name: "Frank".to_string(),
    };
    
    let v1_bytes = postcard::to_allocvec(&v1_user).unwrap();
    
    // Try to deserialize as unknown version
    let result = UserFamily::try_from_bytes_with_version(&v1_bytes, 99);
    assert!(result.is_err(), "Should fail for unknown version");
}

#[test]
fn test_invalid_data_error() {
    let invalid_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF];
    
    // Should fail gracefully
    let result = UserFamily::from_bytes_auto(&invalid_bytes);
    assert!(result.is_err(), "Should fail for invalid data");
}

#[test]
fn test_multi_step_migration_chain() {
    // Test V1 -> V2 -> V3 migration chain
    let v1_user = UserV1 {
        id: 7,
        name: "Grace".to_string(),
    };
    
    let v1_bytes = postcard::to_allocvec(&v1_user).unwrap();
    
    // Deserialize V1 and migrate all the way to V3
    let family = UserFamily::from_bytes_auto(&v1_bytes).expect("Failed multi-step migration");
    let current: User = family.to_current();
    
    // Verify all migrations applied correctly
    assert_eq!(current.id, 7);
    assert_eq!(current.name, "Grace");
    assert_eq!(current.email, "unknown@example.com"); // From V1->V2
    assert!(!current.verified); // From V2->V3
}
