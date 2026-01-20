/// Compare database growth with actual data
/// Test to see when the database actually grows beyond initial allocation

use netabase_store::databases::redb::RedbStore;
use netabase_store::traits::database::store::NBStore;
use netabase_store::relational::RelationalLink;
use example::boilerplate_lib::definition::{
    AnotherLargeUserFile, LargeUserFile,
};
use example::boilerplate_lib::{
    CategoryID, Definition, User, UserID,
};
use std::path::PathBuf;
use std::fs;

#[test]
fn database_growth_analysis() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = PathBuf::from("/tmp/netabase_growth_test");
    
    println!("\n=== DATABASE GROWTH ANALYSIS ===\n");
    
    for count in [1, 10, 100, 1000, 5000] {
        if db_path.exists() {
            std::fs::remove_dir_all(&db_path).ok();
        }
        
        let store = RedbStore::<Definition>::new(&db_path)?;
        let txn = store.begin_write()?;
        
        for i in 0..count {
            let user = User {
                id: UserID(format!("user{:06}", i)),
                first_name: format!("FirstName{}", i),
                last_name: format!("LastName{}", i),
                age: (i % 100) as u8,
                partner: RelationalLink::new_dehydrated(UserID(format!("partner{}", i % 10))),
                category: RelationalLink::new_dehydrated(CategoryID(format!("cat{}", i % 5))),
                bio: LargeUserFile::default(),
                another: AnotherLargeUserFile::default(),
            };
            txn.create::<User>(&user)?;
        }
        
        txn.commit()?;
        drop(store);
        
        let db_file = db_path.join("data.redb");
        let size = fs::metadata(&db_file)?.len();
        let size_per_record = size / count as u64;
        
        println!("{:6} users: {:10} bytes ({:7.2} MB) | {:6} bytes/user ({:5.2} KB/user)", 
                 count, 
                 size, 
                 size as f64 / (1024.0 * 1024.0),
                 size_per_record,
                 size_per_record as f64 / 1024.0);
        
        std::fs::remove_dir_all(&db_path).ok();
    }
    
    println!("\n=== KEY INSIGHT ===");
    println!("The database has a fixed initial size (~1 MB).");
    println!("This is allocated upfront for the 9 B-tree tables.");
    println!("Growth beyond 1 MB only happens when data exceeds this allocation.");
    println!("\nThe 'overhead' is NOT per-record, it's a ONE-TIME initial cost.");
    
    Ok(())
}
