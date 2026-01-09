use crate::utils::attributes::{
    ContentAddressedAttributeConfig, VersionAttributeConfig, find_attribute, get_version_info,
    has_attribute, parse_content_addressed_attribute, parse_link_attribute,
    parse_subscribe_attribute,
};
use crate::utils::errors;
use syn::{Field, Ident, Path, Result, Type};

/// Information about a field's key type
#[derive(Debug, Clone)]
pub enum FieldKeyType {
    Primary,
    Secondary,
    Relational { definition: Path, model: Path },
    Blob,
    Regular,
}

/// Information collected about a model field
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: Ident,
    pub ty: Type,
    pub key_type: FieldKeyType,
}

/// Information collected about subscription topics on a model
#[derive(Debug, Clone)]
pub struct SubscriptionInfo {
    pub topics: Vec<Path>,
    pub immutable: bool,
}

/// Version information collected from #[netabase_version] attribute.
#[derive(Debug, Clone)]
pub struct ModelVersionInfo {
    /// The model family name (groups versions together).
    pub family: String,
    /// The version number.
    pub version: u32,
    /// Whether this is explicitly marked as the current version.
    pub is_current: Option<bool>,
    /// Whether this version supports downgrade (implements MigrateTo).
    pub supports_downgrade: bool,
}

impl From<VersionAttributeConfig> for ModelVersionInfo {
    fn from(config: VersionAttributeConfig) -> Self {
        Self {
            family: config.family,
            version: config.version,
            is_current: config.is_current,
            supports_downgrade: config.supports_downgrade,
        }
    }
}

/// Visitor that collects information about model fields
#[derive(Debug, Clone)]
pub struct ModelFieldVisitor {
    pub model_name: Ident,
    pub primary_key: Option<FieldInfo>,
    pub secondary_keys: Vec<FieldInfo>,
    pub relational_keys: Vec<FieldInfo>,
    pub blob_fields: Vec<FieldInfo>,
    pub regular_fields: Vec<FieldInfo>,
    pub subscriptions: Option<SubscriptionInfo>,
    /// Version information if this model is versioned.
    pub version_info: Option<ModelVersionInfo>,
    /// Whether this model supports libp2p features
    pub is_libp2p_enabled: bool,
    /// Configuration for content-addressed models (implicit primary key)
    pub content_addressed_config: Option<ContentAddressedAttributeConfig>,
    /// Private fields for checking immutability
    pub all_fields_raw: Vec<Field>,
}

impl ModelFieldVisitor {
    pub fn new(model_name: Ident) -> Self {
        Self {
            model_name,
            primary_key: None,
            secondary_keys: Vec::new(),
            relational_keys: Vec::new(),
            blob_fields: Vec::new(),
            regular_fields: Vec::new(),
            subscriptions: None,
            version_info: None,
            is_libp2p_enabled: false,
            content_addressed_config: None,
            all_fields_raw: Vec::new(),
        }
    }

    /// Visit a field and collect its information
    pub fn visit_field(&mut self, field: &Field) -> Result<()> {
        self.all_fields_raw.push(field.clone());

        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "Tuple structs are not supported"))?;

        let has_primary = has_attribute(&field.attrs, "primary_key");
        let has_secondary = has_attribute(&field.attrs, "secondary_key");
        let has_link = has_attribute(&field.attrs, "link");
        let has_blob = has_attribute(&field.attrs, "blob");

        // Validate that only one key attribute is present
        let attr_count = [has_primary, has_secondary, has_link, has_blob]
            .iter()
            .filter(|&&x| x)
            .count();

        if attr_count > 1 {
            return Err(errors::duplicate_field_attribute(
                field.ident.as_ref().unwrap().span(),
                "multiple key attributes on single field",
            ));
        }

        if has_primary {
            self.visit_primary_key(field_name, &field.ty)?
        } else if has_secondary {
            self.visit_secondary_key(field_name, &field.ty)?
        } else if has_link {
            self.visit_relational_key(field, field_name, &field.ty)?
        } else if has_blob {
            self.visit_blob_field(field_name, &field.ty)?
        } else {
            self.visit_regular_field(field_name, &field.ty)?
        };

        Ok(())
    }

    fn visit_primary_key(&mut self, name: &Ident, ty: &Type) -> Result<()> {
        if self.primary_key.is_some() {
            return Err(errors::multiple_primary_keys(name.span()));
        }

        self.primary_key = Some(FieldInfo {
            name: name.clone(),
            ty: ty.clone(),
            key_type: FieldKeyType::Primary,
        });

        Ok(())
    }

    fn visit_secondary_key(&mut self, name: &Ident, ty: &Type) -> Result<()> {
        self.secondary_keys.push(FieldInfo {
            name: name.clone(),
            ty: ty.clone(),
            key_type: FieldKeyType::Secondary,
        });

        Ok(())
    }

    fn visit_relational_key(&mut self, field: &Field, name: &Ident, ty: &Type) -> Result<()> {
        let link_attr = find_attribute(&field.attrs, "link")
            .ok_or_else(|| syn::Error::new_spanned(field, "Expected link attribute"))?;

        let (definition, model) = parse_link_attribute(link_attr)?;

        self.relational_keys.push(FieldInfo {
            name: name.clone(),
            ty: ty.clone(),
            key_type: FieldKeyType::Relational { definition, model },
        });

        Ok(())
    }

    fn visit_blob_field(&mut self, name: &Ident, ty: &Type) -> Result<()> {
        self.blob_fields.push(FieldInfo {
            name: name.clone(),
            ty: ty.clone(),
            key_type: FieldKeyType::Blob,
        });

        Ok(())
    }

    fn visit_regular_field(&mut self, name: &Ident, ty: &Type) -> Result<()> {
        self.regular_fields.push(FieldInfo {
            name: name.clone(),
            ty: ty.clone(),
            key_type: FieldKeyType::Regular,
        });

        Ok(())
    }

    /// Parse subscribe attribute on the model struct itself
    pub fn visit_model_attributes(&mut self, attrs: &[syn::Attribute]) -> Result<()> {
        if let Some(subscribe_attr) = find_attribute(attrs, "subscribe") {
            let config = parse_subscribe_attribute(subscribe_attr)?;
            self.subscriptions = Some(SubscriptionInfo {
                topics: config.topics,
                immutable: config.immutable,
            });
        }

        // Parse version attribute if present
        if let Some(version_config) = get_version_info(attrs)? {
            self.version_info = Some(ModelVersionInfo::from(version_config));
        }

        // Check for libp2p support
        if has_attribute(attrs, "netabase_libp2p") {
            self.is_libp2p_enabled = true;
        }

        // Check for content_addressed support
        if let Some(attr) = find_attribute(attrs, "netabase_content_addressed") {
            self.content_addressed_config = Some(parse_content_addressed_attribute(attr)?);
        }

        Ok(())
    }

    /// Validate that the visitor collected valid information
    pub fn validate(&self) -> Result<()> {
        // Must have exactly one primary key, unless content addressed
        if self.primary_key.is_none() && self.content_addressed_config.is_none() {
            return Err(errors::no_primary_key(self.model_name.span()));
        }

        // Validate immutability if requested
        if let Some(subs) = &self.subscriptions {
            if subs.immutable {
                for field in &self.all_fields_raw {
                    if let syn::Visibility::Public(_) = field.vis {
                        return Err(syn::Error::new_spanned(
                            field,
                            "immutable models must have private fields (remove 'pub')",
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Get all fields that need to be part of the struct
    pub fn all_fields(&self) -> Vec<&FieldInfo> {
        let mut fields = Vec::new();

        if let Some(ref pk) = self.primary_key {
            fields.push(pk);
        }

        fields.extend(self.secondary_keys.iter());
        fields.extend(self.relational_keys.iter());
        fields.extend(self.blob_fields.iter());
        fields.extend(self.regular_fields.iter());

        fields
    }
}
