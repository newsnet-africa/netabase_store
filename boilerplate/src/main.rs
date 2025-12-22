// Boilerplate example - Main entry point
//
// This example has been restructured into modules:
// - boilerplate_lib/models/user.rs - User model
// - boilerplate_lib/models/post.rs - Post model
// - boilerplate_lib/mod.rs - Definition
//
// Run with: cargo run --example boilerplate

// mod boilerplate_lib; // Removed, now using library crate

use netabase_store_examples::boilerplate_lib;
use netabase_store_examples::boilerplate_lib::Definition;
use netabase_store_examples::boilerplate_lib::DefinitionDiscriminants;
use netabase_store_examples::boilerplate_lib::DefinitionKeys;
use netabase_store_examples::boilerplate_lib::DefinitionSubscriptions;
use netabase_store_examples::boilerplate_lib::DefinitionTreeNames;
use netabase_store_examples::boilerplate_lib::DefinitionTwo;
use netabase_store_examples::boilerplate_lib::DefinitionTwoDiscriminants;
use netabase_store_examples::boilerplate_lib::GlobalDefinitionKeys;
use netabase_store_examples::boilerplate_lib::GlobalKeys;
use netabase_store_examples::boilerplate_lib::models::post::{Post, PostID};
use netabase_store_examples::boilerplate_lib::models::user::{User, UserID, UserKeys, LargeUserFile, AnotherLargeUserFile};
use netabase_store::relational::RelationalLink;
use netabase_store::traits::registery::definition::NetabaseDefinition;
use netabase_store::traits::registery::models::model::NetabaseModel;
use strum::AsRefStr;

use netabase_store_examples::boilerplate_lib::Category;
use netabase_store_examples::boilerplate_lib::CategoryID;
use netabase_store_examples::boilerplate_lib::DefinitionTwoTreeNames;

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
    };

    let user_id = UserID("user1".to_string());
    let user = User {
        id: user_id.clone(),
        name: "Alice".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(user_id.clone()),
        category: RelationalLink::new_dehydrated(category_id.clone()),
        subscriptions: vec![DefinitionSubscriptions::Topic1],
        bio: LargeUserFile(Vec::new()),
        another: AnotherLargeUserFile(Vec::new()),
    };

    let post_id = PostID("post1".to_string());
    let post = Post {
        id: post_id.clone(),
        title: "Hello World".to_string(),
        author: RelationalLink::new_dehydrated(user_id.clone()),
    };

    println!("Created Category: {:?}", category);
    println!("Created User: {:?}", user);
    println!("Created Post: {:?}", post);

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
    println!(
        "User discriminant: {:?}",
        DefinitionDiscriminants::User.as_ref()
    );
    println!(
        "Post discriminant: {:?}",
        DefinitionDiscriminants::Post.as_ref()
    );
    println!(
        "Category discriminant: {:?}",
        DefinitionTwoDiscriminants::Category.as_ref()
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
    let dehydrated =
        RelationalLink::<Definition, Definition, User>::new_dehydrated(user_id.clone());
    println!("1. Dehydrated link:");
    println!("  - is_hydrated: {}", dehydrated.is_hydrated());
    println!("  - is_owned: {}", dehydrated.is_owned());
    println!("  - is_borrowed: {}", dehydrated.is_borrowed());
    println!("  - is_dehydrated: {}", dehydrated.is_dehydrated());
    println!("  - primary_key: {:?}", dehydrated.get_primary_key());

    // 2. Owned - Fully owns the model, no lifetime constraints
    let owned_user = User {
        id: UserID("user2".to_string()),
        name: "Bob".to_string(),
        age: 25,
        partner: RelationalLink::new_dehydrated(user_id.clone()),
        category: RelationalLink::new_dehydrated(category_id.clone()),
        subscriptions: vec![DefinitionSubscriptions::Topic2],
        bio: LargeUserFile(Vec::new()),
        another: AnotherLargeUserFile(Vec::new()),
    };
    let owned = RelationalLink::<Definition, Definition, User>::new_owned(
        UserID("user2".to_string()),
        owned_user,
    );
    println!("\n2. Owned link:");
    println!("  - is_hydrated: {}", owned.is_hydrated());
    println!("  - is_owned: {}", owned.is_owned());
    println!("  - is_borrowed: {}", owned.is_borrowed());
    println!("  - model name: {:?}", owned.get_model().map(|u| &u.name));
    println!("  - model age: {:?}", owned.get_model().map(|u| u.age));

    // 3. Hydrated - Requires 'data lifetime
    let hydrated =
        RelationalLink::<Definition, Definition, User>::new_hydrated(user_id.clone(), &user);
    println!("\n3. Hydrated link:");
    println!("  - is_hydrated: {}", hydrated.is_hydrated());
    println!("  - is_owned: {}", hydrated.is_owned());
    println!("  - is_borrowed: {}", hydrated.is_borrowed());
    println!(
        "  - model name: {:?}",
        hydrated.get_model().map(|u| &u.name)
    );

    // 4. Borrowed (simulated - in real usage from AccessGuard)
    let borrowed =
        RelationalLink::<Definition, Definition, User>::new_borrowed(user_id.clone(), &user);
    println!("\n4. Borrowed link:");
    println!("  - is_hydrated: {}", borrowed.is_hydrated());
    println!("  - is_owned: {}", borrowed.is_owned());
    println!("  - is_borrowed: {}", borrowed.is_borrowed());
    println!(
        "  - model name: {:?}",
        borrowed.as_borrowed().map(|u| &u.name)
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
    println!("    - name: {:?}", extracted.as_ref().map(|u| &u.name));
    println!("    - age: {:?}", extracted.as_ref().map(|u| u.age));

    // Demonstrate ordering: Dehydrated < Owned < Hydrated < Borrowed
    println!("\n6. Variant ordering (Dehydrated < Owned < Hydrated < Borrowed):");
    let test_dehydrated =
        RelationalLink::<Definition, Definition, User>::new_dehydrated(UserID("test".to_string()));
    let test_owned_user = User {
        id: UserID("test".to_string()),
        name: "Test".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(user_id.clone()),
        category: RelationalLink::new_dehydrated(category_id.clone()),
        subscriptions: vec![],
        bio: LargeUserFile(Vec::new()),
        another: AnotherLargeUserFile(Vec::new()),
    };
    let test_owned = RelationalLink::<Definition, Definition, User>::new_owned(
        UserID("test".to_string()),
        test_owned_user,
    );
    let test_user = User {
        id: UserID("test".to_string()),
        name: "Test".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(user_id.clone()),
        category: RelationalLink::new_dehydrated(category_id.clone()),
        subscriptions: vec![],
        bio: LargeUserFile(Vec::new()),
        another: AnotherLargeUserFile(Vec::new()),
    };
    let test_hydrated = RelationalLink::<Definition, Definition, User>::new_hydrated(
        UserID("test".to_string()),
        &test_user,
    );
    let test_borrowed = RelationalLink::<Definition, Definition, User>::new_borrowed(
        UserID("test".to_string()),
        &test_user,
    );

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
        reg.get_model_topics(DefinitionDiscriminants::User)
    );
    println!(
        "  - Post subscribes to: {:?}",
        reg.get_model_topics(DefinitionDiscriminants::Post)
    );
    println!(
        "  - Does User subscribe to Topic1? {}",
        reg.model_subscribes_to("Topic1", DefinitionDiscriminants::User)
    );
    println!(
        "  - Does Post subscribe to Topic1? {}",
        reg.model_subscribes_to("Topic1", DefinitionDiscriminants::Post)
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
