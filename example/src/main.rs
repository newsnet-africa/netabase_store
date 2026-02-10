// Boilerplate example - Main entry point
//
// Now fully powered by macros! The entire boilerplate is generated automatically.
// Run with: cargo run --bin netabase_store_examples

use example::boilerplate_lib::main_repository::definition::{AnotherLargeUserFile, LargeUserFile};
use example::boilerplate_lib::{
    Category, CategoryID, MainRepositoryStores, Post, PostID, User, UserID,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Netabase Store - Boilerplate Example");
    println!("=====================================\n");

    // Use a temporary directory for the repository
    let temp_dir = tempfile::tempdir()?;
    let repo_path = temp_dir.path();

    println!("Initializing MainRepository at {:?}", repo_path);

    // 1. Initialize the repository stores
    // This creates the folder structure and initializes databases for all definitions
    let stores = MainRepositoryStores::new(repo_path)?;

    println!("Repository initialized successfully.\n");

    // 2. Create test data
    let category_id = CategoryID("cat1".to_string());
    let category = Category {
        id: category_id.clone(),
        name: "General".to_string(),
        description: "A general category".to_string(),
    };

    let user_id = UserID("user1".to_string());
    let alice_bio_data = vec![0u8; 150_000]; // 150KB
    let alice_another_data = vec![1u8; 70_000]; // 70KB

    let user = User {
        id: user_id.clone(),
        first_name: "Alice".to_string(),
        last_name: "Smith".to_string(),
        age: 30,
        partner: netabase_store::relational::RelationalLink::new_dehydrated(user_id.clone()),
        category: netabase_store::relational::RelationalLink::new_dehydrated(category_id.clone()),
        bio: LargeUserFile {
            data: alice_bio_data.clone(),
            metadata: "Alice's Bio".to_string(),
        },
        another: AnotherLargeUserFile(alice_another_data.clone()),
    };

    let post = Post {
        id: PostID("post1".to_string()),
        title: "Hello World".to_string(),
        author_id: "user1".to_string(),
        content: "This is a test post".to_string(),
        published: true,
        tags: vec!["rust".to_string(), "database".to_string()],
    };

    // 3. Write data to the respective stores
    println!("Writing data to stores...");

    // Writing to DefinitionTwo (Category)
    {
        let txn = stores.definition_two.begin_write()?;
        txn.create(&category)?;
        txn.commit()?;
    }

    // Writing to Definition (User and Post)
    {
        let txn = stores.definition.begin_write()?;
        txn.create(&user)?;
        txn.create(&post)?;
        txn.commit()?;
    }

    println!("Data written successfully.\n");

    // 4. Read data back and demonstrate hydration
    println!("Reading and verifying data...");

    let txn = stores.definition.begin_read()?;

    // Read user
    let read_user: User = txn.read(&user_id)?.expect("User should exist");
    println!(
        "Read User: {} {}, Age: {}",
        read_user.first_name, read_user.last_name, read_user.age
    );

    // Demonstrate blob reconstruction (handled automatically)
    println!(
        "  - Bio length: {} bytes (originally {} bytes)",
        read_user.bio.data.len(),
        alice_bio_data.len()
    );

    // Demonstrate relational hydration
    println!("\nHydrating relational links...");

    // Link within same definition (User -> Partner)
    let partner = read_user.partner.hydrate_self(&txn)?;
    println!(
        "  - Partner: {}",
        partner
            .get_model()
            .map(|u| format!("{} {}", u.first_name, u.last_name))
            .unwrap_or_else(|| "Not found".to_string())
    );

    // Link across definitions (User -> Category)
    // For cross-definition links, we can read the category directly from its store
    let category_id = read_user.category.get_primary_key();
    drop(txn); // Close the read transaction
    
    let category_txn = stores.definition_two.begin_read()?;
    let category: Category = category_txn.read::<Category>(&category_id)?.expect("Category should exist");
    println!("  - Category: {}", category.name);

    println!("\n✅ All features demonstrated successfully using the Repository Pattern!");

    Ok(())
}
