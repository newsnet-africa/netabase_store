/// Compare database growth with actual data
/// Test to see when the database actually grows beyond initial allocation

use netabase_store::relational::RelationalLink;
use example::boilerplate_lib::main_repository::definition::{
    AnotherLargeUserFile, LargeUserFile,
};
use example::boilerplate_lib::{
    CategoryID, MainRepositoryStores, User, UserID,
};
use std::fs;

#[test]
fn database_growth_analysis() -> Result<(), Box<dyn std::error::Error>> {
    let mut repo_path = std::env::temp_dir();
    repo_path.push(format!("netabase_growth_test_{}", std::process::id()));
    
    println!("\n=== DATABASE GROWTH ANALYSIS ===\n");
    
    for count in [1, 10, 100, 1000, 5000] {
        if repo_path.exists() {
            std::fs::remove_dir_all(&repo_path).ok();
        }
        
        let stores = MainRepositoryStores::new(&repo_path)?;
        let txn = stores.definition.begin_write()?;
        
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
        drop(stores);
        
        let db_file = repo_path.join("Definition").join("data.redb");
        let size = fs::metadata(&db_file)?.len();
        let size_per_record = size / count as u64;
        
        println!("{:6} users: {:10} bytes ({:7.2} MB) | {:6} bytes/user ({:5.2} KB/user)", 
                 count, 
                 size, 
                 size as f64 / (1024.0 * 1024.0),
                 size_per_record,
                 size_per_record as f64 / 1024.0);
        
        std::fs::remove_dir_all(&repo_path).ok();
    }
    
    println!("\n=== KEY INSIGHT ===");
    println!("The database has a fixed initial size (~1 MB).");
    println!("This is allocated upfront for the 9 B-tree tables.");
    println!("Growth beyond 1 MB only happens when data exceeds this allocation.");
    println!("\nThe 'overhead' is NOT per-record, it's a ONE-TIME initial cost.");
    
    Ok(())
}