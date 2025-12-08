use std::convert::TryInto;
use bincode::{Encode, Decode};

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct TestId(u64);

impl TryFrom<Vec<u8>> for TestId {
    type Error = bincode::error::DecodeError;
    
    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let (value, _): (u64, usize) = bincode::decode_from_slice(&data, bincode::config::standard())?;
        Ok(TestId(value))
    }
}

impl TryFrom<TestId> for Vec<u8> {
    type Error = bincode::error::EncodeError;
    
    fn try_from(value: TestId) -> Result<Self, Self::Error> {
        bincode::encode_to_vec(value.0, bincode::config::standard())
    }
}

#[test]
fn test_basic_conversion() {
    let test_id = TestId(42);
    let bytes: Vec<u8> = test_id.clone().try_into().expect("Failed to convert to bytes");
    let recovered: TestId = bytes.try_into().expect("Failed to convert from bytes");
    assert_eq!(test_id, recovered);
}