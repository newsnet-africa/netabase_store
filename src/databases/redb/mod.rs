pub mod transaction;

use strum::EnumDiscriminants;
use crate::traits::registery::definition::NetabaseDefinition;

pub struct RedbStore<D: NetabaseDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    tree_names: D::TreeNames,
    db: RedbStorePermissions,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelOperationPermission {
    Read,
    Create,
    Update,
    Delete,
    All,
}

#[derive(Debug, Clone)]
pub enum TablePermissionLevel {
    ReadOnly,
    ReadWrite,
    Admin,
}

#[derive(Debug, Clone)]
pub enum NetabasePermissions<D: NetabaseDefinition> 
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    Database {
        level: TablePermissionLevel,
        can_create_tables: bool,
        can_drop_tables: bool,
        can_alter_schema: bool,
        tables: Vec<D::ModelTableDefinition>,
    },
    Model {
        operations: Vec<ModelOperationPermission>,
        table: D::ModelTableDefinition,
    },
}

#[derive(EnumDiscriminants)]
#[strum_discriminants(name(DefinitionPermissions))]
pub enum RedbStorePermissions {
    ReadOnly(redb::ReadOnlyDatabase),
    ReadWrite(redb::Database),
}

impl<D: NetabaseDefinition> Default for NetabasePermissions<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn default() -> Self {
        Self::Database {
            level: TablePermissionLevel::ReadOnly,
            can_create_tables: false,
            can_drop_tables: false,
            can_alter_schema: false,
            tables: Vec::new(),
        }
    }
}

impl<D: NetabaseDefinition> NetabasePermissions<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    pub fn database_read_only() -> Self {
        Self::Database {
            level: TablePermissionLevel::ReadOnly,
            can_create_tables: false,
            can_drop_tables: false,
            can_alter_schema: false,
            tables: Vec::new(),
        }
    }

    pub fn database_read_write() -> Self {
        Self::Database {
            level: TablePermissionLevel::ReadWrite,
            can_create_tables: false,
            can_drop_tables: false,
            can_alter_schema: false,
            tables: Vec::new(),
        }
    }

    pub fn database_admin() -> Self {
        Self::Database {
            level: TablePermissionLevel::Admin,
            can_create_tables: true,
            can_drop_tables: true,
            can_alter_schema: true,
            tables: Vec::new(),
        }
    }

    pub fn model_read_only(table: D::ModelTableDefinition) -> Self {
        Self::Model {
            operations: vec![ModelOperationPermission::Read],
            table,
        }
    }

    pub fn model_read_write(table: D::ModelTableDefinition) -> Self {
        Self::Model {
            operations: vec![
                ModelOperationPermission::Read,
                ModelOperationPermission::Create,
                ModelOperationPermission::Update,
                ModelOperationPermission::Delete,
            ],
            table,
        }
    }

    pub fn model_admin(table: D::ModelTableDefinition) -> Self {
        Self::Model {
            operations: vec![ModelOperationPermission::All],
            table,
        }
    }

    pub fn can_perform_operation(&self, operation: &ModelOperationPermission) -> bool {
        match self {
            Self::Database { level, .. } => {
                match level {
                    TablePermissionLevel::ReadOnly => {
                        matches!(operation, ModelOperationPermission::Read)
                    },
                    TablePermissionLevel::ReadWrite => {
                        !matches!(operation, ModelOperationPermission::All)
                    },
                    TablePermissionLevel::Admin => true,
                }
            },
            Self::Model { operations, .. } => {
                operations.contains(operation) 
                    || operations.contains(&ModelOperationPermission::All)
            }
        }
    }

    pub fn can_read(&self) -> bool {
        self.can_perform_operation(&ModelOperationPermission::Read)
    }

    pub fn can_write(&self) -> bool {
        match self {
            Self::Database { level, .. } => {
                !matches!(level, TablePermissionLevel::ReadOnly)
            },
            Self::Model { operations, .. } => {
                operations.iter().any(|op| {
                    matches!(op, 
                        ModelOperationPermission::Create | 
                        ModelOperationPermission::Update | 
                        ModelOperationPermission::Delete |
                        ModelOperationPermission::All
                    )
                })
            }
        }
    }
}
