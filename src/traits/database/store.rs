use std::path::Path;
use crate::traits::registery::definition::NetabaseDefinition;
use crate::errors::NetabaseResult;

pub trait NBStore<D: NetabaseDefinition>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
    /// Create a new database store with the given path
    fn new<P: AsRef<Path>>(path: P) -> NetabaseResult<Self>
    where
        Self: Sized,
        D::TreeNames: Default;

    fn execute_transaction<F: Fn()>(f: F);
}
