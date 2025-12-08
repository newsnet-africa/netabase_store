use crate::traits::{
    definition::DiscriminantName,
    model::{NetabaseModelTrait, key::NetabaseModelKeyTrait},
};
use std::collections::HashMap;
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

pub type TreeName = String;

/// Contains discriminants for secondary keys of a specific model
#[derive(Debug)]
pub struct SecondaryKeyTrees<SecEnum>
where
    SecEnum: IntoDiscriminant + Clone + Debug,
    SecEnum::Discriminant: IntoEnumIterator + std::hash::Hash + Eq + Debug + Send + Sync + Clone + DiscriminantName,
{
    pub trees: HashMap<SecEnum::Discriminant, TreeName>,
}

/// Contains discriminants for relational keys of a specific model
pub struct RelationalKeyTrees<RelEnum>
where
    RelEnum: IntoDiscriminant + Clone + Debug,
    RelEnum::Discriminant: IntoEnumIterator + std::hash::Hash + Eq + Debug + Send + Sync + Clone + DiscriminantName,
{
    pub trees: HashMap<RelEnum::Discriminant, TreeName>,
}

/// Contains hash tree information for a model using Blake3 and generic M
#[derive(Debug, Clone)]
pub struct HashTree<M, D>
where
    M: crate::traits::model::NetabaseModelTrait<D>,
    D: crate::traits::definition::NetabaseDefinition,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: DiscriminantName + Clone,
{
    pub blake3_hash: blake3::Hash,
    pub tree_name: TreeName,
    _marker: std::marker::PhantomData<(M, D)>,
}

impl<M, D> HashTree<M, D>
where
    M: crate::traits::model::NetabaseModelTrait<D>,
    D: crate::traits::definition::NetabaseDefinition,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: DiscriminantName + Clone,
{
    pub fn new(hash: blake3::Hash, tree_name: TreeName) -> Self {
        Self {
            blake3_hash: hash,
            tree_name,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Information about all trees for a specific model - simplified with single generic M
pub struct ModelTrees<M, D>
where
    M: crate::traits::model::NetabaseModelTrait<D>,
    D: crate::traits::definition::NetabaseDefinition,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: DiscriminantName + Clone,
    // Add the required TryFrom bounds for SecondaryEnum
    Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum>,
    <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: TryFrom<Vec<u8>>,
    // Add the required TryFrom bounds for RelationalEnum  
    Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum>,
    <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: TryFrom<Vec<u8>>,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as strum::IntoDiscriminant>::Discriminant: std::clone::Clone,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as strum::IntoDiscriminant>::Discriminant: DiscriminantName,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as strum::IntoDiscriminant>::Discriminant: std::clone::Clone,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as strum::IntoDiscriminant>::Discriminant: DiscriminantName
{
    pub main_tree: TreeName,
    pub secondary_keys: SecondaryKeyTrees<
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    >,
    pub relational_keys: RelationalKeyTrees<
        <M::Keys as crate::traits::model::key::NetabaseModelKeyTrait<D, M>>::RelationalEnum,
    >,
    pub hash_tree: Option<HashTree<M, D>>,
}

/// The central management structure for all trees - simplified without Box<Any>
/// This now simply tracks which models exist and delegates to TreeManager for tree names
#[derive(Debug, Clone)]
pub struct AllTrees<D>
where
    D: IntoDiscriminant,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    pub registered_models: Vec<D::Discriminant>,
}

impl<D> AllTrees<D>
where
    D: IntoDiscriminant,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    pub fn new() -> Self {
        Self {
            registered_models: Vec::new(),
        }
    }

    /// Register a model - this replaces the complex add_model_trees method
    pub fn register_model(&mut self, model_discriminant: D::Discriminant) {
        if !self.registered_models.contains(&model_discriminant) {
            self.registered_models.push(model_discriminant);
        }
    }

    /// Get all registered models
    pub fn get_registered_models(&self) -> &[D::Discriminant] {
        &self.registered_models
    }
}

pub trait TreeManager<D>
where
    D: IntoDiscriminant,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Returns the complete tree structure for this definition
    fn all_trees() -> AllTrees<D>;

    /// Get the main tree name for a specific model using DiscriminantName trait
    fn get_tree_name(model_discriminant: &D::Discriminant) -> Option<TreeName> {
        Some(model_discriminant.name().to_string())
    }

    /// Get secondary tree names for a model using DiscriminantName
    fn get_secondary_tree_names(model_discriminant: &D::Discriminant) -> Vec<TreeName>;

    /// Get relational tree names for a model using DiscriminantName
    fn get_relational_tree_names(model_discriminant: &D::Discriminant) -> Vec<TreeName>;
}
