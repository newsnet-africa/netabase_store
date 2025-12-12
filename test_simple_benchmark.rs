use bench_comprehensive_all_features::{BenchUser, BenchmarkDataGenerator};

fn main() {
    println!("Testing benchmark data structures...");
    
    let mut generator = BenchmarkDataGenerator::new();
    let user = generator.next_user();
    println!("Created user: {}", user.username);
    
    let json = serde_json::to_string(&user).unwrap();
    println!("Serialized to JSON: {} bytes", json.len());
    
    println!("Test completed successfully!");
}
