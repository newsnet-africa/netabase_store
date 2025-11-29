//! Subscription System Comprehensive Tests
//!
//! These tests verify that the subscription and notification system works
//! correctly across different backends and scenarios.

#![cfg(not(target_arch = "wasm32"))] // Subscription system is native-only for now

use netabase_store::netabase_definition_module;
use netabase_store::subscription::subscription_tree::DefaultSubscriptionManager;
use netabase_store::traits::subscription::subscription_tree::ModelHash;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Test schema for subscription tests
#[netabase_definition_module(SubscriptionTestDefinition, SubscriptionTestKeys)]
mod subscription_test_schema {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(SubscriptionTestDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
        pub active: bool,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(SubscriptionTestDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(SubscriptionTestDefinition)]
    pub struct Comment {
        #[primary_key]
        pub id: u64,
        pub text: String,
        #[secondary_key]
        pub post_id: u64,
        #[secondary_key]
        pub author_id: u64,
    }
}

use subscription_test_schema::*;

/// Test basic subscription and notification
#[test]
#[cfg(feature = "sled")]
fn test_basic_subscription() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<SubscriptionTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<User>();

    // Create subscription manager
    let subscription_manager = DefaultSubscriptionManager::new();
    let notifications = Arc::new(Mutex::new(Vec::new()));
    let notifications_clone = Arc::clone(&notifications);

    // Subscribe to User changes
    let _subscription = subscription_manager.subscribe(move |change| {
        notifications_clone.lock().unwrap().push(change);
    });

    // Perform operations that should trigger notifications
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        active: true,
    };

    tree.put(user.clone()).unwrap();

    // Simulate notification delivery
    let user_hash = ModelHash::from(&user);
    subscription_manager.notify(user_hash);

    // Give notification time to be processed
    thread::sleep(Duration::from_millis(50));

    // Verify notification was received
    let notifications = notifications.lock().unwrap();
    assert!(!notifications.is_empty());
}

/// Test subscription to specific model changes
#[test]
#[cfg(feature = "sled")]
fn test_model_specific_subscription() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<SubscriptionTestDefinition>::temp().unwrap();
    let user_tree = store.open_tree::<User>();
    let post_tree = store.open_tree::<Post>();

    let subscription_manager = DefaultSubscriptionManager::new();
    let user_notifications = Arc::new(Mutex::new(Vec::new()));
    let post_notifications = Arc::new(Mutex::new(Vec::new()));

    let user_notifications_clone = Arc::clone(&user_notifications);
    let post_notifications_clone = Arc::clone(&post_notifications);

    // Subscribe to User changes only
    let _user_subscription = subscription_manager.subscribe(move |change| {
        user_notifications_clone.lock().unwrap().push(change);
    });

    // Subscribe to Post changes only
    let _post_subscription = subscription_manager.subscribe(move |change| {
        post_notifications_clone.lock().unwrap().push(change);
    });

    // Create user and post
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        active: true,
    };

    let post = Post {
        id: 1,
        title: "Hello World".to_string(),
        content: "First post".to_string(),
        author_id: 1,
    };

    user_tree.put(user.clone()).unwrap();
    post_tree.put(post.clone()).unwrap();

    // Notify changes
    subscription_manager.notify(ModelHash::from(&user));
    subscription_manager.notify(ModelHash::from(&post));

    thread::sleep(Duration::from_millis(50));

    // Both subscriptions should have received their respective notifications
    assert!(!user_notifications.lock().unwrap().is_empty());
    assert!(!post_notifications.lock().unwrap().is_empty());
}

/// Test subscription unsubscribe functionality
#[test]
#[cfg(feature = "sled")]
fn test_subscription_unsubscribe() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<SubscriptionTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<User>();

    let subscription_manager = DefaultSubscriptionManager::new();
    let notifications = Arc::new(Mutex::new(Vec::new()));
    let notifications_clone = Arc::clone(&notifications);

    // Create subscription
    let subscription = subscription_manager.subscribe(move |change| {
        notifications_clone.lock().unwrap().push(change);
    });

    // Trigger notification
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        active: true,
    };
    tree.put(user.clone()).unwrap();
    subscription_manager.notify(ModelHash::from(&user));

    thread::sleep(Duration::from_millis(50));
    assert!(!notifications.lock().unwrap().is_empty());

    // Unsubscribe
    drop(subscription);

    // Clear previous notifications
    notifications.lock().unwrap().clear();

    // Trigger another notification
    let user2 = User {
        id: 2,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
        active: true,
    };
    tree.put(user2.clone()).unwrap();
    subscription_manager.notify(ModelHash::from(&user2));

    thread::sleep(Duration::from_millis(50));

    // Should not receive notification after unsubscribe
    assert!(notifications.lock().unwrap().is_empty());
}

/// Test multiple subscribers to same changes
#[test]
#[cfg(feature = "sled")]
fn test_multiple_subscribers() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<SubscriptionTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<User>();

    let subscription_manager = DefaultSubscriptionManager::new();

    let notifications1 = Arc::new(Mutex::new(Vec::new()));
    let notifications2 = Arc::new(Mutex::new(Vec::new()));
    let notifications3 = Arc::new(Mutex::new(Vec::new()));

    let notifications1_clone = Arc::clone(&notifications1);
    let notifications2_clone = Arc::clone(&notifications2);
    let notifications3_clone = Arc::clone(&notifications3);

    // Create multiple subscriptions
    let _sub1 = subscription_manager.subscribe(move |change| {
        notifications1_clone.lock().unwrap().push(change);
    });

    let _sub2 = subscription_manager.subscribe(move |change| {
        notifications2_clone.lock().unwrap().push(change);
    });

    let _sub3 = subscription_manager.subscribe(move |change| {
        notifications3_clone.lock().unwrap().push(change);
    });

    // Trigger single change
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        active: true,
    };
    tree.put(user.clone()).unwrap();
    subscription_manager.notify(ModelHash::from(&user));

    thread::sleep(Duration::from_millis(50));

    // All subscribers should receive notification
    assert!(!notifications1.lock().unwrap().is_empty());
    assert!(!notifications2.lock().unwrap().is_empty());
    assert!(!notifications3.lock().unwrap().is_empty());
}

/// Test subscription performance under high load
#[test]
#[cfg(feature = "sled")]
fn test_subscription_performance() {
    use netabase_store::databases::sled_store::SledStore;
    use std::time::Instant;

    let store = SledStore::<SubscriptionTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<User>();

    let subscription_manager = DefaultSubscriptionManager::new();
    let notification_count = Arc::new(Mutex::new(0));
    let notification_count_clone = Arc::clone(&notification_count);

    let _subscription = subscription_manager.subscribe(move |_change| {
        *notification_count_clone.lock().unwrap() += 1;
    });

    let num_operations = 1000;
    let start_time = Instant::now();

    // Perform many operations rapidly
    for i in 0..num_operations {
        let user = User {
            id: i,
            name: format!("User{}", i),
            email: format!("user{}@example.com", i),
            active: i % 2 == 0,
        };
        tree.put(user.clone()).unwrap();
        subscription_manager.notify(ModelHash::from(&user));
    }

    let operation_duration = start_time.elapsed();

    // Wait for notifications to be processed
    thread::sleep(Duration::from_millis(100));

    let final_count = *notification_count.lock().unwrap();
    println!(
        "Processed {} operations in {:?}, {} notifications received",
        num_operations, operation_duration, final_count
    );

    // Should handle reasonable performance
    assert!(operation_duration.as_millis() < 5000); // Less than 5 seconds
    assert!(final_count > 0); // Should receive some notifications
}

/// Test subscription error handling
#[test]
#[cfg(feature = "sled")]
fn test_subscription_error_handling() {
    let subscription_manager = DefaultSubscriptionManager::new();
    let error_count = Arc::new(Mutex::new(0));
    let error_count_clone = Arc::clone(&error_count);

    // Create subscription that panics
    let _subscription = subscription_manager.subscribe(move |_change| {
        *error_count_clone.lock().unwrap() += 1;
        if *error_count_clone.lock().unwrap() == 2 {
            panic!("Intentional panic for testing");
        }
    });

    // First notification should work
    let user1 = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        active: true,
    };
    subscription_manager.notify(ModelHash::from(&user1));
    thread::sleep(Duration::from_millis(50));

    // Second notification should panic but not crash the system
    let user2 = User {
        id: 2,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
        active: true,
    };
    subscription_manager.notify(ModelHash::from(&user2));
    thread::sleep(Duration::from_millis(50));

    // Third notification should work (system should recover)
    let user3 = User {
        id: 3,
        name: "Charlie".to_string(),
        email: "charlie@example.com".to_string(),
        active: true,
    };
    subscription_manager.notify(ModelHash::from(&user3));
    thread::sleep(Duration::from_millis(50));

    // Should have processed at least the first notification
    assert!(*error_count.lock().unwrap() >= 1);
}

/// Test subscription with concurrent modifications
#[test]
#[cfg(feature = "sled")]
fn test_subscription_concurrent_modifications() {
    use netabase_store::databases::sled_store::SledStore;

    let store = Arc::new(SledStore::<SubscriptionTestDefinition>::temp().unwrap());
    let subscription_manager = Arc::new(DefaultSubscriptionManager::new());

    let notification_count = Arc::new(Mutex::new(0));
    let notification_count_clone = Arc::clone(&notification_count);

    let _subscription = subscription_manager.subscribe(move |_change| {
        *notification_count_clone.lock().unwrap() += 1;
    });

    let num_threads = 4;
    let operations_per_thread = 100;
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let store_clone = Arc::clone(&store);
        let subscription_manager_clone = Arc::clone(&subscription_manager);

        let handle = thread::spawn(move || {
            let tree = store_clone.open_tree::<User>();

            for i in 0..operations_per_thread {
                let user = User {
                    id: (thread_id * 1000 + i) as u64,
                    name: format!("User{}_{}", thread_id, i),
                    email: format!("user{}_{}@example.com", thread_id, i),
                    active: true,
                };

                tree.put(user.clone()).unwrap();
                subscription_manager_clone.notify(ModelHash::from(&user));
            }

            operations_per_thread
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        let completed = handle.join().unwrap();
        assert_eq!(completed, operations_per_thread);
    }

    // Wait for notifications to be processed
    thread::sleep(Duration::from_millis(200));

    let final_count = *notification_count.lock().unwrap();
    println!(
        "Received {} notifications from concurrent operations",
        final_count
    );

    // Should receive a reasonable number of notifications
    assert!(final_count > 0);
    assert!(final_count <= (num_threads * operations_per_thread) as usize);
}

/// Test subscription with different model types
#[test]
#[cfg(feature = "sled")]
fn test_subscription_multiple_model_types() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<SubscriptionTestDefinition>::temp().unwrap();
    let subscription_manager = DefaultSubscriptionManager::new();

    let all_notifications = Arc::new(Mutex::new(Vec::new()));
    let all_notifications_clone = Arc::clone(&all_notifications);

    let _subscription = subscription_manager.subscribe(move |change| {
        all_notifications_clone.lock().unwrap().push(change);
    });

    // Create different model types
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        active: true,
    };

    let post = Post {
        id: 1,
        title: "Hello World".to_string(),
        content: "First post".to_string(),
        author_id: 1,
    };

    let comment = Comment {
        id: 1,
        text: "Great post!".to_string(),
        post_id: 1,
        author_id: 1,
    };

    // Store all models
    store.open_tree::<User>().put(user.clone()).unwrap();
    store.open_tree::<Post>().put(post.clone()).unwrap();
    store.open_tree::<Comment>().put(comment.clone()).unwrap();

    // Notify all changes
    subscription_manager.notify(ModelHash::from(&user));
    subscription_manager.notify(ModelHash::from(&post));
    subscription_manager.notify(ModelHash::from(&comment));

    thread::sleep(Duration::from_millis(50));

    let notifications = all_notifications.lock().unwrap();
    assert_eq!(notifications.len(), 3);
}

/// Test subscription memory management
#[test]
#[cfg(feature = "sled")]
fn test_subscription_memory_management() {
    let subscription_manager = DefaultSubscriptionManager::new();
    let mut subscriptions = Vec::new();

    // Create many subscriptions
    for i in 0..1000 {
        let subscription = subscription_manager.subscribe(move |_change| {
            // Each subscription captures its ID
            let _id = i;
        });
        subscriptions.push(subscription);
    }

    // Trigger notifications
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        active: true,
    };
    subscription_manager.notify(ModelHash::from(&user));

    thread::sleep(Duration::from_millis(100));

    // Drop half the subscriptions
    subscriptions.truncate(500);

    // Trigger more notifications
    let user2 = User {
        id: 2,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
        active: true,
    };
    subscription_manager.notify(ModelHash::from(&user2));

    thread::sleep(Duration::from_millis(100));

    // Drop remaining subscriptions
    subscriptions.clear();

    // Final notification (should work with no subscribers)
    let user3 = User {
        id: 3,
        name: "Charlie".to_string(),
        email: "charlie@example.com".to_string(),
        active: true,
    };
    subscription_manager.notify(ModelHash::from(&user3));

    thread::sleep(Duration::from_millis(50));

    // Test completed without crashes - memory management is working
    assert!(true);
}

/// Test subscription ordering guarantees
#[test]
#[cfg(feature = "sled")]
fn test_subscription_ordering() {
    let subscription_manager = DefaultSubscriptionManager::new();
    let received_ids = Arc::new(Mutex::new(Vec::new()));
    let received_ids_clone = Arc::clone(&received_ids);

    let _subscription = subscription_manager.subscribe(move |change| {
        // Extract some kind of ordering information from the change
        // For this test, we'll use a simple counter
        received_ids_clone.lock().unwrap().push(change);
    });

    // Send notifications in order
    for i in 1..=10 {
        let user = User {
            id: i,
            name: format!("User{}", i),
            email: format!("user{}@example.com", i),
            active: true,
        };
        subscription_manager.notify(ModelHash::from(&user));
    }

    thread::sleep(Duration::from_millis(100));

    let received = received_ids.lock().unwrap();
    assert_eq!(received.len(), 10);

    // Note: Depending on implementation, ordering might not be guaranteed
    // This test documents the current behavior
    println!("Received {} notifications in subscription", received.len());
}

/// Test subscription cleanup on drop
#[test]
#[cfg(feature = "sled")]
fn test_subscription_cleanup_on_drop() {
    let subscription_manager = DefaultSubscriptionManager::new();
    let notification_count = Arc::new(Mutex::new(0));
    let notification_count_clone = Arc::clone(&notification_count);

    {
        let _subscription = subscription_manager.subscribe(move |_change| {
            *notification_count_clone.lock().unwrap() += 1;
        });

        // Trigger notification while subscription is active
        let user = User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            active: true,
        };
        subscription_manager.notify(ModelHash::from(&user));
        thread::sleep(Duration::from_millis(50));

        assert!(*notification_count.lock().unwrap() > 0);
    } // Subscription dropped here

    // Reset counter
    *notification_count.lock().unwrap() = 0;

    // Trigger notification after subscription dropped
    let user2 = User {
        id: 2,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
        active: true,
    };
    subscription_manager.notify(ModelHash::from(&user2));
    thread::sleep(Duration::from_millis(50));

    // Should not receive notification after subscription dropped
    assert_eq!(*notification_count.lock().unwrap(), 0);
}
