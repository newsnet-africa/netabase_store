use crate::traits::structural::definition::NetabaseDefinition;
use crate::traits::structural::keys::NetabaseModelKeys;
use crate::traits::structural::tables::NetabaseModelTables;
use crate::traits::structural::values::StoreValue;

pub trait NetabaseModel<D: NetabaseDefinition>: Sized + StoreValue + Serialisable {
    type ModelName;
    type Tables: NetabaseModelTables<D, Self>;
    type Keys: NetabaseModelKeys<D, Self>;

    fn primary_key(&self) -> &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary;
    fn secondary_keys(&self) -> Option<&[<Self::Keys as NetabaseModelKeys<D, Self>>::Secondary]>;
    fn relational_keys(&self) -> Option<&[<Self::Keys as NetabaseModelKeys<D, Self>>::Relational]>;
    fn subscription_keys(
        &self,
    ) -> Option<&[<Self::Keys as NetabaseModelKeys<D, Self>>::Subscription]>;
}

pub trait Serialisable: std::marker::Sized {
    type SerialisedType: AsRef<u8>;
    fn serialise(&self) -> Vec<u8>;
    fn deserialise(serialised: impl Into<Self::SerialisedType>) -> impl Into<Self>;
}
