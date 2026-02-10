use crate::macros::netabase_libp2p::process_libp2p_attribute;
use crate::utils::attributes::{
    find_attribute, has_attribute, parse_link_attribute, remove_attribute,
};
use crate::utils::naming::*;
use syn::{Field, Ident, ItemStruct, Type, parse_quote, visit_mut::VisitMut};

/// Check if a type is a Vec<T> type
fn is_vec_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Vec";
        }
    }
    false
}

/// Mutator that transforms the model structs
pub struct ModelMutator {
    pub definition_name: Ident,
    pub repository_name: syn::Type,
    pub current_model_name: Option<Ident>,
    pub current_model_family: Option<String>,
}

impl ModelMutator {
    pub fn new(definition_name: Ident, repository_name: syn::Type) -> Self {
        Self {
            definition_name,
            repository_name,
            current_model_name: None,
            current_model_family: None,
        }
    }
}

impl VisitMut for ModelMutator {
    fn visit_item_struct_mut(&mut self, item_struct: &mut ItemStruct) {
        // Check if this struct is a NetabaseModel
        let is_netabase_model = item_struct.attrs.iter().any(|attr| {
            if let syn::Meta::List(meta_list) = &attr.meta
                && meta_list.path.is_ident("derive") {
                    return meta_list.tokens.to_string().contains("NetabaseModel");
                }
            false
        });

        if !is_netabase_model {
            return;
        }

        self.current_model_name = Some(item_struct.ident.clone());
        let _model_name = item_struct.ident.clone();

        // Extract family name from netabase_version attribute if present
        self.current_model_family = None;
        if let Some(version_attr) = find_attribute(&item_struct.attrs, "netabase_version")
            && let Ok(version_config) =
                crate::utils::attributes::parse_version_attribute(version_attr)
            {
                self.current_model_family = Some(version_config.family);
            }

        // Subscriptions are trait-level, not instance-level
        // The #[subscribe(...)] attribute defines which topics the MODEL TYPE subscribes to
        // No field is added to instances - get_subscription_keys() returns static values

        // Process libp2p attribute and inject field if needed
        // The result isn't needed here as ModelMutator modifies in place
        // and we already have visitor info from ModelFieldVisitor
        process_libp2p_attribute(item_struct);

        // Remove netabase attributes from struct
        remove_attribute(&mut item_struct.attrs, "subscribe");
        remove_attribute(&mut item_struct.attrs, "netabase_content_addressed");
        
        item_struct.attrs = item_struct
            .attrs
            .iter()
            .filter_map(|attr| {
                if let syn::Meta::List(meta_list) = &attr.meta
                    && meta_list.path.is_ident("derive") {
                        let tokens = meta_list.tokens.to_string();
                        if tokens.contains("NetabaseModel") {
                            if tokens.trim() == "NetabaseModel" {
                                return None;
                            }
                        }
                    }
                Some(attr.clone())
            })
            .collect();

        // Visit fields
        syn::visit_mut::visit_item_struct_mut(self, item_struct);

        // Subscriptions are now trait-level only - no field injection needed

        self.current_model_name = None;
        self.current_model_family = None;
    }

    fn visit_field_mut(&mut self, field: &mut Field) {
        if self.current_model_name.is_none() {
            return;
        }
        let model_name = self.current_model_name.as_ref().unwrap();

        let has_primary = has_attribute(&field.attrs, "primary_key");
        let has_secondary = has_attribute(&field.attrs, "secondary_key");
        let has_link = has_attribute(&field.attrs, "link");
        let has_blob = has_attribute(&field.attrs, "blob");

        if has_primary {
            // Change type to ModelID (use family name for versioned models)
            let id_type = if let Some(ref family) = self.current_model_family {
                let family_ident = Ident::new(family, model_name.span());
                primary_key_type_name(&family_ident)
            } else {
                primary_key_type_name(model_name)
            };
            field.ty = parse_quote! { #id_type };
            remove_attribute(&mut field.attrs, "primary_key");
        } else if has_secondary {
            remove_attribute(&mut field.attrs, "secondary_key");
        } else if has_link {
            // Change type to RelationalLink or Vec<RelationalLink>
            if let Some(link_attr) = find_attribute(&field.attrs, "link")
                && let Ok((target_def, target_model)) = parse_link_attribute(link_attr) {
                    let current_def = &self.definition_name;
                    let repo = &self.repository_name;
                    
                    // Check if the original field type is Vec<...>
                    let is_vec = is_vec_type(&field.ty);
                    
                    if is_vec {
                        // Transform to Vec<RelationalLink<...>>
                        field.ty = parse_quote! {
                            Vec<netabase_store::relational::RelationalLink<
                                'static,
                                #repo,
                                #current_def,
                                #target_def,
                                #target_model
                            >>
                        };
                    } else {
                        // Single RelationalLink
                        field.ty = parse_quote! {
                            netabase_store::relational::RelationalLink<
                                'static,
                                #repo,
                                #current_def,
                                #target_def,
                                #target_model
                            >
                        };
                    }
                }
            remove_attribute(&mut field.attrs, "link");
        } else if has_blob {
            remove_attribute(&mut field.attrs, "blob");
        }
    }
}
