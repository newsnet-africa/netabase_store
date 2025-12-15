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
pub enum NetabasePermissions {
    Database {
        level: TablePermissionLevel,
        can_create_tables: bool,
        can_drop_tables: bool,
        can_alter_schema: bool,
    },
    Model {
        operations: Vec<ModelOperationPermission>,
    },
}

impl Default for NetabasePermissions {
    fn default() -> Self {
        Self::Database {
            level: TablePermissionLevel::ReadOnly,
            can_create_tables: false,
            can_drop_tables: false,
            can_alter_schema: false,
        }
    }
}

impl NetabasePermissions {
    pub fn database_read_only() -> Self {
        Self::Database {
            level: TablePermissionLevel::ReadOnly,
            can_create_tables: false,
            can_drop_tables: false,
            can_alter_schema: false,
        }
    }

    pub fn database_read_write() -> Self {
        Self::Database {
            level: TablePermissionLevel::ReadWrite,
            can_create_tables: false,
            can_drop_tables: false,
            can_alter_schema: false,
        }
    }

    pub fn database_admin() -> Self {
        Self::Database {
            level: TablePermissionLevel::Admin,
            can_create_tables: true,
            can_drop_tables: true,
            can_alter_schema: true,
        }
    }

    pub fn model_read_only() -> Self {
        Self::Model {
            operations: vec![ModelOperationPermission::Read],
        }
    }

    pub fn model_read_write() -> Self {
        Self::Model {
            operations: vec![
                ModelOperationPermission::Read,
                ModelOperationPermission::Create,
                ModelOperationPermission::Update,
                ModelOperationPermission::Delete,
            ],
        }
    }

    pub fn model_admin() -> Self {
        Self::Model {
            operations: vec![ModelOperationPermission::All],
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
            Self::Model { operations } => {
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
            Self::Model { operations } => {
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
