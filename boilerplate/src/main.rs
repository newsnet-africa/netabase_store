// Boilerplate example - Main entry point
//
// Now fully powered by macros! The entire boilerplate is generated automatically.
// Run with: cargo run --bin netabase_store_examples

use netabase_store::blob::NetabaseBlobItem;
use netabase_store::relational::RelationalLink;
use netabase_store::traits::registery::definition::NetabaseDefinition;
use netabase_store::traits::registery::models::model::NetabaseModel;
use netabase_store_examples::boilerplate_lib::definition::{AnotherLargeUserFile, LargeUserFile};
use netabase_store_examples::boilerplate_lib::{
    Category, CategoryID, Definition, DefinitionKeys, DefinitionSubscriptions, DefinitionTreeName,
    DefinitionTreeNames, DefinitionTwo, DefinitionTwoTreeName, Post, PostID, User, UserBlobKeys,
    UserID, UserKeys,
};

fn main() {
    println!("Netabase Store - Boilerplate Example");
    println!("=====================================\n");

    println!("Phase 6: Module structure created");
    println!("- models/user.rs: User model");
    println!("- models/post.rs: Post model");
    println!("- models/category.rs: Category model");
    println!("- mod.rs: Definition & DefinitionTwo\n");

    println!("Testing NetabaseStore type system");

    // Test data creation
    let category_id = CategoryID("cat1".to_string());
    let category = Category {
        id: category_id.clone(),
        name: "General".to_string(),
        description: "A general category".to_string(),
        subscriptions: vec![],
    };

    let user_id = UserID("user1".to_string());
    let alice_bio_data = vec![0u8; 150_000]; // 150KB, should split into 3 blobs (60K, 60K, 30K)
    let alice_another_data = vec![1u8; 700_000]; // 70KB, should split into 2 blobs (60K, 10K)

    let user = User {
        id: user_id.clone(),
        first_name: "Alice".to_string(),
        last_name: "Smith".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(user_id.clone()),
        category: RelationalLink::new_dehydrated(category_id.clone()),
        subscriptions: vec![DefinitionSubscriptions::Topic1],
        bio: LargeUserFile {
            data: alice_bio_data.clone(),
            metadata: "Alice's Bio".to_string(),
        },
        another: AnotherLargeUserFile(alice_another_data.clone()),
    };

    let post_id = PostID("post1".to_string());
    let post = Post {
        id: post_id.clone(),
        title: "Hello World".to_string(),
        author_id: "user1".to_string(),
        content: "This is a test post".to_string(),
        published: true,
        tags: vec!["rust".to_string(), "database".to_string()],
        subscriptions: vec![DefinitionSubscriptions::Topic3],
    };

    println!("Created Category: {:?}", category);
    // Print summary instead of full debug for user to avoid flooding terminal with 220KB of data
    println!(
        "Created User: Alice with {} bytes bio and {} bytes another",
        user.bio.data.len(),
        user.another.0.len()
    );
    println!("Created Post: {:?}", post);

    // Test blob splitting logic
    let blob_entries = user
        .get_blob_entries()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    println!("User blob splitting test:");
    println!("  - Total blob entries: {}", blob_entries.len());
    let bio_blobs = blob_entries
        .iter()
        .filter(|(k, _)| matches!(k, UserBlobKeys::Bio { .. }))
        .count();
    let another_blobs = blob_entries
        .iter()
        .filter(|(k, _)| matches!(k, UserBlobKeys::Another { .. }))
        .count();
    println!("  - LargeUserFile blobs: {} (expected 3)", bio_blobs);
    println!(
        "  - AnotherLargeUserFile blobs: {} (expected 2)",
        another_blobs
    );

    // Test reconstruction
    println!("\nUser blob reconstruction test:");
    let bio_blob_items: Vec<netabase_store_examples::boilerplate_lib::UserBlobItem> = blob_entries
        .iter()
        .filter(|(k, _)| matches!(k, UserBlobKeys::Bio { .. }))
        .map(|(_, v)| v.clone())
        .collect();
    let reconstructed_bio = LargeUserFile::reconstruct_from_blobs(bio_blob_items);
    println!(
        "  - Reconstructed bio length: {}",
        reconstructed_bio.data.len()
    );
    println!(
        "  - Bio matches original: {}",
        reconstructed_bio.data == alice_bio_data
    );

    let another_blob_items: Vec<netabase_store_examples::boilerplate_lib::UserBlobItem> =
        blob_entries
            .iter()
            .filter(|(k, _)| matches!(k, UserBlobKeys::Another { .. }))
            .map(|(_, v)| v.clone())
            .collect();
    let reconstructed_another = AnotherLargeUserFile::reconstruct_from_blobs(another_blob_items);
    println!(
        "  - Reconstructed another length: {}",
        reconstructed_another.0.len()
    );
    println!(
        "  - Another matches original: {}",
        reconstructed_another.0 == alice_another_data
    );

    // Test primary keys
    println!("User primary key: {:?}", user.get_primary_key());
    println!("Post primary key: {:?}", post.get_primary_key());
    println!("Category primary key: {:?}", category.get_primary_key());

    // Test secondary keys
    println!("User secondary keys: {:?}", user.get_secondary_keys());
    println!("Post secondary keys: {:?}", post.get_secondary_keys());
    println!(
        "Category secondary keys: {:?}",
        category.get_secondary_keys()
    );

    // Test relational keys
    println!("User relational keys: {:?}", user.get_relational_keys());
    println!("Post relational keys: {:?}", post.get_relational_keys());
    println!(
        "Category relational keys: {:?}",
        category.get_relational_keys()
    );

    // Test subscription keys
    println!("User subscription keys: {:?}", user.get_subscription_keys());
    println!("Post subscription keys: {:?}", post.get_subscription_keys());
    println!(
        "Category subscription keys: {:?}",
        category.get_subscription_keys()
    );

    // Test discriminants
    println!("User discriminant: {:?}", DefinitionTreeName::User.as_ref());
    println!("Post discriminant: {:?}", DefinitionTreeName::Post.as_ref());
    println!(
        "Category discriminant: {:?}",
        DefinitionTwoTreeName::Category.as_ref()
    );

    // Test tree names structure with formatted table names
    println!("User main table: {}", User::TREE_NAMES.main.table_name);
    println!("User secondary tables:");
    for sec in User::TREE_NAMES.secondary {
        println!("  - {} -> {}", sec.discriminant.as_ref(), sec.table_name);
    }
    println!("User relational tables:");
    for rel in User::TREE_NAMES.relational {
        println!("  - {} -> {}", rel.discriminant.as_ref(), rel.table_name);
    }
    println!("User subscription tables:");
    if let Some(subs) = User::TREE_NAMES.subscription {
        for sub in subs {
            println!("  - {} -> {}", sub.discriminant.as_ref(), sub.table_name);
        }
    }

    println!("Post main table: {}", Post::TREE_NAMES.main.table_name);
    println!("Post secondary tables:");
    for sec in Post::TREE_NAMES.secondary {
        println!("  - {} -> {}", sec.discriminant.as_ref(), sec.table_name);
    }
    println!("Post relational tables:");
    for rel in Post::TREE_NAMES.relational {
        println!("  - {} -> {}", rel.discriminant.as_ref(), rel.table_name);
    }
    println!("Post subscription tables:");
    if let Some(subs) = Post::TREE_NAMES.subscription {
        for sub in subs {
            println!("  - {} -> {}", sub.discriminant.as_ref(), sub.table_name);
        }
    }

    println!(
        "Category main table: {}",
        Category::TREE_NAMES.main.table_name
    );
    println!("Category secondary tables:");
    for sec in Category::TREE_NAMES.secondary {
        println!("  - {} -> {}", sec.discriminant.as_ref(), sec.table_name);
    }
    println!("Category relational tables:");
    for rel in Category::TREE_NAMES.relational {
        println!("  - {} -> {}", rel.discriminant.as_ref(), rel.table_name);
    }
    println!("Category subscription tables:");
    if let Some(subs) = Category::TREE_NAMES.subscription {
        for sub in subs {
            println!("  - {} -> {}", sub.discriminant.as_ref(), sub.table_name);
        }
    }

    // Demonstrate usage of the new DefinitionTreeNames and DefinitionKeys enums
    // (In a real app these would be populated or used for lookups)
    let _tree_names_enum = DefinitionTreeNames::User(User::TREE_NAMES);
    let _keys_enum = DefinitionKeys::User(UserKeys::Primary(user_id.clone()));

    println!("\n=== RelationalLink Variants Demo ===");

    // 1. Dehydrated - No lifetime constraints
    let dehydrated = RelationalLink::<
        netabase_store::traits::registery::repository::Standalone,
        Definition,
        Definition,
        User,
    >::new_dehydrated(user_id.clone());
    println!("1. Dehydrated link:");
    println!("  - is_hydrated: {}", dehydrated.is_hydrated());
    println!("  - is_owned: {}", dehydrated.is_owned());
    println!("  - is_borrowed: {}", dehydrated.is_borrowed());
    println!("  - is_dehydrated: {}", dehydrated.is_dehydrated());
    println!("  - primary_key: {:?}", dehydrated.get_primary_key());

    // 2. Owned - Fully owns the model, no lifetime constraints
    let owned_user = User {
        id: UserID("user2".to_string()),
        first_name: "Bob".to_string(),
        last_name: "Jones".to_string(),
        age: 25,
        partner: RelationalLink::new_dehydrated(user_id.clone()),
        category: RelationalLink::new_dehydrated(category_id.clone()),
        subscriptions: vec![DefinitionSubscriptions::Topic2],
        bio: LargeUserFile {
            data: vec![2u8; 80_000],
            metadata: "".to_string(),
        }, // 80KB -> 2 blobs
        another: AnotherLargeUserFile(vec![3u8; 10_000]), // 10KB -> 1 blob
    };
    let owned = RelationalLink::<
        netabase_store::traits::registery::repository::Standalone,
        Definition,
        Definition,
        User,
    >::new_owned(UserID("user2".to_string()), owned_user);
    println!("\n2. Owned link:");
    println!("  - is_hydrated: {}", owned.is_hydrated());
    println!("  - is_owned: {}", owned.is_owned());
    println!("  - is_borrowed: {}", owned.is_borrowed());
    println!(
        "  - model name: {:?}",
        owned
            .get_model()
            .map(|u| format!("{} {}", u.first_name, u.last_name))
    );
    println!("  - model age: {:?}", owned.get_model().map(|u| u.age));

    // 3. Hydrated - Requires 'data lifetime
    let hydrated = RelationalLink::<
        netabase_store::traits::registery::repository::Standalone,
        Definition,
        Definition,
        User,
    >::new_hydrated(user_id.clone(), &user);
    println!("\n3. Hydrated link:");
    println!("  - is_hydrated: {}", hydrated.is_hydrated());
    println!("  - is_owned: {}", hydrated.is_owned());
    println!("  - is_borrowed: {}", hydrated.is_borrowed());
    println!(
        "  - model name: {:?}",
        hydrated
            .get_model()
            .map(|u| format!("{} {}", u.first_name, u.last_name))
    );

    // 4. Borrowed (simulated - in real usage from AccessGuard)
    let borrowed = RelationalLink::<
        netabase_store::traits::registery::repository::Standalone,
        Definition,
        Definition,
        User,
    >::new_borrowed(user_id.clone(), &user);
    println!("\n4. Borrowed link:");
    println!("  - is_hydrated: {}", borrowed.is_hydrated());
    println!("  - is_owned: {}", borrowed.is_owned());
    println!("  - is_borrowed: {}", borrowed.is_borrowed());
    println!(
        "  - model name: {:?}",
        borrowed
            .as_borrowed()
            .map(|u| format!("{} {}", u.first_name, u.last_name))
    );

    // 5. Demonstrate conversions
    println!("\n5. Conversions:");
    let dehydrated_from_borrowed = borrowed.clone().dehydrate();
    println!("  - Dehydrated from borrowed:");
    println!(
        "    - is_dehydrated: {}",
        dehydrated_from_borrowed.is_dehydrated()
    );

    // Extract owned model
    let extracted = owned.into_owned();
    println!("  - Extracted owned model:");
    println!(
        "    - name: {:?}",
        extracted
            .as_ref()
            .map(|u| format!("{} {}", u.first_name, u.last_name))
    );
    println!("    - age: {:?}", extracted.as_ref().map(|u| u.age));

    // Demonstrate ordering: Dehydrated < Owned < Hydrated < Borrowed
    println!("\n6. Variant ordering (Dehydrated < Owned < Hydrated < Borrowed):");
    let test_dehydrated = RelationalLink::<
        netabase_store::traits::registery::repository::Standalone,
        Definition,
        Definition,
        User,
    >::new_dehydrated(UserID("test".to_string()));
    let test_owned_user = User {
        id: UserID("test".to_string()),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(user_id.clone()),
        category: RelationalLink::new_dehydrated(category_id.clone()),
        subscriptions: vec![],
        bio: LargeUserFile {
            data: vec![4u8; 65_000],
            metadata: "".to_string(),
        }, // 65KB -> 2 blobs
        another: AnotherLargeUserFile(vec![5u8; 5_000]),
    };
    let test_owned = RelationalLink::<
        netabase_store::traits::registery::repository::Standalone,
        Definition,
        Definition,
        User,
    >::new_owned(UserID("test".to_string()), test_owned_user);
    let test_user = User {
        id: UserID("test".to_string()),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(user_id.clone()),
        category: RelationalLink::new_dehydrated(category_id.clone()),
        subscriptions: vec![],
        bio: LargeUserFile {
            data: vec![6u8; 125_000],
            metadata: "".to_string(),
        }, // 125KB -> 3 blobs
        another: AnotherLargeUserFile(vec![7u8; 2_000]),
    };
    let test_hydrated = RelationalLink::<
        netabase_store::traits::registery::repository::Standalone,
        Definition,
        Definition,
        User,
    >::new_hydrated(UserID("test".to_string()), &test_user);
    let test_borrowed = RelationalLink::<
        netabase_store::traits::registery::repository::Standalone,
        Definition,
        Definition,
        User,
    >::new_borrowed(UserID("test".to_string()), &test_user);

    println!("  - Dehydrated < Owned: {}", test_dehydrated < test_owned);
    println!("  - Owned < Hydrated: {}", test_owned < test_hydrated);
    println!("  - Hydrated < Borrowed: {}", test_hydrated < test_borrowed);

    println!("\n=== Subscription Registry Demo ===");
    let reg = &<Definition as NetabaseDefinition>::SUBSCRIPTION_REGISTRY;
    println!("Definition subscription registry:");
    println!(
        "  - Topic1 subscribers: {:?}",
        reg.get_subscribers("Topic1")
    );
    println!(
        "  - Topic3 subscribers: {:?}",
        reg.get_subscribers("Topic3")
    );
    println!(
        "  - User subscribes to: {:?}",
        reg.get_model_topics(DefinitionTreeName::User)
    );
    println!(
        "  - Post subscribes to: {:?}",
        reg.get_model_topics(DefinitionTreeName::Post)
    );
    println!(
        "  - Does User subscribe to Topic1? {}",
        reg.model_subscribes_to("Topic1", DefinitionTreeName::User)
    );
    println!(
        "  - Does Post subscribe to Topic1? {}",
        reg.model_subscribes_to("Topic1", DefinitionTreeName::Post)
    );

    let reg2 = &<DefinitionTwo as NetabaseDefinition>::SUBSCRIPTION_REGISTRY;
    println!("\nDefinitionTwo subscription registry:");
    println!(
        "  - General subscribers: {:?}",
        reg2.get_subscribers("General")
    );

    println!("\n✅ All features demonstrated successfully!");
    println!("\nType system test completed successfully!");
}
