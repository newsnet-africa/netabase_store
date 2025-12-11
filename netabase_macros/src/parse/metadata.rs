//! Metadata structures for parsed netabase definitions
//!
//! These structures hold all the information extracted from the AST during
//! parsing. They provide a clean interface between the parsing phase and
//! the code generation phase.

use syn::{Ident, Type, Visibility, Path};

/// Permission granted to a child module by its parent
#[derive(Debug, Clone)]
pub struct ChildPermissionGrant {
    /// Name of the child module
    pub child_name: Ident,
    /// Permission level granted by parent
    pub permission_level: PermissionLevel,
    /// Whether this child can access sibling modules
    pub cross_sibling_access: bool,
}

/// Permission levels in the hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevel {
    /// No access
    None,
    /// Read-only access
    Read,
    /// Write-only access
    Write,
    /// Full read-write access
    ReadWrite,
    /// Full access including permission management
    Admin,
}

impl PermissionLevel {
    pub fn can_read(&self) -> bool {
        matches!(self, PermissionLevel::Read | PermissionLevel::ReadWrite | PermissionLevel::Admin)
    }

    pub fn can_write(&self) -> bool {
        matches!(self, PermissionLevel::Write | PermissionLevel::ReadWrite | PermissionLevel::Admin)
    }

    pub fn can_manage_permissions(&self) -> bool {
        matches!(self, PermissionLevel::Admin)
    }
}

/// Complete metadata for a netabase definition module
#[derive(Debug, Clone)]
pub struct ModuleMetadata {
    /// Name of the Rust module (e.g., "definitions")
    pub module_name: Ident,

    /// Name of the definition enum (e.g., "UserDefinition")
    pub definition_name: Ident,

    /// Name of the keys enum (e.g., "UserDefinitionKeys")
    pub keys_name: Ident,

    /// Available subscription topics declared at module level
    pub available_subscriptions: Vec<Ident>,

    /// All models defined in this module
    pub models: Vec<ModelMetadata>,

    /// Nested definition modules (for hierarchical definitions)
    pub nested_modules: Vec<ModuleMetadata>,

    /// Permission hierarchy - defines which child modules this parent can access
    pub child_permissions: Vec<ChildPermissionGrant>,

    /// Parent module (None if this is root level)
    pub parent_module: Option<Box<ModuleMetadata>>,
}

impl ModuleMetadata {
    /// Create a new empty module metadata
    pub fn new(module_name: Ident, definition_name: Ident, keys_name: Ident) -> Self {
        Self {
            module_name,
            definition_name,
            keys_name,
            available_subscriptions: Vec::new(),
            models: Vec::new(),
            nested_modules: Vec::new(),
            child_permissions: Vec::new(),
            parent_module: None,
        }
    }

    /// Add an available subscription topic
    pub fn add_subscription(&mut self, topic: Ident) {
        if !self.available_subscriptions.contains(&topic) {
            self.available_subscriptions.push(topic);
        }
    }

    /// Add a model to this module
    pub fn add_model(&mut self, model: ModelMetadata) {
        self.models.push(model);
    }

    /// Add a nested module with permission settings
    pub fn add_nested_module(&mut self, module: ModuleMetadata) {
        self.nested_modules.push(module);
    }

    /// Set the parent module reference (for upward navigation)
    pub fn set_parent(&mut self, parent: ModuleMetadata) {
        self.parent_module = Some(Box::new(parent));
    }

    /// Add a child permission grant
    pub fn add_child_permission(&mut self, grant: ChildPermissionGrant) {
        self.child_permissions.push(grant);
    }

    /// Check if a subscription topic is available
    pub fn has_subscription(&self, topic: &Ident) -> bool {
        self.available_subscriptions.iter().any(|t| t == topic)
    }

    /// Get permission level for a specific child module
    pub fn get_child_permission(&self, child_name: &Ident) -> PermissionLevel {
        self.child_permissions
            .iter()
            .find(|grant| grant.child_name == *child_name)
            .map(|grant| grant.permission_level)
            .unwrap_or(PermissionLevel::Read) // Default to read-only
    }

    /// Check if this module is a root-level module (no parent)
    pub fn is_root(&self) -> bool {
        self.parent_module.is_none()
    }

    /// Get all child module names for permission enumeration
    pub fn child_module_names(&self) -> Vec<&Ident> {
        self.nested_modules.iter().map(|m| &m.definition_name).collect()
    }

    /// Get the full hierarchical path (from root to this module)
    pub fn hierarchical_path(&self) -> Vec<String> {
        let mut path = Vec::new();
        if let Some(ref parent) = self.parent_module {
            path.extend(parent.hierarchical_path());
        }
        path.push(self.definition_name.to_string());
        path
    }

    /// Check if this module can access a sibling module
    pub fn can_access_sibling(&self, sibling_name: &Ident) -> bool {
        // Check if parent grants cross-sibling access
        if let Some(ref parent) = self.parent_module {
            return parent.child_permissions
                .iter()
                .find(|grant| grant.child_name == self.definition_name)
                .map(|grant| grant.cross_sibling_access)
                .unwrap_or(false);
        }
        false
    }
}

/// Metadata for a single model struct
#[derive(Debug, Clone)]
pub struct ModelMetadata {
    /// Model name (e.g., "User")
    pub name: Ident,

    /// Visibility (pub, pub(crate), etc.)
    pub vis: Visibility,

    /// All fields in the struct
    pub fields: Vec<FieldMetadata>,

    /// Subscription topics this model subscribes to
    pub subscriptions: Vec<Ident>,
}

impl ModelMetadata {
    /// Create a new model metadata
    pub fn new(name: Ident, vis: Visibility) -> Self {
        Self {
            name,
            vis,
            fields: Vec::new(),
            subscriptions: Vec::new(),
        }
    }

    /// Add a field to this model
    pub fn add_field(&mut self, field: FieldMetadata) {
        self.fields.push(field);
    }

    /// Add a subscription topic
    pub fn add_subscription(&mut self, topic: Ident) {
        if !self.subscriptions.contains(&topic) {
            self.subscriptions.push(topic);
        }
    }

    /// Get the primary key field
    pub fn primary_key_field(&self) -> Option<&FieldMetadata> {
        self.fields.iter().find(|f| f.is_primary_key)
    }

    /// Get all secondary key fields
    pub fn secondary_key_fields(&self) -> Vec<&FieldMetadata> {
        self.fields.iter().filter(|f| f.is_secondary_key).collect()
    }

    /// Get all relational fields
    pub fn relational_fields(&self) -> Vec<&FieldMetadata> {
        self.fields.iter().filter(|f| f.is_relation).collect()
    }

    /// Get all cross-definition relational fields
    pub fn cross_definition_fields(&self) -> Vec<&FieldMetadata> {
        self.fields.iter().filter(|f| f.is_cross_definition()).collect()
    }

    /// Get all local relational fields (not cross-definition)
    pub fn local_relational_fields(&self) -> Vec<&FieldMetadata> {
        self.fields.iter().filter(|f| f.is_relation && !f.is_cross_definition()).collect()
    }

    /// Get all regular (non-key) fields
    pub fn regular_fields(&self) -> Vec<&FieldMetadata> {
        self.fields.iter()
            .filter(|f| !f.is_primary_key && !f.is_secondary_key && !f.is_relation)
            .collect()
    }

    /// Check if this model has any secondary keys
    pub fn has_secondary_keys(&self) -> bool {
        self.fields.iter().any(|f| f.is_secondary_key)
    }

    /// Check if this model has any relations
    pub fn has_relations(&self) -> bool {
        self.fields.iter().any(|f| f.is_relation)
    }

    /// Check if this model has any cross-definition relations
    pub fn has_cross_definition_relations(&self) -> bool {
        self.fields.iter().any(|f| f.is_cross_definition())
    }

    /// Get required permission levels for all cross-definition fields
    pub fn required_cross_permissions(&self) -> Vec<PermissionLevel> {
        self.cross_definition_fields()
            .iter()
            .map(|f| f.required_permission())
            .collect()
    }
}

/// Metadata for a single field in a model
#[derive(Debug, Clone)]
pub struct FieldMetadata {
    /// Field name (e.g., "id", "email")
    pub name: Ident,

    /// Field type (e.g., u64, String)
    pub ty: Type,

    /// Visibility
    pub vis: Visibility,

    /// Whether this field is the primary key
    pub is_primary_key: bool,

    /// Whether this field is a secondary key (indexed)
    pub is_secondary_key: bool,

    /// Whether this field is a relation to another model
    pub is_relation: bool,

    /// Path to cross-definition model (if using #[cross_definition_link])
    pub cross_definition_link: Option<CrossDefinitionLink>,
}

/// Cross-definition relationship information
#[derive(Debug, Clone)]
pub struct CrossDefinitionLink {
    /// Path to the target definition (e.g., "inner::InnerDefinition")
    pub target_path: Path,
    /// Target model name within that definition
    pub target_model: Option<Ident>,
    /// Permission level required to access this link
    pub required_permission: PermissionLevel,
    /// Whether this is a many-to-one or one-to-many relationship
    pub relationship_type: RelationshipType,
}

/// Type of cross-definition relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipType {
    /// One-to-one relationship
    OneToOne,
    /// One-to-many relationship (this model has many of the target)
    OneToMany,
    /// Many-to-one relationship (many of this model belong to one target)
    ManyToOne,
    /// Many-to-many relationship
    ManyToMany,
}

impl FieldMetadata {
    /// Create a new field metadata
    pub fn new(name: Ident, ty: Type, vis: Visibility) -> Self {
        Self {
            name,
            ty,
            vis,
            is_primary_key: false,
            is_secondary_key: false,
            is_relation: false,
            cross_definition_link: None,
        }
    }

    /// Check if this field has any key/relation attributes
    pub fn has_special_attribute(&self) -> bool {
        self.is_primary_key || self.is_secondary_key || self.is_relation
    }

    /// Check if this field is a cross-definition relation
    pub fn is_cross_definition(&self) -> bool {
        self.cross_definition_link.is_some()
    }

    /// Get the required permission level for accessing this field
    pub fn required_permission(&self) -> PermissionLevel {
        if let Some(ref link) = self.cross_definition_link {
            link.required_permission
        } else {
            PermissionLevel::Read // Default for local fields
        }
    }

    /// Get a human-readable description of the field type
    pub fn field_type_description(&self) -> &'static str {
        if self.is_primary_key {
            "primary key"
        } else if self.is_secondary_key {
            "secondary key"
        } else if self.is_relation && self.is_cross_definition() {
            "cross-definition relation"
        } else if self.is_relation {
            "relation"
        } else {
            "regular field"
        }
    }
}

/// Error collector for accumulating multiple parsing errors
#[derive(Debug, Default)]
pub struct ErrorCollector {
    errors: Vec<syn::Error>,
}

impl ErrorCollector {
    /// Create a new error collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error to the collection
    pub fn add(&mut self, error: syn::Error) {
        self.errors.push(error);
    }

    /// Add an error with a specific span and message
    pub fn add_spanned<T: quote::ToTokens>(&mut self, tokens: T, message: impl std::fmt::Display) {
        self.errors.push(syn::Error::new_spanned(tokens, message));
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the number of errors
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Check if the collector is empty
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Convert into a Result, combining all errors if any exist
    pub fn into_result(self) -> Result<(), syn::Error> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            // Combine all errors into one
            let mut iter = self.errors.into_iter();
            let mut combined = iter.next().unwrap();
            for error in iter {
                combined.combine(error);
            }
            Err(combined)
        }
    }

    /// Convert into a Result with a value, combining all errors if any exist
    pub fn into_result_with<T>(self, value: T) -> Result<T, syn::Error> {
        self.into_result().map(|_| value)
    }
}

/// Validation helper for metadata structures
pub struct MetadataValidator;

impl MetadataValidator {
    /// Validate that a model has exactly one primary key
    pub fn validate_primary_key(model: &ModelMetadata, errors: &mut ErrorCollector) {
        let primary_keys: Vec<_> = model.fields.iter()
            .filter(|f| f.is_primary_key)
            .collect();

        match primary_keys.len() {
            0 => {
                errors.add_spanned(
                    &model.name,
                    format!(
                        "Model '{}' must have exactly one field marked with #[primary_key]",
                        model.name
                    )
                );
            }
            1 => { /* Valid */ }
            n => {
                errors.add_spanned(
                    &model.name,
                    format!(
                        "Model '{}' has {} primary keys, but only one is allowed",
                        model.name, n
                    )
                );
            }
        }
    }

    /// Validate that fields don't have conflicting attributes
    pub fn validate_field_attributes(field: &FieldMetadata, errors: &mut ErrorCollector) {
        let attribute_count = [
            field.is_primary_key,
            field.is_secondary_key,
            field.is_relation,
        ].iter().filter(|&&x| x).count();

        if attribute_count > 1 {
            errors.add_spanned(
                &field.name,
                format!(
                    "Field '{}' can only have one of: #[primary_key], #[secondary_key], #[relation]",
                    field.name
                )
            );
        }
    }

    /// Validate that subscriptions are declared at module level
    pub fn validate_subscriptions(
        model: &ModelMetadata,
        available: &[Ident],
        errors: &mut ErrorCollector,
    ) {
        for sub in &model.subscriptions {
            if !available.iter().any(|a| a == sub) {
                errors.add_spanned(
                    sub,
                    format!(
                        "Subscription topic '{}' not declared in module. Available topics: [{}]",
                        sub,
                        available.iter()
                            .map(|i| i.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                );
            }
        }
    }

    /// Validate entire module metadata
    pub fn validate_module(module: &ModuleMetadata) -> Result<(), syn::Error> {
        let mut errors = ErrorCollector::new();

        // Validate each model
        for model in &module.models {
            Self::validate_primary_key(model, &mut errors);
            Self::validate_subscriptions(model, &module.available_subscriptions, &mut errors);

            // Validate each field
            for field in &model.fields {
                Self::validate_field_attributes(field, &mut errors);
            }
        }

        // Recursively validate nested modules
        for nested in &module.nested_modules {
            if let Err(e) = Self::validate_module(nested) {
                errors.add(e);
            }
        }

        errors.into_result()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_module_metadata_new() {
        let module = ModuleMetadata::new(
            parse_quote!(test_def),
            parse_quote!(TestDef),
            parse_quote!(TestDefKeys)
        );

        assert_eq!(module.module_name.to_string(), "test_def");
        assert_eq!(module.definition_name.to_string(), "TestDef");
        assert_eq!(module.keys_name.to_string(), "TestDefKeys");
        assert_eq!(module.models.len(), 0);
        assert_eq!(module.available_subscriptions.len(), 0);
    }

    #[test]
    fn test_field_type_description() {
        let mut field = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );

        assert_eq!(field.field_type_description(), "regular field");

        field.is_primary_key = true;
        assert_eq!(field.field_type_description(), "primary key");

        field.is_primary_key = false;
        field.is_secondary_key = true;
        assert_eq!(field.field_type_description(), "secondary key");
    }

    #[test]
    fn test_error_collector() {
        let mut collector = ErrorCollector::new();
        assert!(!collector.has_errors());

        let tokens: syn::Ident = parse_quote!(test);
        collector.add_spanned(tokens, "error message");
        assert!(collector.has_errors());
        assert_eq!(collector.len(), 1);

        let result = collector.into_result();
        assert!(result.is_err());
    }
}
