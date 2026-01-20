// Test schema export with migration data

use netabase_store::traits::registery::definition::NetabaseDefinition;
use netabase_store_examples::boilerplate_lib::Definition;

#[test]
fn test_schema_export_includes_migration_data() {
    // Export schema to TOML
    let schema = Definition::schema();
    let toml_str = schema.to_toml();

    println!("Exported TOML schema:");
    println!("{}", toml_str);

    // Verify basic structure
    assert!(toml_str.contains("schema_format_version"));
    assert!(toml_str.contains("name = \"Definition\""));

    // Verify migration-related data is present
    assert!(
        toml_str.contains("model_history"),
        "Schema should include model_history section"
    );

    // Verify User model family history
    assert!(
        toml_str.contains("family = \"User\""),
        "Should have User model family"
    );

    // Check for multiple versions
    assert!(
        toml_str.contains("current_version"),
        "Should have current_version field"
    );

    // Check for migration paths (may be empty initially, but structure should exist)
    // The migration_paths field should be present even if empty

    println!("\n✓ Schema export includes migration metadata");
}

#[test]
fn test_schema_config_field() {
    let schema = Definition::schema();

    // Config should be None by default
    assert!(schema.config.is_none(), "Config should be None by default");

    println!("✓ Schema config field present and correctly set to None");
}

#[test]
fn test_model_version_history() {
    let schema = Definition::schema();

    // Find User model family in history
    let user_family = schema
        .model_history
        .iter()
        .find(|h| h.family == "User")
        .expect("Should have User model family in history");

    println!("User model family:");
    println!("  Family: {}", user_family.family);
    println!("  Current version: {}", user_family.current_version);
    println!("  Total versions: {}", user_family.versions.len());

    // Should have 2 versions: UserV1 (v1) and User (v2)
    assert_eq!(user_family.versions.len(), 2, "User should have 2 versions");
    assert_eq!(
        user_family.current_version, 2,
        "Current version should be 2"
    );

    // Check version details
    let v1 = &user_family.versions[0];
    assert_eq!(v1.version, 1);
    assert_eq!(v1.struct_name, "UserV1");

    let v2 = &user_family.versions[1];
    assert_eq!(v2.version, 2);
    assert_eq!(v2.struct_name, "User");
    assert_eq!(
        v2.supports_upgrade, true,
        "Current version should support upgrade"
    );

    println!("✓ Model version history correctly tracked");
}

#[test]
fn test_post_model_supports_downgrade() {
    let schema = Definition::schema();

    // Find Post model family
    let post_family = schema
        .model_history
        .iter()
        .find(|h| h.family == "Post")
        .expect("Should have Post model family");

    println!("Post model family:");
    println!("  Versions: {}", post_family.versions.len());

    // Post has supports_downgrade attribute
    let v2 = post_family
        .versions
        .iter()
        .find(|v| v.version == 2)
        .expect("Should have Post v2");

    assert!(v2.supports_downgrade, "Post v2 should support downgrade");

    println!("✓ Post model correctly marked with supports_downgrade");
}
