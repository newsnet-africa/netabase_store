use serde::{Deserialize, Deserializer, Serializer};
use std::time::SystemTime;

pub mod serde_system_time_option {
    use super::*;

    pub fn serialize<S>(value: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(time) => {
                let duration = time.duration_since(SystemTime::UNIX_EPOCH).map_err(|e| {
                    serde::ser::Error::custom(format!("SystemTime before UNIX EPOCH: {}", e))
                })?;
                serializer.serialize_some(&duration.as_nanos())
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<SystemTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nanos: Option<u128> = Option::deserialize(deserializer)?;
        match nanos {
            Some(n) => {
                // Potential truncation if u128, but standard SystemTime is u64 seconds
                // Actually duration_from_nanos takes u64. u128 nanos might overflow u64.
                // Let's use as_secs and subsec_nanos
                let secs = (n / 1_000_000_000) as u64;
                let subsec = (n % 1_000_000_000) as u32;
                Ok(Some(SystemTime::UNIX_EPOCH + std::time::Duration::new(secs, subsec)))
            }
            None => Ok(None),
        }
    }
}
