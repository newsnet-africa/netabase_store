#[path = "common/mod.rs"]
mod common;

use netabase_store::relational::RelationalLink;
use example::boilerplate_lib::definition::{
    AnotherLargeUserFile, LargeUserFile, User, UserID,
};
use example::boilerplate_lib::{CategoryID, Definition, DefinitionSubscriptions};
use netabase_store::databases::redb::transaction::RedbModelCrud;

#[test]
fn debug_subscription_tables() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("debug_sub")?;

    // Create user with subscription
    {
        let txn = store.begin_write()?;
        let user = User {
            id: UserID("user1".into()),
            first_name: "Alice".into(),
            last_name: "Smith".into(),
            age: 30,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };
        
        println!("Creating user (subscriptions are trait-level, not instance data)");
        
        // Check what get_subscription_keys returns
        use netabase_store::traits::registery::models::model::NetabaseModel;
        let sub_keys = user.get_subscription_keys();
        println!("get_subscription_keys() returned: {} keys", sub_keys.len());
        
        txn.create(&user)?;
        txn.commit()?;
    }

    // Read back to verify
    {
        let txn = store.begin_read()?;
        let user: Option<User> = txn.read(&UserID("user1".into()))?;
        
        // Subscriptions are trait-level, verify via get_subscription_keys()
        use netabase_store::traits::registery::models::model::NetabaseModel;
        if let Some(ref u) = user {
            let subs = u.get_subscription_keys();
            println!("Read back user subscription keys: {} topics", subs.len());
        }
    }

    // Try to query using prepare_model to see table details
    {
        let txn = store.begin_read()?;
        let tables = txn.prepare_model::<User>()?;
        
        println!("Subscription tables count: {}", tables.subscription.len());
        for (i, (_, name)) in tables.subscription.iter().enumerate() {
            println!("  Subscription table {}: {}", i, name);
        }
        
        // Try query
        let primary_keys = User::query_by_subscription(&DefinitionSubscriptions::Topic1, &tables)?;
        println!("Query returned {} primary keys", primary_keys.len());
    }

    common::cleanup_test_db(db_path);
    Ok(())
}
