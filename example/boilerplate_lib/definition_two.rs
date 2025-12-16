// DefinitionTwo module

use crate::boilerplate_lib::models::category::{Category, CategoryKeys};
use netabase_store::traits::registery::definition::NetabaseDefinition;
use netabase_store::traits::registery::definition::NetabaseDefinitionKeys;
use netabase_store::traits::registery::definition::NetabaseDefinitionTreeNames;
use netabase_store::traits::registery::definition::redb_definition::RedbDefinition;
use netabase_store::traits::registery::models::model::RedbModelTableDefinitions;
use netabase_store::traits::registery::models::treenames::ModelTreeNames;
use strum::{AsRefStr, EnumDiscriminants};
use serde::{Serialize, Deserialize};
use bincode::{Encode, Decode};

#[derive(Clone, EnumDiscriminants, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode, PartialOrd, Ord)]
#[strum_discriminants(name(DefinitionTwoDiscriminants))]
#[strum_discriminants(derive(AsRefStr))]
pub enum DefinitionTwo {
    Category(Category),
}

use netabase_store::traits::registery::definition::subscription::{
    DefinitionSubscriptionRegistry, SubscriptionEntry,
};
use netabase_store::traits::permissions::{
    DefinitionPermissions, ModelAccessLevel,
    definition::CrossDefinitionAccess,
};
use crate::boilerplate_lib::GlobalDefinitionKeys;

impl NetabaseDefinition for DefinitionTwo {
    type TreeNames = DefinitionTwoTreeNames;
    type DefKeys = DefinitionTwoKeys;

    const SUBSCRIPTION_REGISTRY: DefinitionSubscriptionRegistry<'static, Self> =
        DefinitionSubscriptionRegistry::new(&[
            SubscriptionEntry {
                topic: "General",
                subscribers: &[DefinitionTwoDiscriminants::Category],
            },
        ]);

    const PERMISSIONS: DefinitionPermissions<'static, Self> = DefinitionPermissions {
        model_access: &[
            (DefinitionTwoDiscriminants::Category, ModelAccessLevel::ReadWrite),
        ],
        cross_definition_access: &[
            // DefinitionTwo doesn't access other definitions
        ],
    };
}

#[derive(Clone, Debug)]
pub enum DefinitionTwoTreeNames {
    Category(ModelTreeNames<'static, DefinitionTwo, Category>),
}

impl NetabaseDefinitionTreeNames<DefinitionTwo> for DefinitionTwoTreeNames {}

#[derive(Clone, Debug)]
pub enum DefinitionTwoKeys {
    Category(CategoryKeys),
}

impl NetabaseDefinitionKeys<DefinitionTwo> for DefinitionTwoKeys {}

impl RedbDefinition for DefinitionTwo {
    type ModelTableDefinition<'db> = RedbModelTableDefinitions<'db, Category, Self>;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode, Serialize, Deserialize, AsRefStr)]
pub enum DefinitionTwoSubscriptions {
    General,
}
