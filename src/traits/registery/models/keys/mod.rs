use crate::traits::registery::models::keys::blob::NetabaseModelBlobKey;
pub use crate::traits::registery::models::keys::primary::NetabaseModelPrimaryKey;
pub use crate::traits::registery::models::keys::relational::{
    NetabaseModelRelationalKey, NetabaseModelRelationalKeyForeign,
};
pub use crate::traits::registery::models::keys::secondary::NetabaseModelSecondaryKey;
pub use crate::traits::registery::models::keys::subscription::NetabaseModelSubscriptionKey;
use crate::traits::registery::{
    definition::NetabaseDefinition, models::model::NetabaseModelMarker,
};

pub mod blob;
pub mod primary;
pub mod relational;
pub mod secondary;
pub mod subscription;

/// Trait that defines all key types for a model.
///
/// The key type traits (NetabaseModelPrimaryKey, etc.) are now simplified
/// without the K parameter and without lifetimes to avoid early/late-bound lifetime issues.
pub trait NetabaseModelKeys<D: NetabaseDefinition, M: NetabaseModelMarker<D>>:
    std::marker::Sized
where
    D::Discriminant: 'static + std::fmt::Debug,
    Self::Primary: redb::Key + 'static,
    Self::Secondary: redb::Key + 'static,
    Self::Relational: redb::Key + 'static,
    Self::Subscription: redb::Key + 'static,
{
    type Primary: NetabaseModelPrimaryKey<D, M>;
    type Secondary: NetabaseModelSecondaryKey<D, M>;
    type Relational: NetabaseModelRelationalKey<D, M>;
    type Subscription: NetabaseModelSubscriptionKey<D, M> + 'static;
    type Blob: NetabaseModelBlobKey<D, M>;
}
