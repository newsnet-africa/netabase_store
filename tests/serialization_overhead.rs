use bincode::{Decode, Encode};
use std::mem;

// Simulate a complex struct that might be in your NetabaseModel
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct ComplexData {
    pub id: String,
    pub content: Vec<u8>,
    pub metadata: std::collections::HashMap<String, String>,
    pub timestamps: Vec<u64>,
    pub large_field: [u8; 1024], // 1KB of data
}

// Simulate unit variants (no data)
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct UnitTypeA;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct UnitTypeB;

// Medium complexity struct
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct MediumData {
    pub name: String,
    pub value: i64,
    pub flags: Vec<bool>,
}

// Simple struct
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct SimpleData {
    pub id: u64,
    pub active: bool,
}

// The enum wrapper (similar to your NetabaseDefinition)
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum DataWrapper {
    UnitA(UnitTypeA),
    UnitB(UnitTypeB),
    Simple(SimpleData),
    Medium(MediumData),
    Complex(ComplexData),
    // Add more variants to simulate 12 total
    Variant6(String),
    Variant7(Vec<u8>),
    Variant8(i32),
    Variant9(i64),
    Variant10(bool),
    Variant11(Option<String>),
    Variant12(std::collections::BTreeMap<String, i32>),
}

impl ComplexData {
    fn sample() -> Self {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());

        Self {
            id: "complex_id_12345".to_string(),
            content: vec![0u8; 500], // 500 bytes
            metadata,
            timestamps: vec![1640995200, 1640995260, 1640995320],
            large_field: [42u8; 1024],
        }
    }
}

impl MediumData {
    fn sample() -> Self {
        Self {
            name: "medium_data_sample".to_string(),
            value: 42_000_000,
            flags: vec![true, false, true, false, true],
        }
    }
}

impl SimpleData {
    fn sample() -> Self {
        Self {
            id: 12345,
            active: true,
        }
    }
}

fn serialize_data<T: Encode>(data: &T) -> Vec<u8> {
    bincode::encode_to_vec(data, bincode::config::standard()).unwrap()
}

fn calculate_overhead_percentage(wrapped_size: usize, direct_size: usize) -> f64 {
    if direct_size == 0 {
        return 0.0;
    }
    ((wrapped_size as f64 - direct_size as f64) / direct_size as f64) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization_overhead() {
        println!("\n=== Rust Enum Serialization Overhead Analysis ===\n");

        // Test Unit Types
        let unit_a = UnitTypeA;
        let wrapped_unit_a = DataWrapper::UnitA(unit_a.clone());

        let unit_a_direct = serialize_data(&unit_a);
        let unit_a_wrapped = serialize_data(&wrapped_unit_a);

        println!("Unit Type A:");
        println!("  Direct serialization: {} bytes", unit_a_direct.len());
        println!("  Wrapped in enum: {} bytes", unit_a_wrapped.len());
        println!(
            "  Overhead: {} bytes ({:.2}%)",
            unit_a_wrapped.len() - unit_a_direct.len(),
            calculate_overhead_percentage(unit_a_wrapped.len(), unit_a_direct.len())
        );

        // Test Simple Data
        let simple = SimpleData::sample();
        let wrapped_simple = DataWrapper::Simple(simple.clone());

        let simple_direct = serialize_data(&simple);
        let simple_wrapped = serialize_data(&wrapped_simple);

        println!("\nSimple Data (u64 + bool):");
        println!("  Direct serialization: {} bytes", simple_direct.len());
        println!("  Wrapped in enum: {} bytes", simple_wrapped.len());
        println!(
            "  Overhead: {} bytes ({:.2}%)",
            simple_wrapped.len() - simple_direct.len(),
            calculate_overhead_percentage(simple_wrapped.len(), simple_direct.len())
        );

        // Test Medium Data
        let medium = MediumData::sample();
        let wrapped_medium = DataWrapper::Medium(medium.clone());

        let medium_direct = serialize_data(&medium);
        let medium_wrapped = serialize_data(&wrapped_medium);

        println!("\nMedium Data (String + i64 + Vec<bool>):");
        println!("  Direct serialization: {} bytes", medium_direct.len());
        println!("  Wrapped in enum: {} bytes", medium_wrapped.len());
        println!(
            "  Overhead: {} bytes ({:.2}%)",
            medium_wrapped.len() - medium_direct.len(),
            calculate_overhead_percentage(medium_wrapped.len(), medium_direct.len())
        );

        // Test Complex Data
        let complex = ComplexData::sample();
        let wrapped_complex = DataWrapper::Complex(complex.clone());

        let complex_direct = serialize_data(&complex);
        let complex_wrapped = serialize_data(&wrapped_complex);

        println!("\nComplex Data (~1.5KB struct):");
        println!("  Direct serialization: {} bytes", complex_direct.len());
        println!("  Wrapped in enum: {} bytes", complex_wrapped.len());
        println!(
            "  Overhead: {} bytes ({:.2}%)",
            complex_wrapped.len() - complex_direct.len(),
            calculate_overhead_percentage(complex_wrapped.len(), complex_direct.len())
        );

        // Summary
        println!("\n=== Summary ===");
        println!("The enum discriminant adds a small constant overhead (typically 1-8 bytes)");
        println!("depending on the number of variants (12 variants ≈ 1 byte discriminant)");
        println!("\nKey insights:");
        println!("- Unit types: High relative overhead (100%+ typically)");
        println!("- Small structs: Moderate relative overhead (10-50%)");
        println!("- Large structs: Minimal relative overhead (<5%)");

        // Test memory representation
        println!("\n=== Memory Size Analysis ===");
        println!("Unit type in memory: {} bytes", mem::size_of::<UnitTypeA>());
        println!(
            "Simple data in memory: {} bytes",
            mem::size_of::<SimpleData>()
        );
        println!(
            "Medium data in memory: {} bytes",
            mem::size_of::<MediumData>()
        );
        println!(
            "Complex data in memory: {} bytes",
            mem::size_of::<ComplexData>()
        );
        println!(
            "Enum wrapper in memory: {} bytes",
            mem::size_of::<DataWrapper>()
        );

        // Network efficiency recommendations
        println!("\n=== Network Transfer Recommendations ===");

        let total_overhead = (unit_a_wrapped.len() - unit_a_direct.len())
            + (simple_wrapped.len() - simple_direct.len())
            + (medium_wrapped.len() - medium_direct.len())
            + (complex_wrapped.len() - complex_direct.len());

        let total_direct =
            unit_a_direct.len() + simple_direct.len() + medium_direct.len() + complex_direct.len();

        let total_wrapped = unit_a_wrapped.len()
            + simple_wrapped.len()
            + medium_wrapped.len()
            + complex_wrapped.len();

        println!("If sending all 4 test structures:");
        println!("  Direct: {} bytes total", total_direct);
        println!("  Wrapped: {} bytes total", total_wrapped);
        println!(
            "  Total overhead: {} bytes ({:.2}%)",
            total_overhead,
            calculate_overhead_percentage(total_wrapped, total_direct)
        );

        println!("\nRecommendation: For your use case with libp2p Kademlia DHT:");
        if calculate_overhead_percentage(total_wrapped, total_direct) < 10.0 {
            println!("✓ The enum wrapper approach is RECOMMENDED");
            println!("  - Overhead is minimal (<10%)");
            println!("  - Significantly simpler deserialization logic");
            println!("  - Type safety benefits outweigh small size cost");
        } else {
            println!("⚠ Consider the key-based approach");
            println!("  - Enum overhead is significant (>10%)");
            println!("  - May be worth the additional complexity");
        }
    }

    #[test]
    fn test_libp2p_record_compatibility() {
        println!("\n=== libp2p Kademlia Record Analysis ===");

        // Simulate what would go into libp2p::kad::Record
        let complex_data = ComplexData::sample();
        let wrapped_data = DataWrapper::Complex(complex_data.clone());

        // Direct approach: key identifies type, value is raw data
        let direct_key = b"complex_data_type_id";
        let direct_value = serialize_data(&complex_data);

        // Enum approach: key is generic, value includes type info
        let enum_key = b"netabase_record";
        let enum_value = serialize_data(&wrapped_data);

        println!("Direct approach (key + value):");
        println!("  Key size: {} bytes", direct_key.len());
        println!("  Value size: {} bytes", direct_value.len());
        println!("  Total: {} bytes", direct_key.len() + direct_value.len());

        println!("\nEnum approach (key + value):");
        println!("  Key size: {} bytes", enum_key.len());
        println!("  Value size: {} bytes", enum_value.len());
        println!("  Total: {} bytes", enum_key.len() + enum_value.len());

        let enum_total = enum_key.len() + enum_value.len();
        let direct_total = direct_key.len() + direct_value.len();

        if enum_total >= direct_total {
            let overhead = enum_total - direct_total;
            println!("\nEnum approach overhead: {} bytes", overhead);
        } else {
            let savings = direct_total - enum_total;
            println!("\nEnum approach savings: {} bytes", savings);
        }

        println!("\nWith libp2p::kad::Record structure:");
        println!("- Both approaches use the same Record.key (serialized RecordKey)");
        println!("- Both approaches use Record.value (Vec<u8>)");
        println!("- The enum discriminant is embedded in the value bytes");
        println!("- Record.publisher and Record.expires are optional metadata");

        // Demonstrate deserialization complexity
        println!("\nDeserialization complexity:");
        println!("Direct approach:");
        println!("  1. Extract key, determine type from key");
        println!("  2. Match type to appropriate deserializer");
        println!("  3. Deserialize value with type-specific logic");
        println!("  4. Additional error handling for unknown types");

        println!("\nEnum approach:");
        println!("  1. Deserialize value as DataWrapper enum");
        println!("  2. Pattern match on enum variant");
        println!("  3. Built-in type safety and error handling");
    }
}
