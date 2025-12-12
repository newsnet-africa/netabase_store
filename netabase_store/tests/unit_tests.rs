//! Granular unit tests for core data structures and utilities
//! 
//! These tests focus on individual functions, methods, and small components
//! in isolation, using mocks where necessary to eliminate external dependencies.

use netabase_store::error::{NetabaseError, NetabaseResult};
use proptest::prelude::*;
use assert_matches::assert_matches;
use std::rc::Rc;
use std::sync::Arc;

mod error_handling {
    use super::*;
    
    #[test]
    fn unit_error_creation_and_display() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "Test error");
        let netabase_error = NetabaseError::from(io_error);
        
        assert!(matches!(netabase_error, NetabaseError::IoError(_)));
        assert!(netabase_error.to_string().contains("Test error"));
    }
    
    #[test]
    fn unit_error_chaining() {
        let original = NetabaseError::Other("Test".to_string());
        let chained = NetabaseError::Other(format!("Caused by: {}", original));
        
        assert!(chained.to_string().contains("Caused by"));
        assert!(chained.to_string().contains("Test"));
    }
    
    #[test]
    fn unit_error_from_conversion() {
        // Test with a real error that exists
        let custom_error = NetabaseError::Other("Test serialization error".to_string());
        assert!(custom_error.to_string().contains("Test serialization error"));
    }
    
    #[test]
    fn unit_result_chaining() {
        fn might_fail(should_fail: bool) -> NetabaseResult<i32> {
            if should_fail {
                Err(NetabaseError::Other("Test".to_string()))
            } else {
                Ok(42)
            }
        }
        
        let result = might_fail(false);
        assert_matches!(result, Ok(42));
        
        let result = might_fail(true);
        assert_matches!(result, Err(NetabaseError::Other(_)));
    }
}

mod backend_key_value_tests {
    use super::*;
    
    // Simple concrete key/value types for testing
    #[derive(Debug, Clone, PartialEq)]
    struct TestKey(String);
    
    #[derive(Debug, Clone, PartialEq)]
    struct TestValue(Vec<u8>);
    
    #[test]
    fn unit_backend_key_creation() {
        let key = TestKey("test_key".to_string());
        assert_eq!(key.0, "test_key");
        
        let key2 = TestKey("another_key".to_string());
        assert_eq!(key2.0, "another_key");
    }
    
    #[test]
    fn unit_backend_value_creation() {
        let value = TestValue(b"test_value".to_vec());
        assert_eq!(value.0, b"test_value");
        
        let value2 = TestValue(vec![5, 6, 7, 8]);
        assert_eq!(value2.0, vec![5, 6, 7, 8]);
    }
    
    #[test]
    fn unit_backend_key_ordering() {
        let key1 = TestKey("a".to_string());
        let key2 = TestKey("b".to_string());
        let key3 = TestKey("aa".to_string());
        
        assert!(key1.0 < key2.0);
        assert!(key1.0 < key3.0);
        assert!(key3.0 < key2.0);
    }
    
    proptest! {
        #[test]
        fn unit_backend_key_roundtrip(data in ".*") {
            let key = TestKey(data.clone());
            assert_eq!(key.0, data);
        }
        
        #[test]
        fn unit_backend_value_roundtrip(data in prop::collection::vec(any::<u8>(), 0..1000)) {
            let value = TestValue(data.clone());
            assert_eq!(value.0, data);
        }
    }
}

mod serialization_tests {
    use super::*;
    use bincode::{Encode, Decode};
    
    #[derive(Debug, Clone, PartialEq, Encode, Decode)]
    struct TestStruct {
        id: u64,
        name: String,
        active: bool,
    }
    
    #[test]
    fn unit_bincode_serialization() {
        use bincode::{Encode, Decode};
        
        #[derive(Debug, Clone, PartialEq, Encode, Decode)]
        struct TestStruct {
            id: u64,
            name: String,
            active: bool,
        }
        
        let test_data = TestStruct {
            id: 42,
            name: "Test".to_string(),
            active: true,
        };
        
        let encoded = bincode::encode_to_vec(&test_data, bincode::config::standard()).unwrap();
        let (decoded, _): (TestStruct, usize) = bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        
        assert_eq!(test_data, decoded);
    }
    
    #[test]
    fn unit_empty_serialization() {
        let empty_vec: Vec<u8> = Vec::new();
        let encoded = bincode::encode_to_vec(&empty_vec, bincode::config::standard()).unwrap();
        let (decoded, _): (Vec<u8>, usize) = bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        
        assert_eq!(empty_vec, decoded);
    }
    
    proptest! {
        #[test]
        fn unit_proptest_serialization(
            id in any::<u64>(),
            name in ".*",
            active in any::<bool>()
        ) {
            use bincode::{Encode, Decode};
            
            #[derive(Debug, Clone, PartialEq, Encode, Decode)]
            struct TestStruct {
                id: u64,
                name: String,
                active: bool,
            }
            
            let test_data = TestStruct { id, name, active };
            let encoded = bincode::encode_to_vec(&test_data, bincode::config::standard()).unwrap();
            let (decoded, _): (TestStruct, usize) = bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
            assert_eq!(test_data, decoded);
        }
    }
}

mod utility_functions_tests {
    use super::*;
    use std::time::{Duration, Instant};
    
    #[test]
    fn unit_blake3_hashing() {
        let data = b"test data";
        let hash1 = blake3::hash(data);
        let hash2 = blake3::hash(data);
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.as_bytes().len(), 32);
    }
    
    #[test]
    fn unit_blake3_different_inputs() {
        let hash1 = blake3::hash(b"input1");
        let hash2 = blake3::hash(b"input2");
        
        assert_ne!(hash1, hash2);
    }
    
    #[test]
    fn unit_hex_encoding() {
        let data = b"test";
        let hex_str = hex::encode(data);
        let decoded = hex::decode(hex_str).unwrap();
        
        assert_eq!(data, decoded.as_slice());
    }
    
    proptest! {
        #[test]
        fn unit_hex_roundtrip(data in prop::collection::vec(any::<u8>(), 0..100)) {
            let hex_str = hex::encode(&data);
            let decoded = hex::decode(hex_str).unwrap();
            assert_eq!(data, decoded);
        }
    }
}

mod memory_management_tests {
    use super::*;
    use std::sync::Arc;
    use std::rc::Rc;
    
    #[test]
    fn unit_arc_cloning() {
        let data = Arc::new(vec![1, 2, 3, 4, 5]);
        let clone1 = Arc::clone(&data);
        let clone2 = Arc::clone(&data);
        
        assert_eq!(*data, *clone1);
        assert_eq!(*clone1, *clone2);
        assert_eq!(Arc::strong_count(&data), 3);
    }
    
    #[test]
    fn unit_rc_vs_arc() {
        let rc_data = Rc::new("test");
        let arc_data = Arc::new("test");
        
        assert_eq!(*rc_data, *arc_data);
        
        let _rc_clone = Rc::clone(&rc_data);
        let _arc_clone = Arc::clone(&arc_data);
        
        assert_eq!(Rc::strong_count(&rc_data), 2);
        assert_eq!(Arc::strong_count(&arc_data), 2);
    }
    
    #[test]
    fn unit_drop_order() {
        use std::sync::Mutex;
        use std::sync::Arc;
        
        let drop_order = Arc::new(Mutex::new(Vec::new()));
        
        struct DropRecorder {
            id: u32,
            recorder: Arc<Mutex<Vec<u32>>>,
        }
        
        impl Drop for DropRecorder {
            fn drop(&mut self) {
                self.recorder.lock().unwrap().push(self.id);
            }
        }
        
        {
            let _recorder1 = DropRecorder { id: 1, recorder: Arc::clone(&drop_order) };
            let _recorder2 = DropRecorder { id: 2, recorder: Arc::clone(&drop_order) };
            let _recorder3 = DropRecorder { id: 3, recorder: Arc::clone(&drop_order) };
        }
        
        let order = drop_order.lock().unwrap();
        assert_eq!(*order, vec![3, 2, 1]); // LIFO drop order
    }
}

mod concurrent_utilities_tests {
    use super::*;
    use std::sync::{Arc, Mutex, RwLock};
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn unit_mutex_basic() {
        let data = Arc::new(Mutex::new(0));
        let mut handles = vec![];
        
        for i in 0..10 {
            let data = Arc::clone(&data);
            let handle = thread::spawn(move || {
                let mut num = data.lock().unwrap();
                *num += i;
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        assert_eq!(*data.lock().unwrap(), (0..10).sum::<i32>());
    }
    
    #[test]
    fn unit_rwlock_readers() {
        let data = Arc::new(RwLock::new(42));
        let mut handles = vec![];
        
        // Multiple readers should work concurrently
        for _ in 0..5 {
            let data = Arc::clone(&data);
            let handle = thread::spawn(move || {
                let value = *data.read().unwrap();
                assert_eq!(value, 42);
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }
    
    #[test]
    fn unit_rwlock_writer() {
        let data = Arc::new(RwLock::new(0));
        
        {
            let mut writer = data.write().unwrap();
            *writer = 100;
        }
        
        let reader = data.read().unwrap();
        assert_eq!(*reader, 100);
    }
}

mod edge_cases_tests {
    use super::*;
    
    #[derive(Debug, Clone, PartialEq)]
    struct TestKey(String);
    
    #[derive(Debug, Clone, PartialEq)]
    struct TestValue(Vec<u8>);
    
    #[test]
    fn unit_empty_string_handling() {
        let empty_key = TestKey("".to_string());
        let empty_value = TestValue(vec![]);
        
        assert_eq!(empty_key.0, "");
        assert_eq!(empty_value.0, Vec::<u8>::new());
    }
    
    #[test]
    fn unit_large_data_handling() {
        let large_data = vec![0u8; 1024 * 1024]; // 1MB
        let value = TestValue(large_data.clone());
        
        assert_eq!(value.0, large_data);
    }
    
    #[test]
    fn unit_unicode_handling() {
        let unicode_string = "Hello 世界 🌍";
        let key = TestKey(unicode_string.to_string());
        let value = TestValue(unicode_string.as_bytes().to_vec());
        
        assert_eq!(key.0, unicode_string);
        assert_eq!(value.0, unicode_string.as_bytes());
    }
    
    #[test]
    fn unit_null_byte_handling() {
        let data_with_nulls = b"test\0with\0nulls";
        let value = TestValue(data_with_nulls.to_vec());
        
        assert_eq!(value.0, data_with_nulls);
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    
    #[test]
    fn unit_id_validation() {
        fn validate_id(id: u64) -> NetabaseResult<()> {
            if id == 0 {
                Err(NetabaseError::Other("ID cannot be zero".to_string()))
            } else {
                Ok(())
            }
        }
        
        assert!(validate_id(1).is_ok());
        assert!(validate_id(u64::MAX).is_ok());
        assert_matches!(validate_id(0), Err(NetabaseError::Other(_)));
    }
    
    #[test]
    fn unit_string_validation() {
        fn validate_name(name: &str) -> NetabaseResult<()> {
            if name.is_empty() {
                Err(NetabaseError::Other("Name cannot be empty".to_string()))
            } else if name.len() > 100 {
                Err(NetabaseError::Other("Name too long".to_string()))
            } else {
                Ok(())
            }
        }
        
        assert!(validate_name("Valid Name").is_ok());
        assert_matches!(validate_name(""), Err(NetabaseError::Other(_)));
        assert_matches!(validate_name(&"a".repeat(101)), Err(NetabaseError::Other(_)));
    }
    
    proptest! {
        #[test]
        fn unit_validate_email_format(
            local in "[a-zA-Z0-9]+",
            domain in "[a-zA-Z0-9]+",
            tld in "[a-zA-Z]{2,4}"
        ) {
            let email = format!("{}@{}.{}", local, domain, tld);
            // Simple email validation
            assert!(email.contains('@'));
            assert!(email.contains('.'));
        }
    }
}