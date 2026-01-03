//! Hash algorithm traits for configurable schema hashing.
//!
//! This module provides hash algorithm abstractions for P2P node comparison.
//! Users can choose between fast hashes for performance or cryptographic
//! hashes for collision resistance in distributed systems.
//!
//! # Available Algorithms
//!
//! - `FastHash`: Uses FxHash for maximum performance
//! - `DefaultHash`: Uses SipHash (Rust's default) for balance
//! - `CryptoHash`: Uses SHA256 for cryptographic strength
//!
//! # Example
//!
//! ```rust,ignore
//! use netabase_store::traits::database::hash::{FastHash, CryptoHash};
//!
//! // Quick comparison using fast hash
//! let fast_hash = MyRepo::schema_hash::<FastHash>();
//!
//! // Secure comparison using cryptographic hash
//! let crypto_hash = MyRepo::schema_hash::<CryptoHash>();
//! ```

use std::hash::{Hash, Hasher};

/// Trait for hash algorithms used in schema comparison.
///
/// Implement this trait to provide custom hash algorithms for
/// repository schema comparison in P2P contexts.
pub trait HashAlgorithm {
    /// The hasher type to use.
    type Hasher: Hasher + Default;

    /// Create a new hasher instance.
    fn new_hasher() -> Self::Hasher {
        Self::Hasher::default()
    }

    /// Hash a string and return the hash value.
    fn hash_string(s: &str) -> u64 {
        let mut hasher = Self::new_hasher();
        s.hash(&mut hasher);
        hasher.finish()
    }

    /// Hash arbitrary bytes and return the hash value.
    fn hash_bytes(bytes: &[u8]) -> u64 {
        let mut hasher = Self::new_hasher();
        bytes.hash(&mut hasher);
        hasher.finish()
    }
}

/// Fast hash algorithm using FxHash.
///
/// Provides maximum performance for local operations where
/// cryptographic collision resistance is not required.
///
/// **Best for**: Local schema caching, quick comparisons, non-adversarial environments.
#[derive(Debug, Clone, Copy)]
pub struct FastHash;

impl HashAlgorithm for FastHash {
    type Hasher = rustc_hash::FxHasher;
}

/// Default hash algorithm using SipHash.
///
/// Provides a good balance between speed and collision resistance.
/// This is Rust's default hash algorithm.
///
/// **Best for**: General-purpose hashing, trusted network environments.
#[derive(Debug, Clone, Copy)]
pub struct DefaultHash;

impl HashAlgorithm for DefaultHash {
    type Hasher = std::collections::hash_map::DefaultHasher;
}

/// Cryptographic hash algorithm using SHA256.
///
/// Provides strong collision resistance for distributed systems
/// where adversarial nodes might attempt to craft collisions.
///
/// **Best for**: P2P networks, untrusted environments, security-critical comparisons.
///
/// Note: This wraps SHA256 into the `Hasher` trait for compatibility.
#[derive(Debug, Clone, Copy)]
pub struct CryptoHash;

/// SHA256 hasher wrapper implementing `std::hash::Hasher`.
///
/// This wraps the SHA256 algorithm to work with the standard `Hasher` trait.
/// The final hash value is derived from the first 8 bytes of the SHA256 output.
#[derive(Default)]
pub struct Sha256Hasher {
    data: Vec<u8>,
}

impl Hasher for Sha256Hasher {
    fn finish(&self) -> u64 {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(&self.data);
        // Take first 8 bytes as u64
        u64::from_le_bytes([
            hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7],
        ])
    }

    fn write(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
}

impl HashAlgorithm for CryptoHash {
    type Hasher = Sha256Hasher;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_hash() {
        let hash1 = FastHash::hash_string("test");
        let hash2 = FastHash::hash_string("test");
        let hash3 = FastHash::hash_string("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_default_hash() {
        let hash1 = DefaultHash::hash_string("test");
        let hash2 = DefaultHash::hash_string("test");
        let hash3 = DefaultHash::hash_string("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_crypto_hash() {
        let hash1 = CryptoHash::hash_string("test");
        let hash2 = CryptoHash::hash_string("test");
        let hash3 = CryptoHash::hash_string("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_different_algorithms_produce_different_hashes() {
        let input = "same input";
        let fast = FastHash::hash_string(input);
        let _default = DefaultHash::hash_string(input);
        let crypto = CryptoHash::hash_string(input);

        // Different algorithms should produce different hashes
        // (statistically extremely unlikely to be equal)
        assert_ne!(fast, crypto);
        // Note: fast and default might occasionally collide but it's unlikely
    }
}
