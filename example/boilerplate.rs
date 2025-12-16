// Boilerplate example - Main entry point
//
// This example has been restructured into modules:
// - boilerplate_lib/models/user.rs - User model
// - boilerplate_lib/models/post.rs - Post model
// - boilerplate_lib/mod.rs - Definition
//
// Run with: cargo run --example boilerplate

mod boilerplate_lib;

use boilerplate_lib::DefinitionDiscriminants;
use boilerplate_lib::DefinitionKeys;
use boilerplate_lib::DefinitionTreeNames;
use boilerplate_lib::DefinitionSubscriptions;
use boilerplate_lib::DefinitionTwoDiscriminants;
use boilerplate_lib::models::user::{User, UserID, UserKeys};
use boilerplate_lib::models::post::{Post, PostID};
use boilerplate_lib::models::category::{Category, CategoryID};
use netabase_store::traits::registery::models::model::NetabaseModel;
use netabase_store::relational::RelationalLink;
use strum::AsRefStr;

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
    println!("Category secondary keys: {:?}", category.get_secondary_keys());

    // Test relational keys
    println!("User relational keys: {:?}", user.get_relational_keys());
    println!("Post relational keys: {:?}", post.get_relational_keys());
    println!("Category relational keys: {:?}", category.get_relational_keys());
    
    // Test subscription keys
    println!("User subscription keys: {:?}", user.get_subscription_keys());
    println!("Post subscription keys: {:?}", post.get_subscription_keys());
    println!("Category subscription keys: {:?}", category.get_subscription_keys());

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

    println!("Category main table: {}", Category::TREE_NAMES.main.table_name);
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
    let _keys_enum = DefinitionKeys::User(UserKeys::Primary(user_id));

    println!("Type system test completed successfully!");
}