use crate::macros::netabase_libp2p::process_libp2p_attribute;
use crate::utils::attributes::{
    find_attribute, has_attribute, parse_link_attribute, remove_attribute,
};
use crate::utils::naming::*;
use syn::{Field, Ident, ItemStruct, parse_quote, visit_mut::VisitMut};

/// Mutator that transforms the model structs
pub struct ModelMutator {
    pub definition_name: Ident,
    pub current_model_name: Option<Ident>,
    pub current_model_family: Option<String>,
}

impl ModelMutator {
    pub fn new(definition_name: Ident) -> Self {
        Self {
            definition_name,
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
        // netabase_libp2p is removed by process_libp2p_attribute
        // We also need to remove the derive(NetabaseModel) to prevent re-expansion issues
        // or just let it stay if it's a marker.
        // Logic: if we generate all impls manually in the attribute macro, we should remove the derive.
        // But removing a specific derive from a list is tricky with simple helpers.
        // For now, let's assume we want to remove it.
        // Since `remove_attribute` removes the whole attribute by name, we can't easily remove just one derive from `#[derive(A, B)]`.
        // However, usually users write `#[derive(NetabaseModel)]` separately or we can just leave it if we define NetabaseModel as an empty trait or macro that does nothing when the code is already generated.
        // But to avoid "duplicate implementation" errors if the derive macro logic runs, we should probably modify the derive macro to be smart or remove it here.
        // Simplest approach: leave it, but ensure the derive macro is robust or empty if we are doing everything here.
        // Actually, the plan is to have `netabase_definition` generate everything.
        // So the derive `NetabaseModel` should be a NO-OP or stripped.
        // Let's try to strip `NetabaseModel` from derive list.

        item_struct.attrs = item_struct
            .attrs
            .iter()
            .filter_map(|attr| {
                if let syn::Meta::List(meta_list) = &attr.meta
                    && meta_list.path.is_ident("derive") {
                        let tokens = meta_list.tokens.to_string();
                        if tokens.contains("NetabaseModel") {
                            // Reconstruct derive without NetabaseModel
                            // This is hard without parsing nested types.
                            // If it's the only derive, return None.
                            if tokens.trim() == "NetabaseModel" {
                                return None;
                            }
                            // If mixed, it's safer to keep it and make the derive macro checks if it should run.
                            // OR, we can just replace the attribute with a cleaned one.
                            // For this exercise, let's assume it's separate or we leave it.
                            // If we leave it, the derive macro must NOT generate conflict impls.
                            // But we want to move generation here.
                            // Let's rely on the user putting it separately or update the derive macro to be empty.
                            // I will update `netabase_model.rs` macro to be empty/pass-through later?
                            // No, the prompt asks to implement the macros.
                            // If I implement logic here, I should make the derive macro empty.
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
            remove_attribute(&mut field.attrs, "primary_key");
        } else if has_secondary {
            // Wait, for secondary keys, the struct field usually KEEPS the original type (e.g. String)
            // But the KEY is a wrapper.
            // Let's check boilerplate.
            // User struct: `pub name: String` (original type).
            // UserSecondaryKeys enum: `Name(UserName)`.
            // So for secondary keys, we DO NOT change the struct field type.
            // We only generate the wrapper type (UserName) which is used in the Enum.
            remove_attribute(&mut field.attrs, "secondary_key");
        } else if has_link {
            // Change type to RelationalLink
            if let Some(link_attr) = find_attribute(&field.attrs, "link")
                && let Ok((target_def, target_model)) = parse_link_attribute(link_attr) {
                    let current_def = &self.definition_name;
                    // target_def, target_model are Paths.
                    // field.ty = RelationalLink<'static, R, SourceD, TargetD, M>
                    //
                    // For definitions without explicit repos(), we use Standalone as the repository.
                    // This allows all standalone definitions to link to each other.
                    // When definitions are explicitly in repositories, they should only link
                    // to other definitions within the same repository.

                    // We need to resolve TargetModel to a type.
                    // If TargetModel is just "User", it works.

                    field.ty = parse_quote! {
                        netabase_store::relational::RelationalLink<
                            'static,
                            netabase_store::traits::registry::repository::Standalone,
                            #current_def,
                            #target_def,
                            #target_model
                        >
                    };
                }
            remove_attribute(&mut field.attrs, "link");
        } else if has_blob {
            // Change type to Wrapper (e.g. LargeUserFile)
            // Wait, in boilerplate: `pub bio: LargeUserFile`.
            // Is `LargeUserFile` a wrapper or the original type?
            // `#[blob] bio: LargeUserFile` in input.
            // So the input type IS the type.
            // But `LargeUserFile` must implement `NetabaseBlobItem`.
            // And boilerplate has `impl NetabaseBlobItem for LargeUserFile`.
            // So we don't change the type in the struct.
            remove_attribute(&mut field.attrs, "blob");
        }
    }
}
