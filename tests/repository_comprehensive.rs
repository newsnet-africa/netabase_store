//! Tests for repository store implementation

mod common;

use common::{cleanup_test_db, create_test_db};
use netabase_store::databases::redb::repository::RedbRepositoryDefinitions;
use netabase_store::errors::NetabaseResult;
use netabase_store::traits::registery::repository::NetabaseRepository;

use netabase_store_examples::{
    Category, CategoryID, Definition, MainRepository, User, UserID,
};

/// Test that the repository marker struct is generated correctly
#[test]
fn test_repository_marker() {
    // Repository should have the correct name
    assert_eq!(MainRepository::name(), "MainRepository");

    // Repository should have the correct definition count
    assert_eq!(MainRepository::definition_count(), 2);
}

/// Test that definition_names returns the correct list
#[test]
fn test_definition_names() {
    let names = MainRepository::definition_names();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"Definition"));
    assert!(names.contains(&"DefinitionTwo"));
}

/// Test that repository stores can create the folder structure
#[test]
fn test_repository_stores_creation() -> NetabaseResult<()> {
    use netabase_store_examples::MainRepositoryStores;

    let temp_dir = std::env::temp_dir().join(format!(
        "netabase_repo_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    // Create the repository stores
    let _stores = MainRepositoryStores::new(&temp_dir)?;

    // Verify folder structure was created
    assert!(temp_dir.exists(), "Repository folder should exist");
    assert!(
        temp_dir.join("Definition").exists(),
        "Definition folder should exist"
    );
    assert!(
        temp_dir.join("DefinitionTwo").exists(),
        "DefinitionTwo folder should exist"
    );
    assert!(
        temp_dir.join("Definition").join("data.redb").exists(),
        "Definition database should exist"
    );
    assert!(
        temp_dir.join("Definition").join("schema.toml").exists(),
        "Definition schema should exist"
    );
    assert!(
        temp_dir.join("DefinitionTwo").join("data.redb").exists(),
        "DefinitionTwo database should exist"
    );
    assert!(
        temp_dir.join("DefinitionTwo").join("schema.toml").exists(),
        "DefinitionTwo schema should exist"
    );

    // Clean up
    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(())
}

/// Test that each definition store works independently
#[test]
fn test_definition_stores_independent() -> NetabaseResult<()> {
    use netabase_store::relational::RelationalLink;
    use netabase_store_examples::{AnotherLargeUserFile, LargeUserFile, MainRepositoryStores};

    let temp_dir = std::env::temp_dir().join(format!(
        "netabase_repo_indep_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    // Create the repository stores
    let stores = MainRepositoryStores::new(&temp_dir)?;

    // Create a category in DefinitionTwo store
    {
        let txn = stores.definition_two.begin_write()?;
        let category = Category {
            id: CategoryID("cat1".to_string()),
            name: "Electronics".to_string(),
            description: "Electronic devices".to_string(),
            subscriptions: vec![],
        };
        txn.create(&category)?;
        txn.commit()?;
    }

    // Verify category was created
    {
        let txn = stores.definition_two.begin_read()?;
        let category = txn.read::<Category>(&CategoryID("cat1".to_string()))?;
        assert!(category.is_some());
        assert_eq!(category.unwrap().name, "Electronics");
    }

    // Create a user in Definition store that links to the category
    {
        let txn = stores.definition.begin_write()?;
        let user = User {
            id: UserID("user1".to_string()),
            name: "Alice".to_string(),
            age: 30,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".to_string())),
            subscriptions: vec![],
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create(&user)?;
        txn.commit()?;
    }

    // Verify user was created
    {
        let txn = stores.definition.begin_read()?;
        let user = txn.read::<User>(&UserID("user1".to_string()))?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.name, "Alice");
        // The category link should be dehydrated (just holds the ID)
        assert_eq!(user.category.get_primary_key().0, "cat1");
    }

    // Clean up
    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(())
}

/// Test that standalone definition stores still work
#[test]
fn test_standalone_definition_store() -> NetabaseResult<()> {
    // Create a standalone Definition store (not part of repository)
    let (store, db_path) = create_test_db::<Definition>("standalone_definition")?;

    use netabase_store::relational::RelationalLink;
    use netabase_store_examples::{AnotherLargeUserFile, LargeUserFile};

    // Create a user
    {
        let txn = store.begin_write()?;
        let user = User {
            id: UserID("standalone_user".to_string()),
            name: "Bob".to_string(),
            age: 25,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".to_string())),
            subscriptions: vec![],
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create(&user)?;
        txn.commit()?;
    }

    // Verify user was created
    {
        let txn = store.begin_read()?;
        let user = txn.read::<User>(&UserID("standalone_user".to_string()))?;
        assert!(user.is_some());
        assert_eq!(user.unwrap().name, "Bob");
    }

    cleanup_test_db(db_path);
    Ok(())
}
