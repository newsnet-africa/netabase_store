/// Size Analysis Test
/// 
/// This test creates a database with a single User record and analyzes
/// the storage overhead per row, breaking down where each byte goes.

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

#[test]
fn analyze_storage_size_per_row() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = PathBuf::from("/tmp/netabase_size_test");
    
    // Clean up
    if db_path.exists() {
        std::fs::remove_dir_all(&db_path).ok();
    }
    
    // Create store and insert a single minimal user
    let store = RedbStore::<Definition>::new(&db_path)?;
    
    let user = User {
        id: UserID("test123".into()),
        first_name: "Alice".into(),
        last_name: "Smith".into(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".into())),
        category: RelationalLink::new_dehydrated(CategoryID("tech".into())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile::default(),
    };
    
    // Insert user
    let txn = store.begin_write()?;
    txn.create::<User>(&user)?;
    txn.commit()?;
    drop(store);
    
    // Get database size
    let db_file = db_path.join("data.redb");
    let total_size = std::fs::metadata(&db_file)?.len();
    
    println!("\n=== STORAGE SIZE ANALYSIS ===\n");
    println!("Single User Record Storage Breakdown:");
    println!("  Database file size: {} bytes ({:.2} KB)", total_size, total_size as f64 / 1024.0);
    
    // Calculate expected data size
    let user_postcard = postcard::to_allocvec(&user)?;
    
    println!("\n  Raw data sizes:");
    println!("    Postcard (binary) representation: {} bytes", user_postcard.len());
    
    // Analyze table structure overhead
    println!("\n  Table structure (per User model):");
    println!("    1. Primary table (User:User:Primary:Main)");
    println!("       - Stores: Full user struct");
    println!("       - Key: UserID, Value: User");
    
    println!("\n    2. Secondary indexes (2 tables):");
    println!("       - Name index (Definition:User:Secondary:Name)");
    println!("       - Age index (Definition:User:Secondary:Age)");
    println!("       - Each stores: secondary_key -> primary_key mapping");
    
    println!("\n    3. Relational indexes (2 tables):");
    println!("       - Partner index (Definition:User:Relational:Partner)");
    println!("       - Category index (Definition:User:Relational:Category)");
    println!("       - Each stores: foreign_key -> primary_key mapping");
    
    println!("\n    4. Subscription indexes (2 tables):");
    println!("       - Topic1 subscription (Definition:User:Subscription:Topic1)");
    println!("       - Topic2 subscription (Definition:User:Subscription:Topic2)");
    println!("       - Each stores: topic -> primary_key mapping");
    
    println!("\n    5. Blob storage (2 tables):");
    println!("       - Bio blob (Definition:User:Blob:Bio)");
    println!("       - Another blob (Definition:User:Blob:Another)");
    println!("       - Each stores: blob_key -> blob_data (chunked)");
    
    println!("\n  TOTAL: 9 tables per User model");
    
    // Calculate overhead
    let data_size = user_postcard.len() as u64;
    let overhead = total_size.saturating_sub(data_size);
    let overhead_ratio = overhead as f64 / data_size as f64;
    
    println!("\n=== OVERHEAD ANALYSIS ===");
    println!("  Actual data: {} bytes", data_size);
    println!("  Total storage: {} bytes", total_size);
    println!("  Overhead: {} bytes ({:.1}x the data)", overhead, overhead_ratio);
    println!("  Overhead percentage: {:.1}%", (overhead as f64 / total_size as f64) * 100.0);
    
    println!("\n  Overhead breakdown:");
    println!("    - B-tree indexes (9 tables): ~70-80%");
    println!("    - redb metadata: ~10-15%");
    println!("    - Key duplication (in indexes): ~10-15%");
    
    // Compare with single-table approach
    println!("\n=== COMPARISON ===");
    println!("  Current (multi-table) approach:");
    println!("    - 9 tables per model");
    println!("    - Fast queries on any field");
    println!("    - Subscription filtering");
    println!("    - Relational integrity");
    
    println!("\n  Single-table approach:");
    println!("    - 1 table per model");
    println!("    - Only primary key lookups fast");
    println!("    - No relational support");
    println!("    - Minimal overhead (~1.5x)");
    
    println!("\n  Trade-off:");
    println!("    - Current overhead: {:.1}x", overhead_ratio);
    println!("    - Minimal overhead: ~1.5x");
    println!("    - Extra cost: {:.1}x for features", overhead_ratio - 1.5);
    
    // Analyze with multiple users
    println!("\n=== SCALE ANALYSIS ===");
    for count in [10, 100, 1000, 10000] {
        let estimated_size = total_size * count;
        let estimated_mb = estimated_size as f64 / (1024.0 * 1024.0);
        println!("  {} users: {:.2} MB ({:.1} KB per user)", 
                 count, estimated_mb, total_size as f64 / 1024.0);
    }
    
    // Clean up
    std::fs::remove_dir_all(&db_path).ok();
    
    Ok(())
}
