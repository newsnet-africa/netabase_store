use syn::{Field, Ident, Path, Type, Result};
use crate::utils::attributes::{has_attribute, find_attribute, parse_link_attribute, parse_subscribe_attribute};
use crate::utils::errors;

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
        }
    }

    /// Visit a field and collect its information
    pub fn visit_field(&mut self, field: &Field) -> Result<()> {
        let field_name = field.ident.as_ref().ok_or_else(|| {
            syn::Error::new_spanned(field, "Tuple structs are not supported")
        })?;

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
                "multiple key attributes on single field"
            ));
        }

        let field_info = if has_primary {
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
            let topics = parse_subscribe_attribute(subscribe_attr)?;
            self.subscriptions = Some(SubscriptionInfo { topics });
        }

        Ok(())
    }

    /// Validate that the visitor collected valid information
    pub fn validate(&self) -> Result<()> {
        // Must have exactly one primary key
        if self.primary_key.is_none() {
            return Err(errors::no_primary_key(self.model_name.span()));
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
