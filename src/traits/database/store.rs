use std::path::Path;
use crate::traits::permissions::DefinitionPermissions;
use crate::traits::registery::definition::NetabaseDefinition;
use crate::errors::NetabaseResult;

pub trait NBStore<D: NetabaseDefinition>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
    /// Create a new database store with the given path and permissions
    /// The permissions determine read/write access at the database level
    fn new<P: AsRef<Path>>(path: P, permissions: DefinitionPermissions<'static, D>) -> NetabaseResult<Self>
    where
        Self: Sized,
        D::TreeNames: Default;

    fn execute_transaction<F: Fn()>(f: F);
}
