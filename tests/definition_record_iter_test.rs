use netabase_store::traits::database::store::NBStore;
use netabase_store::prelude::*;
use netabase_store::traits::registery::definition::redb_definition::RedbDefinition;
use serde::{Deserialize, Serialize};

#[netabase_macros::netabase_definition(RecordIterTestDef)]
pub mod record_iter_test_def {
    use super::*;

    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    #[netabase_libp2p]
    pub struct ModelA {
        #[primary_key]
        pub id: String,
        pub val: u32,
    }

    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct ModelB {
        #[primary_key]
        pub id: u64,
        pub txt: String,
    }
}

#[test]
fn test_definition_record_iterator() -> NetabaseResult<()> {
    use record_iter_test_def::*;
    use netabase_store::traits::libp2p::libp2p_model::Libp2pMetadata;
    use std::time::SystemTime;
    
    // Setup DB
    let (store, _path) = RedbStore::<RecordIterTestDef>::new_temporary()?;
    
    // Insert data
    let txn = store.begin_write()?;
    // ModelA has libp2p enabled
    txn.create(&ModelA { 
        id: ModelAID("a1".to_string()), 
        val: 10,
        libp2p_metadata: Some(Libp2pMetadata {
            publisher: None,
            expires: None,
            extra: None
        })
    })?;
    // ModelB does NOT have libp2p enabled (defaults will be used)
    txn.create(&ModelB { id: ModelBID(1), txt: "b1".to_string() })?;
    txn.commit()?;
    
    // Test Record Iterator
    let txn = store.begin_read()?;
    
    txn.with_read_transaction(|rt| {
        // Open tables first
        let tables = RecordIterTestDef::open_read_only_tables(rt)?;
            
        // Then iterate
        let iter = RecordIterTestDef::iter_records(&tables)?;
            
        let items: Vec<_> = iter.collect();
        
        assert_eq!(items.len(), 2);
        
        // Verify items are Records
        for item in items {
            let record = item.expect("Record iteration failed");
            assert!(!record.key.as_ref().is_empty());
            assert!(!record.value.is_empty());
        }
        
        Ok(())
    })?;
    
    Ok(())
}
