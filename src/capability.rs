use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::marker::PhantomData;
use libp2p::PeerId;
use crate::prelude::{NetabaseDefinition, NetabaseModel};
use crate::traits::registery::models::keys::NetabaseModelKeys;
use crate::primitives::{NamespaceId, Operation, Signature, SubspaceId};
use strum::IntoDiscriminant;

/// The granular scope of data access from an Owner (Subspace)
#[derive(Serialize, Deserialize)]
#[serde(bound = "<M::Keys as NetabaseModelKeys<D, M>>::Primary: Serialize + DeserializeOwned")]
pub enum DataScope<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
{
    /// Access all data owned by `owner` in model `M`
    Subspace {
        owner: SubspaceId,
    },
    /// Access a directory-like prefix of data owned by `owner`
    Prefix {
        owner: SubspaceId,
        prefix: Vec<u8>, 
    },
    /// Access a specific record (path) owned by `owner`
    Path {
        owner: SubspaceId,
        path: <M::Keys as NetabaseModelKeys<D, M>>::Primary,
    },
}

impl<D, M> DataScope<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
{
    pub fn owner(&self) -> &SubspaceId {
        match self {
            Self::Subspace { owner } => owner,
            Self::Prefix { owner, .. } => owner,
            Self::Path { owner, .. } => owner,
        }
    }
}

// Manual implementations
impl<D, M> Clone for DataScope<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::Subspace { owner } => Self::Subspace { owner: *owner },
            Self::Prefix { owner, prefix } => Self::Prefix { owner: *owner, prefix: prefix.clone() },
            Self::Path { owner, path } => Self::Path { owner: *owner, path: path.clone() },
        }
    }
}

impl<D, M> std::fmt::Debug for DataScope<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Subspace { owner } => f.debug_struct("Subspace").field("owner", owner).finish(),
            Self::Prefix { owner, prefix } => f.debug_struct("Prefix").field("owner", owner).field("prefix", prefix).finish(),
            Self::Path { owner, path } => f.debug_struct("Path").field("owner", owner).field("path", path).finish(),
        }
    }
}

impl<D, M> PartialEq for DataScope<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Subspace { owner: a }, Self::Subspace { owner: b }) => a == b,
            (Self::Prefix { owner: oa, prefix: pa }, Self::Prefix { owner: ob, prefix: pb }) => oa == ob && pa == pb,
            (Self::Path { owner: oa, path: pa }, Self::Path { owner: ob, path: pb }) => oa == ob && pa == pb,
            _ => false,
        }
    }
}

impl<D, M> Eq for DataScope<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
{}

/// Operational Capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound = "DataScope<D, M>: Serialize + DeserializeOwned")]
pub struct Capability<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: std::fmt::Debug,
{
    pub operation: Operation,
    pub namespace: NamespaceId, // Added back
    pub scope: DataScope<D, M>,
    pub grantee: PeerId,
    pub expires: u64,
    pub signature: Signature, // Must be signed by scope.owner()
}

impl<D, M> Capability<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: std::fmt::Debug,
{
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.expires < now
    }
}

// Implement NetabasePath for DataScope to convert to bytes for network/storage checks
use crate::key::NetabasePath;

impl<D, M> NetabasePath for DataScope<D, M>
where
    D: NetabaseDefinition,
    M: NetabaseModel<D>,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static,
    <M::Keys as NetabaseModelKeys<D, M>>::Primary: std::fmt::Debug + serde::Serialize,
{
    fn to_path(&self) -> Vec<u8> {
        // Path structure: [Owner] ++ [Path]
        // Note: The "Table" prefix is implicit in the Model type M. 
        // If this capability is used to query the Store, the Store adds the Table prefix.
        // If this is for Network P2P, the Owner (Subspace) is the first component.
        
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.owner().as_ref());

        match self {
            DataScope::Subspace { .. } => {
                // Just owner prefix
            }
            DataScope::Prefix { prefix, .. } => {
                bytes.extend_from_slice(prefix);
            }
            DataScope::Path { path, .. } => {
                 if let Ok(p_bytes) = crate::postcard::to_allocvec(path) {
                     bytes.extend(p_bytes);
                }
            }
        }
        bytes
    }
}
