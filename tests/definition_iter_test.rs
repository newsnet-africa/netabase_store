use netabase_store::prelude::*;
use serde::{Deserialize, Serialize};

#[netabase_macros::netabase_definition(IterTestDef)]
pub mod iter_test_def {
    use super::*;

    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord
    )]
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
fn test_definition_iterator() -> NetabaseResult<()> {
    use iter_test_def::*;
    
    // Setup DB
    let (store, _path) = RedbStore::<IterTestDef>::new_temporary()?;
    
    // Insert data
    let txn = store.begin_write()?;
    txn.create(&ModelA { id: ModelAID("a1".to_string()), val: 10 })?;
    txn.create(&ModelA { id: ModelAID("a2".to_string()), val: 20 })?;
    txn.create(&ModelB { id: ModelBID(1), txt: "b1".to_string() })?;
    txn.commit()?;
    
    // Test Iterator
    let txn = store.begin_read()?;
    
    // Use with_read_transaction to access the raw redb::ReadTransaction
    txn.with_read_transaction(|rt| {
        // Generated structs should be available in the module
        let tables = IterTestDefReadOnlyTables::new(rt)
            .map_err(netabase_store::errors::NetabaseError::RedbError)?;
            
        let iter = tables.iter().map_err(netabase_store::errors::NetabaseError::RedbError)?;
        
        let items: Vec<_> = iter.collect();
        
        assert_eq!(items.len(), 3);
        
        // Verify content
        let count_a = items.iter().filter(|r| matches!(r, Ok(IterTestDef::ModelA(_)))).count();
        let count_b = items.iter().filter(|r| matches!(r, Ok(IterTestDef::ModelB(_)))).count();
        
        assert_eq!(count_a, 2);
        assert_eq!(count_b, 1);
        
        Ok(())
    })?;
    
    Ok(())
}
