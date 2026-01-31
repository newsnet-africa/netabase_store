use serde::{Serialize, Deserialize, de::DeserializeOwned};
#[cfg(feature = "libp2p")]
use libp2p::PeerId;
use crate::{
    prelude::{NetabaseDefinition, NetabaseModel},
    traits::registery::models::NetabaseModelKeys,
};
use strum::IntoDiscriminant;

use crate::primitives::{NamespaceId, Operation, Signature, SubspaceId};
use crate::node_metadata::{PublicNodeData, NodePublicKey};
use crate::primitives::EntryPath;
use crate::key::NetabasePath;

pub type CapabilityExpiration = u64;
pub type CapabilitySignature = Signature;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(bound = "K: Serialize + DeserializeOwned")]
pub enum PathRange<K> {
    /// Matches any key that starts with the given byte prefix.
    /// Useful for hierarchical keys (e.g. "users/admin/").
    PathPrefix(EntryPath),
    /// Matches any key strictly within [start, end].
    Range {
        start: K,
        end: K
    },
    /// Matches exactly one key.
    Exact(K),
    /// Matches all keys (Unbounded).
    All,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(bound = "M::Keys: NetabaseModelKeys<D, M>, <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned, <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned")]
pub enum CapabilityRange<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    /// Access to the entire table (all rows, all indexes).
    FullTable,
    /// Restrict access by Primary Key range.
    PrimaryRange(PathRange<<M::Keys as NetabaseModelKeys<D, M>>::Primary>),
    /// Restrict access by Secondary Key range.
    SecondaryRange(PathRange<<M::Keys as NetabaseModelKeys<D, M>>::Secondary>),
    /// Restrict access by Owner (Publisher).
    /// Only allows accessing records signed/published by this key.
    Owner(NodePublicKey),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(bound = "CapabilityRange<D, M>: Serialize + DeserializeOwned")]
pub enum CapabilityPermission<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    Read(CapabilityRange<D, M>),
    Write(CapabilityRange<D, M>),
    Mint(CapabilityRange<D, M>), // Permission to delegate/mint new capabilities
    Store(CapabilityRange<D, M>), // Permission to host/replicate data
}

impl<D, M> CapabilityPermission<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    pub fn range(&self) -> &CapabilityRange<D, M> {
        match self {
            Self::Read(r) | Self::Write(r) | Self::Mint(r) | Self::Store(r) => r,
        }
    }
    
    pub fn operation_type(&self) -> Operation {
        match self {
            Self::Read(_) => Operation::Read,
            Self::Write(_) => Operation::Write,
            Self::Mint(_) => Operation::Mint,
            Self::Store(_) => Operation::Store,
        }
    }

    /// Check if `self` is a subset of `other` (i.e., `other` allows `self`).
    /// `other` is the parent capability (the grantor).
    /// `self` is the child capability (the grantee).
    pub fn is_subset_of(&self, other: &Self) -> bool {
        // 1. Check Operation Type hierarchy
        match (self, other) {
            // Read is allowed by Read, Write, Mint (usually implies Read)
            (Self::Read(a), Self::Read(b)) => a.is_subset_of(b),
            (Self::Read(a), Self::Write(b)) => a.is_subset_of(b), // Write implies Read
            (Self::Read(a), Self::Mint(b)) => a.is_subset_of(b),  // Mint implies full access usually
            
            // Write allowed by Write, Mint
            (Self::Write(a), Self::Write(b)) => a.is_subset_of(b),
            (Self::Write(a), Self::Mint(b)) => a.is_subset_of(b),

            // Store allowed by Store, Write, Mint
            (Self::Store(a), Self::Store(b)) => a.is_subset_of(b),
            (Self::Store(a), Self::Write(b)) => a.is_subset_of(b),
            (Self::Store(a), Self::Mint(b)) => a.is_subset_of(b),

            // Mint only allowed by Mint
            (Self::Mint(a), Self::Mint(b)) => a.is_subset_of(b),

            _ => false,
        }
    }
}

impl<D, M> CapabilityRange<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    pub fn is_subset_of(&self, other: &Self) -> bool {
        match (other, self) {
            // FullTable contains everything
            (Self::FullTable, _) => true,
            
            // Same types compare ranges
            (Self::PrimaryRange(parent_r), Self::PrimaryRange(child_r)) => {
                parent_r.includes(child_r)
            },
            (Self::SecondaryRange(parent_r), Self::SecondaryRange(child_r)) => {
                parent_r.includes(child_r)
            },
            (Self::Owner(parent_pk), Self::Owner(child_pk)) => {
                parent_pk == child_pk
            },

            // Cross-type:
            // Usually different types are disjoint, unless we implement complex resolution.
            // e.g. Does "PrimaryRange(All)" contain "SecondaryRange(Some)"?
            // Technically yes, but practically hard to verify without data access.
            // For authorization delegation, we usually require strict type matching or FullTable.
            _ => false,
        }
    }
}

impl<K: PartialOrd + Eq> PathRange<K> {
    pub fn includes(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::All, _) => true,
            
            (Self::Exact(p1), Self::Exact(p2)) => p1 == p2,
            (Self::Exact(p1), Self::Range { start, end }) => p1 >= start && p1 <= end, // Technically Range could be single point
            
            (Self::Range { start: s1, end: e1 }, Self::Exact(p2)) => p2 >= s1 && p2 <= e1,
            (Self::Range { start: s1, end: e1 }, Self::Range { start: s2, end: e2 }) => {
                s1 <= s2 && e1 >= e2
            },

            (Self::PathPrefix(p1), Self::PathPrefix(p2)) => {
                p2.0.starts_with(&p1.0)
            },
            // Cannot easily compare PathPrefix with Key Range without knowing Key encoding
            _ => false,
        }
    }
}

/// Capability with Type Safety
#[cfg(feature = "libp2p")]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(bound = "CapabilityPermission<D, M>: Serialize + DeserializeOwned")]
pub struct Capability<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    D::SubscriptionKeysDiscriminant: Serialize + DeserializeOwned + std::fmt::Debug + Clone + PartialEq + Eq,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    pub subscription: D::SubscriptionKeysDiscriminant,
    pub owner: PublicNodeData,
    pub granted_by: PublicNodeData,
    pub issued_to: PublicNodeData,
    pub resource: CapabilityPermission<D, M>,
    pub expiry: CapabilityExpiration,
    pub signature: CapabilitySignature,
    
    // Chain support
    pub delegation: Option<Box<Capability<D, M>>>,
}

#[cfg(feature = "libp2p")]
impl<D, M> Capability<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    D::SubscriptionKeysDiscriminant: Serialize + DeserializeOwned + std::fmt::Debug + Clone + PartialEq + Eq,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.expiry < now
    }

    /// Verify the delegation chain of this capability.
    /// 
    /// Ensures that:
    /// 1. The capability is not expired.
    /// 2. The chain leads back to the `root_owner_key` (the resource owner).
    /// 3. Each step in the chain delegates a valid subset of permissions.
    /// 4. The signatures are valid (TODO: Signature verification implementation).
    pub fn verify_chain(&self, root_owner_key: &NodePublicKey) -> bool {
        if self.is_expired() {
            return false;
        }

        // Verify that this capability was indeed issued to the current owner
        // (This check assumes 'self' is the leaf capability presented by 'self.owner')
        // In a chain A -> B -> C:
        // C holds Cap(granted_by: B, issued_to: C, delegation: Cap(granted_by: A, issued_to: B))
        
        if let Some(parent) = &self.delegation {
            // Recursive check
            if !parent.verify_chain(root_owner_key) {
                return false;
            }

            // Link check: Parent must have been issued to the current grantor
            if parent.issued_to != self.granted_by {
                return false;
            }

            // Permission check: Child must be subset of Parent
            if !self.resource.is_subset_of(&parent.resource) {
                return false;
            }

            // TODO: Verify self.signature with self.granted_by.public_key
            
            true
        } else {
            // Root capability check
            // A Root capability is granted by the Resource Owner to themselves (or directly to someone else)
            // It must be signed by the Resource Owner.
            
            if &self.granted_by.public_key != root_owner_key {
                return false;
            }
            
            // TODO: Verify self.signature with root_owner_key
            
            true
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Use postcard for consistent serialization
        crate::postcard::to_allocvec(self).unwrap_or_default()
    }
}

impl<D, M> NetabasePath for CapabilityRange<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    <M::Keys as NetabaseModelKeys<D, M>>::Secondary: Serialize + DeserializeOwned + std::fmt::Debug + Clone + Eq + PartialOrd,
    M::Keys: std::fmt::Debug + Clone + Eq,
{
    fn to_path(&self) -> Vec<u8> {
        match self {
            CapabilityRange::FullTable => Vec::new(),
            CapabilityRange::Owner(pk) => pk.0.to_vec(),
            CapabilityRange::PrimaryRange(PathRange::PathPrefix(p)) => p.0.clone(),
            CapabilityRange::PrimaryRange(PathRange::Range { start, .. }) |
            CapabilityRange::PrimaryRange(PathRange::Exact(start)) => {
                 if let Ok(p_bytes) = crate::postcard::to_allocvec(start) {
                     p_bytes
                } else {
                    Vec::new()
                }
            }
            CapabilityRange::PrimaryRange(PathRange::All) => Vec::new(),
            // Secondary ranges don't map cleanly to a single linear path for DHT/Content Addressing
            // They are query constraints.
            CapabilityRange::SecondaryRange(_) => Vec::new(),
        }
    }
}