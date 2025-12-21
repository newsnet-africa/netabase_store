use crate::blob::NetabaseBlobItem;
use crate::traits::registery::definition::NetabaseDefinition;
use crate::traits::registery::models::StoreKeyMarker;
use crate::traits::registery::models::keys::NetabaseModelKeys;
pub use crate::traits::registery::models::keys::primary::NetabaseModelPrimaryKey;
use crate::traits::registery::models::model::NetabaseModelMarker;
pub trait NetabaseModelBlobKey<
    'a,
    D: NetabaseDefinition,
    M: NetabaseModelMarker<D>,
    K: NetabaseModelKeys<D, M>,
>: StoreKeyMarker<D> + Clone where
    D::Discriminant: 'static + std::fmt::Debug,
    K::Relational<'a>: StoreKeyMarker<D>,
{
    type PrimaryKey: NetabaseModelPrimaryKey<'a, D, M, K>;
    type BlobItem: NetabaseBlobItem;
}
