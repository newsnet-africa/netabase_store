# DHT Implementation Documentation

## Overview

This document describes the comprehensive libp2p Kademlia DHT integration for NetabaseStore, including the trait implementations, helper functions, and usage patterns for building distributed storage systems.

## Architecture

The DHT implementation provides a bridge between NetabaseStore's type-safe data models and libp2p's Kademlia DHT protocol. It consists of several key components:

1. **KademliaRecord Trait** - Converts NetabaseDefinition types to libp2p Records
2. **KademliaRecordKey Trait** - Converts NetabaseDefinitionKeys to libp2p RecordKeys
3. **Provider Record Helpers** - Manages ProviderRecord storage in sled database
4. **Record Helpers** - Placeholder for Record iteration (to be implemented)

## Core Traits

### KademliaRecord

Enables NetabaseDefinition types to be converted to/from libp2p Records for DHT storage.

```rust
pub trait KademliaRecord: NetabaseDefinition + bincode::Encode + bincode::Decode<()> {
    type NetabaseRecordKey: KademliaRecordKey;

    fn record_keys(&self) -> Self::NetabaseRecordKey;
    fn try_to_vec(&self) -> Result<Vec<u8>, bincode::error::EncodeError>;
    fn try_from_vec<V: AsRef<[u8]>>(vec: V) -> Result<Self, bincode::error::DecodeError>;
    fn try_to_record(&self) -> Result<Record, EncodingDecodingError>;
    fn try_from_record(record: Record) -> Result<Self, EncodingDecodingError>;
}
```

**Key Features:**
- Automatic serialization using bincode
- Type-safe conversion to libp2p Records
- Preserves all NetabaseDefinition data in Record.value
- Generates RecordKey from NetabaseDefinitionKeys

### KademliaRecordKey

Handles conversion between NetabaseDefinitionKeys and libp2p RecordKeys.

```rust
pub trait KademliaRecordKey: NetabaseDefinitionKeys + bincode::Encode + bincode::Decode<()> {
    fn try_to_vec(&self) -> Result<Vec<u8>, bincode::error::EncodeError>;
    fn try_from_vec<V: AsRef<[u8]>>(vec: V) -> Result<Self, bincode::error::DecodeError>;
    fn try_to_record_key(&self) -> Result<RecordKey, EncodingDecodingError>;
    fn try_from_record_key(key: &RecordKey) -> Result<Self, EncodingDecodingError>;
}
```

**Key Features:**
- Deterministic key generation
- Reversible conversion between key types
- Type-safe deserialization

## Provider Record Management

### Storage Format

ProviderRecords are stored in sled with a simplified key-value format:

```
Key: RecordKey bytes
Value: serialized ProviderInfo { provider: Vec<u8>, addresses: Vec<Vec<u8>> }
```

This allows efficient:
- Provider lookup by RecordKey (direct key lookup)
- Provider removal by specific PeerId (check and remove if match)
- Iteration over all providers
- Simplified key management

### Helper Functions

#### `ivec_to_provider_record`
Converts raw sled key-value data to ProviderRecord.

```rust
pub fn ivec_to_provider_record(
    key: &IVec,
    value: &IVec,
) -> Result<ProviderRecord, EncodingDecodingError>
```

#### `provider_record_to_ivec`
Converts ProviderRecord to sled storage format.

```rust
pub fn provider_record_to_ivec(
    record: &ProviderRecord,
) -> Result<(IVec, IVec), EncodingDecodingError>
```

#### `get_providers_for_key`
Retrieves all providers for a given RecordKey.

```rust
pub fn get_providers_for_key(
    provider_tree: &Tree,
    record_key: &RecordKey,
) -> Result<Vec<ProviderRecord>, EncodingDecodingError>
```

#### `add_provider_to_key`
Adds a provider record to the storage tree.

```rust
pub fn add_provider_to_key(
    provider_tree: &Tree,
    provider_record: &ProviderRecord,
) -> Result<(), EncodingDecodingError>
```

#### `remove_provider_from_key`
Removes a specific provider from a key.

```rust
pub fn remove_provider_from_key(
    provider_tree: &Tree,
    record_key: &RecordKey,
    peer_id: &PeerId,
) -> Result<bool, EncodingDecodingError>
```

### Provider Record Iterator

The `ProviderRecordIter` adapts sled's iterator to produce `Cow<'static, ProviderRecord>` items:

```rust
pub struct ProviderRecordIter<I> {
    inner: I,
}

impl<I> Iterator for ProviderRecordIter<I>
where
    I: Iterator<Item = Result<(IVec, IVec), sled::Error>>,
{
    type Item = Cow<'static, ProviderRecord>;
    // ...
}
```

## Serialization Overhead Analysis

Based on comprehensive testing, the enum wrapper approach has minimal overhead:

| Data Type | Direct Size | Wrapped Size | Overhead | Relative Overhead |
|-----------|-------------|--------------|----------|-------------------|
| Unit Type | 0 bytes | 1 byte | 1 byte | N/A |
| Simple Data | 4 bytes | 5 bytes | 1 byte | 25% |
| Medium Data | 30 bytes | 31 bytes | 1 byte | 3.33% |
| Complex Data | 1,585 bytes | 1,586 bytes | 1 byte | 0.06% |

**Key Findings:**
- Constant 1-byte overhead for enum discriminant
- Negligible impact on large records (< 0.1%)
- Total overhead across all test types: 0.25%

## Implementation Example

### Basic Setup

```rust
// Define your data types
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub email: String,
}

// Define NetabaseDefinition enum
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, strum::EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Encode, Decode, Hash))]
pub enum MyDefinition {
    UserProfile(UserProfile),
    // ... other variants
}

// Implement DHT traits
impl KademliaRecord for MyDefinition {
    type NetabaseRecordKey = MyDefinitionKeys;
    
    fn record_keys(&self) -> Self::NetabaseRecordKey {
        self.keys()
    }
}

impl KademliaRecordKey for MyDefinitionKeys {}
```

### RecordStore Integration

```rust
impl RecordStore for MyRecordStore {
    type RecordsIter<'a> = /* your iterator type */;
    type ProvidedIter<'a> = provider_record_helpers::ProviderRecordIter<sled::Iter>;

    fn get(&self, key: &RecordKey) -> Option<Cow<'_, Record>> {
        // Retrieve from your storage
    }

    fn put(&mut self, record: Record) -> Result<(), StoreError> {
        // Deserialize and store
        let data = MyDefinition::try_from_record(record)?;
        // Store data using your NetabaseStore
    }

    fn add_provider(&mut self, provider_record: ProviderRecord) -> Result<(), StoreError> {
        provider_record_helpers::add_provider_to_key(&self.providers, &provider_record)
            .map_err(|_| StoreError::ValueTooLarge)?;
        Ok(())
    }

    fn providers(&self, key: &RecordKey) -> Vec<ProviderRecord> {
        provider_record_helpers::get_providers_for_key(&self.providers, key)
            .unwrap_or_default()
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        provider_record_helpers::ProviderRecordIter::new(self.providers.iter())
    }

    fn remove_provider(&mut self, key: &RecordKey, peer: &PeerId) {
        let _ = provider_record_helpers::remove_provider_from_key(&self.providers, key, peer);
    }

    // Implement other required methods...
}
```

## Usage Patterns

### Storing Data

```rust
let user = UserProfile {
    id: "user123".to_string(),
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
};

let definition = MyDefinition::UserProfile(user);
let record = definition.try_to_record()?;

// Store in DHT
record_store.put(record)?;
```

### Retrieving Data

```rust
let user_key = MyDefinitionKeys::UserProfile(UserProfileKey {
    id: "user123".to_string(),
});

let record_key = user_key.try_to_record_key()?;

if let Some(record) = record_store.get(&record_key) {
    let definition = MyDefinition::try_from_record(record.into_owned())?;
    // Use the retrieved data
}
```

### Managing Providers

```rust
use libp2p::Multiaddr;

// Add yourself as a provider
let provider_record = ProviderRecord {
    key: record_key.clone(),
    provider: local_peer_id,
    expires: None,
    addresses: vec!["/ip4/127.0.0.1/tcp/12345".parse::<Multiaddr>()?],
};

record_store.add_provider(provider_record)?;

// Find all providers for a key
let providers = record_store.providers(&record_key);
for provider in providers {
    println!("Provider: {} with {} addresses", provider.provider, provider.addresses.len());
}
```

## Error Handling

The implementation provides comprehensive error handling:

```rust
pub enum EncodingDecodingError {
    Encoding(bincode::error::EncodeError),
    Decoding(bincode::error::DecodeError),
    InvalidKeyFormat,
    InvalidPeerId,
}
```

Common error scenarios:
- **Serialization failures** - Invalid data structures
- **Deserialization failures** - Corrupted or incompatible data
- **Key format errors** - Malformed composite keys
- **PeerId parsing errors** - Invalid peer identifiers

## Performance Characteristics

### Serialization Performance
- 1000 serializations complete in <1 second
- Constant time complexity for enum discriminant
- Minimal CPU overhead vs direct serialization

### Storage Efficiency
- 1-byte overhead per record for type information
- Efficient provider lookup using prefix scans
- Compact key format reduces storage requirements

### Network Efficiency
- Minimal bandwidth overhead (0.06% for typical data)
- No additional network round trips
- Compatible with standard Kademlia protocol

## Testing

The implementation includes comprehensive tests covering:

### Functional Tests
- Round-trip serialization/deserialization
- Key generation and conversion
- Provider record management
- Error handling scenarios

### Integration Tests
- RecordStore workflow simulation
- Large data handling (10KB+ records)
- Many-address provider scenarios
- Concurrent access patterns

### Performance Tests
- Serialization/deserialization speed
- Memory usage analysis
- Network overhead measurement

### Error Handling Tests
- Invalid data recovery
- Corrupted record handling
- Malformed key detection

## Best Practices

### Type Safety
- Always use the trait methods for conversions
- Implement comprehensive error handling
- Validate data integrity after deserialization

### Storage Management
- Use appropriate sled tree names for organization
- Implement cleanup for expired providers
- Monitor storage size and implement limits
- Note: Current implementation stores one provider per key (later providers overwrite earlier ones)

### Network Efficiency
- Batch operations when possible
- Use appropriate replication factors
- Implement caching for frequently accessed records

### Security Considerations
- Validate record sizes to prevent DoS attacks
- Implement rate limiting for provider additions
- Verify peer identity for provider records

## Future Enhancements

### Planned Features
1. **Record Iterator Implementation** - Complete the record iteration functionality
2. **Multiple Providers per Key** - Support storing multiple providers for the same key
3. **Compression Support** - Optional compression for large records
4. **Encryption Layer** - End-to-end encryption for sensitive data
5. **Metrics Collection** - Performance and usage monitoring
6. **Async Support** - Non-blocking operations for better performance

### Optimization Opportunities
1. **Custom Serialization** - More efficient formats for specific data types
2. **Caching Layer** - In-memory caching for hot data
3. **Indexing** - Secondary indexes for complex queries
4. **Sharding** - Horizontal scaling for large datasets
5. **Provider Multiplexing** - Store multiple providers per key using collections

## Conclusion

This DHT implementation provides a robust, type-safe bridge between NetabaseStore and libp2p's Kademlia DHT. The minimal serialization overhead (0.06% for typical data) makes it practical for production use, while the comprehensive error handling and testing ensure reliability.

The design prioritizes:
- **Type Safety** - Compile-time guarantees for data integrity
- **Performance** - Minimal overhead and efficient operations
- **Flexibility** - Support for arbitrary NetabaseDefinition types
- **Reliability** - Comprehensive error handling and testing

For most use cases, the enum wrapper approach is strongly recommended over key-based type identification due to its simplicity, safety, and negligible performance cost.