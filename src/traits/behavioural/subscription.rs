use crate::traits::structural::definition::NetabaseDefinition;
use crate::traits::structural::values::ModelHash;

pub trait SubscribableModel<D: NetabaseDefinition>: Sized {
    fn subscription_hash(&self) -> ModelHash<Self>;
}
