use netabase_store::databases::redb::transaction::crud::RedbModelCrud;
use netabase_store_examples::boilerplate_lib::{
    ImmutablePost, ImmutablePostEnvelope, MainRepositoryStores, definition::DefinitionSubscriptions,
};

#[test]
fn test_content_addressed_crud() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join(format!(
        "netabase_ca_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir)?;

    // Use generated Stores struct which provides typed access to each definition
    let stores = MainRepositoryStores::new(&temp_dir)?;

    let post1 = ImmutablePost {
        author: "Alice".to_string(),
        content: "Hello World".to_string(),
        timestamp: 1000,
    };

    let post2 = ImmutablePost {
        author: "Bob".to_string(),
        content: "Hello World".to_string(), // Same content, different author -> different hash
        timestamp: 1000,
    };

    let post3 = ImmutablePost {
        author: "Alice".to_string(),
        content: "Another post".to_string(),
        timestamp: 2000,
    };

    // 1. Insert posts
    {
        // Access the specific definition store
        let txn = stores.definition.begin_write()?;

        // Wrap posts in envelopes for insertion
        let env1 = ImmutablePostEnvelope::from(&post1);
        let env2 = ImmutablePostEnvelope::from(&post2);
        let env3 = ImmutablePostEnvelope::from(&post3);

        txn.create(&env1)?;
        txn.create(&env2)?;
        txn.create(&env3)?;

        txn.commit()?;
    }

    // 2. Read back by hash (Primary Key)
    {
        let txn = stores.definition.begin_read()?;

        // Compute hashes manually to query
        use netabase_store_examples::boilerplate_lib::definition::ImmutablePostID;
        use netabase_store_examples::boilerplate_lib::models::hash_model;

        let hash1 = ImmutablePostID(hash_model(&post1));
        let hash2 = ImmutablePostID(hash_model(&post2));

        // Retrieve using the hash (wrapper type)
        let retrieved1 = txn.read::<ImmutablePostEnvelope>(&hash1)?;
        assert!(retrieved1.is_some());
        let env1 = retrieved1.unwrap();
        assert_eq!(env1.inner.author, "Alice");
        assert_eq!(env1.hash, hash1); // Verify hash matches

        let retrieved2 = txn.read::<ImmutablePostEnvelope>(&hash2)?;
        assert!(retrieved2.is_some());
        assert_eq!(retrieved2.unwrap().inner.author, "Bob");
    }

    // 3. Query by Secondary Key
    {
        let txn = stores.definition.begin_read()?;

        use netabase_store_examples::boilerplate_lib::definition::{
            ImmutablePostAuthor, ImmutablePostSecondaryKeys,
        };

        // Find Alice's posts
        let alice_keys = txn.query_by_secondary_key::<ImmutablePostEnvelope>(
            &ImmutablePostSecondaryKeys::Author(ImmutablePostAuthor("Alice".to_string())),
        )?;
        assert_eq!(alice_keys.len(), 2); // post1 and post3

        // Find Bob's posts
        let bob_keys = txn.query_by_secondary_key::<ImmutablePostEnvelope>(
            &ImmutablePostSecondaryKeys::Author(ImmutablePostAuthor("Bob".to_string())),
        )?;
        assert_eq!(bob_keys.len(), 1); // post2
    }

    // 4. Idempotency (Insert same content again)
    {
        let txn = stores.definition.begin_write()?;
        let env1 = ImmutablePostEnvelope::from(&post1);

        // Insert post1 again
        txn.create(&env1)?;

        txn.commit()?;
    }

    // Verify count hasn't changed (still 3 unique posts)
    {
        let txn = stores.definition.begin_read()?;
        // count method is on the trait/impl, reachable via the store or envelope type helper
        // RedbTransaction doesn't have count(), but we can use list or prepare_model
        let tables = txn.prepare_model::<ImmutablePostEnvelope>()?;
        let count = ImmutablePostEnvelope::count_entries(&tables)?;
        assert_eq!(count, 3);
    }

    // Clean up
    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(())
}

#[test]
fn test_content_addressed_subscription() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join(format!(
        "netabase_ca_sub_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_dir)?;

    let stores = MainRepositoryStores::new(&temp_dir)?;

    let post = ImmutablePost {
        author: "Alice".to_string(),
        content: "Topic 1 Post".to_string(),
        timestamp: 1000,
    };

    // Insert
    {
        let txn = stores.definition.begin_write()?;
        let env = ImmutablePostEnvelope::from(&post);
        txn.create(&env)?; // Auto-subscribes to Topic1, Topic2
        txn.commit()?;
    }

    // Query by Subscription
    {
        let txn = stores.definition.begin_read()?;

        // Query Topic1
        let topic1_results = txn
            .query_by_subscription::<ImmutablePostEnvelope, _>(&DefinitionSubscriptions::Topic1)?;
        assert_eq!(topic1_results.len(), 1);
        let model_hash = &topic1_results[0];

        // For content-addressed with u64 key, the ModelHash in subscription table
        // contains the bytes of the u64 PK.
        use netabase_store_examples::boilerplate_lib::models::hash_model;
        let expected_u64_hash = hash_model(&post);

        // Wrap in ID type and serialize to match how it's stored in ModelHash
        use netabase_store_examples::boilerplate_lib::definition::ImmutablePostID;
        let id = ImmutablePostID(expected_u64_hash);
        let expected_bytes = netabase_store::postcard::to_allocvec(&id).unwrap();

        // stored_bytes in ModelHash are padded with zeros to 32 bytes
        let stored_bytes = model_hash.as_bytes();
        assert_eq!(
            &stored_bytes[0..expected_bytes.len()],
            expected_bytes.as_slice()
        );

        // Verify remaining bytes are zero
        for b in &stored_bytes[expected_bytes.len()..] {
            assert_eq!(*b, 0);
        }

        // Can read the actual record using the hash
        let retrieved = txn.read::<ImmutablePostEnvelope>(&id)?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().inner.author, "Alice");

        // Query Topic2 (also subscribed)
        let topic2_results = txn
            .query_by_subscription::<ImmutablePostEnvelope, _>(&DefinitionSubscriptions::Topic2)?;
        assert_eq!(topic2_results.len(), 1);

        // Query Topic3 (not subscribed)
        let topic3_results = txn
            .query_by_subscription::<ImmutablePostEnvelope, _>(&DefinitionSubscriptions::Topic3)?;
        assert!(topic3_results.is_empty());
    }

    // Clean up
    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(())
}
