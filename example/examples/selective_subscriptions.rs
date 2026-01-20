// Example: Selective Subscription Control
//
// This example demonstrates the new create_with_subscriptions() API
// which allows fine-grained control over which subscription topics
// a model instance subscribes to.

use netabase_store::databases::redb::RedbStore;
use netabase_store::relational::RelationalLink;
use netabase_store::traits::database::store::NBStore;
use example::boilerplate_lib::definition::{AnotherLargeUserFile, LargeUserFile};
use example::boilerplate_lib::{
    CategoryID, Definition, DefinitionSubscriptions, User, UserID,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Selective Subscription Control Example ===\n");

    // Create an in-memory database
    let (store, _temp) = RedbStore::<Definition>::new_temporary()?;

    println!("1. Creating users with different subscription strategies\n");

    // Example 1: Default behavior - subscribe to all model topics
    println!("Example 1: Default subscription (all topics)");
    {
        let txn = store.begin_write()?;
        let user = User {
            id: UserID("alice".into()),
            first_name: "Alice".into(),
            last_name: "Smith".into(),
            age: 30,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("tech".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };

        // Using create() subscribes to all model topics (Topic1, Topic2)
        txn.create::<User>(&user)?;
        txn.commit()?;
        println!("  ✓ Alice created with default subscriptions (Topic1, Topic2)");
    }

    // Example 2: Selective subscription - only Topic1
    println!("\nExample 2: Selective subscription (Topic1 only)");
    {
        let txn = store.begin_write()?;
        let user = User {
            id: UserID("bob".into()),
            first_name: "Bob".into(),
            last_name: "Jones".into(),
            age: 25,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("sports".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };

        // Subscribe only to Topic1
        let topics = vec![DefinitionSubscriptions::Topic1];
        txn.create_with_subscriptions::<User>(&user, Some(topics))?;
        txn.commit()?;
        println!("  ✓ Bob created with Topic1 subscription only");
    }

    // Example 3: No subscriptions
    println!("\nExample 3: No subscriptions");
    {
        let txn = store.begin_write()?;
        let user = User {
            id: UserID("charlie".into()),
            first_name: "Charlie".into(),
            last_name: "Brown".into(),
            age: 35,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("none".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };

        // Subscribe to no topics
        txn.create_with_subscriptions::<User>(&user, Some(vec![]))?;
        txn.commit()?;
        println!("  ✓ Charlie created with no subscriptions");
    }

    println!("\n2. Querying by subscription topics\n");

    // Query Topic1
    {
        let txn = store.begin_read()?;
        let results = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
        println!("Topic1 subscribers: {} users", results.len());
        for hash in &results {
            println!("  - User (hash: {}...)", &hash.to_hex()[..16]);
        }
    }

    // Query Topic2
    {
        let txn = store.begin_read()?;
        let results = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic2)?;
        println!("\nTopic2 subscribers: {} users", results.len());
        for hash in &results {
            println!("  - User (hash: {}...)", &hash.to_hex()[..16]);
        }
    }

    println!("\n3. Real-world use cases\n");

    // Use case: Role-based access control
    println!("Use Case 1: Role-based access control");
    {
        let txn = store.begin_write()?;

        // Admin user - gets all topics
        let admin = User {
            id: UserID("admin1".into()),
            first_name: "Admin".into(),
            last_name: "User".into(),
            age: 40,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("admin".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };
        txn.create_with_subscriptions::<User>(&admin, None)?; // All topics

        // Free user - gets only public topic
        let free_user = User {
            id: UserID("free1".into()),
            first_name: "Free".into(),
            last_name: "User".into(),
            age: 20,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("public".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };
        let public_topics = vec![DefinitionSubscriptions::Topic1]; // Topic1 = Public
        txn.create_with_subscriptions::<User>(&free_user, Some(public_topics))?;

        txn.commit()?;
        println!("  ✓ Admin user: all topics");
        println!("  ✓ Free user: public topic only");
    }

    // Use case: Feature flags
    println!("\nUse Case 2: Feature flags (beta access)");
    {
        let txn = store.begin_write()?;

        let beta_user = User {
            id: UserID("beta1".into()),
            first_name: "Beta".into(),
            last_name: "Tester".into(),
            age: 28,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("beta".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };

        // Beta users get access to experimental features (Topic2)
        let beta_topics = vec![
            DefinitionSubscriptions::Topic1, // Public features
            DefinitionSubscriptions::Topic2, // Beta features
        ];
        txn.create_with_subscriptions::<User>(&beta_user, Some(beta_topics))?;

        txn.commit()?;
        println!("  ✓ Beta user: public + beta topics");
    }

    println!("\n4. Summary\n");
    println!("API Usage:");
    println!("  - create(&model)                       → Subscribe to all model topics");
    println!("  - create_with_subscriptions(&m, None)  → Subscribe to all model topics");
    println!("  - create_with_subscriptions(&m, Some(vec![T1])) → Subscribe to specific topics");
    println!("  - create_with_subscriptions(&m, Some(vec![])) → Subscribe to no topics");
    println!("\nBenefits:");
    println!("  ✓ Privacy control (public vs private users)");
    println!("  ✓ Feature flags (beta access)");
    println!("  ✓ Sharding (different instances sync different topics)");
    println!("  ✓ Access control (role-based topics)");

    Ok(())
}
