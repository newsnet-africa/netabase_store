//! Example: Unified Configuration System
//!
//! This example demonstrates the new unified `QueryConfig` system that consolidates
//! all configuration options for database operations.

use netabase_store::config::{QueryConfig, ConfigDefaults, DefaultsBuilder};
use netabase_store::databases::redb::RedbStore;
use netabase_store::doc_example::*;
use netabase_store::traits::database::store::NBStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Unified Configuration System Demo ===\n");

    // Create a temporary database
    let (store, _temp) = RedbStore::<ExampleDef>::new_temporary()?;

    // === 1. Basic QueryConfig Usage ===
    println!("1. Basic QueryConfig:");
    let config = QueryConfig::new()
        .with_limit(10)
        .with_offset(5)
        .no_blobs()
        .with_hydration(1);
    
    println!("   - Limit: {:?}", config.pagination.limit);
    println!("   - Offset: {:?}", config.pagination.offset);
    println!("   - Strip blobs: {}", config.blob.strip_blobs);
    println!("   - Hydration depth: {}\n", config.hydration.depth);

    // === 2. Factory Methods ===
    println!("2. Factory Methods:");
    
    let all = QueryConfig::all();
    println!("   - all(): unlimited query");
    
    let first = QueryConfig::first();
    println!("   - first(): limit = {:?}", first.pagination.limit);
    
    let dump = QueryConfig::dump_all();
    println!("   - dump_all(): with blobs, no hydration\n");

    // === 3. Configuration Defaults ===
    println!("3. Configuration Defaults:");
    
    let mut defaults = ConfigDefaults::new();
    
    // Set store-wide defaults
    defaults.set_store_default(
        QueryConfig::new()
            .with_limit(100)
            .no_blobs()
    );
    println!("   - Store default: limit=100, no_blobs=true");
    
    // Set table-specific defaults
    defaults.set_table_default(
        "User",
        QueryConfig::new()
            .with_limit(50)
            .with_hydration(1)
    );
    println!("   - User table: limit=50, hydration=1");
    
    // Get effective defaults
    let user_defaults = defaults.get_for_table("User");
    let post_defaults = defaults.get_for_table("Post");
    
    println!("   - User effective: limit={:?}, hydration={}", 
        user_defaults.pagination.limit,
        user_defaults.hydration.depth
    );
    println!("   - Post effective: limit={:?}, hydration={}\n", 
        post_defaults.pagination.limit,
        post_defaults.hydration.depth
    );

    // === 4. Merging Configurations ===
    println!("4. Merging Configurations:");
    
    // Query-specific overrides
    let query_config = QueryConfig::new().with_offset(20);
    let final_config = defaults.apply_to("User", query_config);
    
    println!("   - Query sets: offset=20");
    println!("   - After merge: limit={:?}, offset={:?}, hydration={}",
        final_config.pagination.limit,
        final_config.pagination.offset,
        final_config.hydration.depth
    );
    println!("   - Limit and hydration inherited from defaults!\n");

    // === 5. Preset Configurations ===
    println!("5. Preset Configurations:");
    
    let perf = DefaultsBuilder::performance_optimized();
    println!("   - Performance: strip_blobs={}, hydration={}, limit={:?}",
        perf.store_default().blob.strip_blobs,
        perf.store_default().hydration.depth,
        perf.store_default().pagination.limit
    );
    
    let rich = DefaultsBuilder::rich_data();
    println!("   - Rich data: strip_blobs={}, hydration={}, limit={:?}",
        rich.store_default().blob.strip_blobs,
        rich.store_default().hydration.depth,
        rich.store_default().pagination.limit
    );
    
    let api = DefaultsBuilder::api_optimized();
    println!("   - API: strip_blobs={}, hydration={}, limit={:?}\n",
        api.store_default().blob.strip_blobs,
        api.store_default().hydration.depth,
        api.store_default().pagination.limit
    );

    // === 6. Configuration Hierarchy Demo ===
    println!("6. Configuration Hierarchy (store → table → query):");
    
    let mut hierarchy = ConfigDefaults::new();
    
    // Store level: all tables get limit=1000
    hierarchy.set_store_default(QueryConfig::new().with_limit(1000));
    
    // Table level: Users get stricter limit
    hierarchy.set_table_default("User", QueryConfig::new().with_limit(50));
    
    // Query level: specific query needs even more
    let query = QueryConfig::new().with_limit(25).with_offset(10);
    let result = hierarchy.apply_to("User", query);
    
    println!("   - Store: limit=1000");
    println!("   - Table: limit=50");
    println!("   - Query: limit=25, offset=10");
    println!("   - Final: limit={:?}, offset={:?}",
        result.pagination.limit,
        result.pagination.offset
    );
    println!("   → Query takes highest priority!\n");

    // === 7. Query Modes ===
    println!("7. Query Modes:");
    
    let fetch_config = QueryConfig::new();
    println!("   - Default mode: {:?}", fetch_config.mode);
    
    let count_config = QueryConfig::new().count_only();
    println!("   - Count mode: {:?}", count_config.mode);
    
    let reversed = QueryConfig::new().reversed();
    println!("   - Reversed: {}\n", reversed.reversed);

    println!("=== Summary ===");
    println!("✓ Single unified QueryConfig type");
    println!("✓ Replaces CrudOptions and old QueryConfig");
    println!("✓ Three-level hierarchy: store → table → query");
    println!("✓ Builder pattern for easy construction");
    println!("✓ Preset configurations for common use cases");
    println!("✓ Consistent API across all operations");

    Ok(())
}
