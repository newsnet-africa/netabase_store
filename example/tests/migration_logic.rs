use netabase_store::relational::RelationalLink;
use netabase_store::traits::migration::{MigrateFrom, MigrateTo};
use netabase_store_examples::boilerplate_lib::{
    AnotherLargeUserFile, CategoryID, LargeUserFile, User, UserID, UserV1,
};

#[test]
fn test_user_migration_v1_to_v2() {
    // 1. Create a V1 user
    let user_v1 = UserV1 {
        id: UserID("user_123".to_string()),
        name: "Alice Smith".to_string(),
        age: 30,
        category: RelationalLink::new_dehydrated(CategoryID("cat_789".to_string())),
    };

    // 2. Migrate to V2
    let user_v2 = User::migrate_from(user_v1.clone());

    // 3. Verify migration logic
    assert_eq!(user_v2.id.0, user_v1.id.0);
    assert_eq!(user_v2.first_name, "Alice");
    assert_eq!(user_v2.last_name, "Smith");
    assert_eq!(user_v2.age, user_v1.age);

    // Verify default values for new fields
    assert_eq!(user_v2.bio, LargeUserFile::default());
    assert_eq!(user_v2.another, AnotherLargeUserFile::default());

    // Verify dehydrated link creation
    match user_v2.partner {
        RelationalLink::Dehydrated { primary_key, .. } => assert_eq!(primary_key.0, "user_123"),
        _ => panic!("Partner link should be dehydrated"),
    }
}

#[test]
fn test_user_migration_v1_to_v2_single_name() {
    // Test with single name (no whitespace)
    let user_v1 = UserV1 {
        id: UserID("user_456".to_string()),
        name: "Cher".to_string(),
        age: 50,
        category: RelationalLink::new_dehydrated(CategoryID("cat_music".to_string())),
    };

    let user_v2 = User::migrate_from(user_v1);

    assert_eq!(user_v2.first_name, "Cher");
    assert_eq!(user_v2.last_name, ""); // Should be empty
}

#[test]
fn test_user_migration_v2_to_v1_downgrade() {
    // 1. Create a V2 user
    let user_v2 = User {
        id: UserID("user_789".to_string()),
        first_name: "Bob".to_string(),
        last_name: "Builder".to_string(),
        age: 40,
        partner: RelationalLink::new_dehydrated(UserID("user_000".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("cat_build".to_string())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile::default(),
    };

    // 2. Downgrade to V1
    let user_v1 = user_v2.migrate_to();

    // 3. Verify downgrade logic
    assert_eq!(user_v1.id.0, "user_789");
    assert_eq!(user_v1.name, "Bob Builder");
    assert_eq!(user_v1.age, 40);

    // Verify category link is preserved
    match user_v1.category {
        RelationalLink::Dehydrated { primary_key, .. } => assert_eq!(primary_key.0, "cat_build"),
        _ => panic!("Category link should be dehydrated"),
    }
}
