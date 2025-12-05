use crate::{NetabaseDefinitionTrait, NetabaseModelTrait};

pub trait SubscriptionTree<D: NetabaseDefinitionTrait> {
    fn add_model<M: NetabaseModelTrait<D>>(model: M) {}
}
