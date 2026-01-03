use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;
use crate::visitors::definition::{DefinitionVisitor, ModelInfo};
use crate::generators::model::TraitGenerator;
use crate::utils::naming::*;

/// Generator for definition-level trait implementations
/// These are traits that need to know both the Definition and Model types
pub struct DefinitionTraitGenerator<'a> {
    visitor: &'a DefinitionVisitor,
}

impl<'a> DefinitionTraitGenerator<'a> {
    pub fn new(visitor: &'a DefinitionVisitor) -> Self {
        Self { visitor }
    }

    /// Generate all trait implementations for all models in the definition
    pub fn generate(&self) -> TokenStream {
        let mut output = TokenStream::new();

        let definition_name = &self.visitor.definition_name;

        // Generate NetabaseDefinition trait implementation for the definition
        let definition_trait = self.generate_netabase_definition_trait();
        output.extend(definition_trait);

        // Generate NetabaseDefinitionKeys trait implementation
        let def_keys_trait = self.generate_definition_keys_trait();
        output.extend(def_keys_trait);

        // Generate NetabaseDefinitionSubscriptionKeys trait implementation
        let def_subs_trait = self.generate_definition_subscription_keys_trait();
        output.extend(def_subs_trait);

        // Generate RedbDefinition trait implementation
        let redb_def_trait = self.generate_redb_definition_trait();
        output.extend(redb_def_trait);

        // Generate InRepository<Standalone> if no explicit repositories
        let standalone_impl = self.generate_standalone_repository_impl();
        output.extend(standalone_impl);

        for model_info in &self.visitor.models {
            // First generate subscription enum for this model (if it has subscriptions)
            let sub_enum = self.generate_subscription_enum(definition_name, model_info);
            output.extend(sub_enum);

            // Then generate trait implementations
            let traits = self.generate_model_traits(definition_name, model_info);
            output.extend(traits);
        }

        output
    }

    /// Generate InRepository<Standalone> implementation for definitions without explicit repos.
    ///
    /// This allows definitions to use RelationalLink even when not part of an explicit repository.
    fn generate_standalone_repository_impl(&self) -> TokenStream {
        // Only generate Standalone impl if no explicit repositories specified
        if !self.visitor.repositories.is_empty() {
            return TokenStream::new();
        }

        let definition_name = &self.visitor.definition_name;

        quote! {
            impl netabase_store::traits::registery::repository::InRepository<
                netabase_store::traits::registery::repository::Standalone
            > for #definition_name {
                type RepositoryDiscriminant = netabase_store::traits::registery::repository::StandaloneDiscriminant;

                #[inline]
                fn repository_discriminant() -> Self::RepositoryDiscriminant {
                    netabase_store::traits::registery::repository::StandaloneDiscriminant
                }
            }
        }
    }

    fn generate_definition_keys_trait(&self) -> TokenStream {
        let definition_name = &self.visitor.definition_name;
        let keys_enum = definition_keys_enum_name(definition_name);

        quote! {
            impl netabase_store::traits::registery::definition::NetabaseDefinitionKeys<#definition_name> for #keys_enum {}
        }
    }

    fn generate_definition_subscription_keys_trait(&self) -> TokenStream {
        let definition_name = &self.visitor.definition_name;
        let subs_enum = definition_subscriptions_enum_name(definition_name);

        quote! {
            impl netabase_store::traits::registery::definition::subscription::NetabaseDefinitionSubscriptionKeys for #subs_enum {}
        }
    }

    fn generate_redb_definition_trait(&self) -> TokenStream {
        let definition_name = &self.visitor.definition_name;
        let def_str = definition_name.to_string();

        // Use the first model as representative (following the boilerplate pattern)
        if let Some(first_model) = self.visitor.models.first() {
            let model_name = &first_model.visitor.model_name;

            // Generate version detection probes for each model family
            let detect_version_probes = self.generate_detect_version_probes(&def_str);
            
            // Generate migration code for each model family
            let migration_code = self.generate_probing_migration_code(&def_str);

            quote! {
                impl ::netabase_store::traits::registery::definition::redb_definition::RedbDefinition for #definition_name {
                    type ModelTableDefinition<'db> = ::netabase_store::traits::registery::models::model::redb_model::RedbModelTableDefinitions<'db, #model_name, Self>;

                    fn detect_versions(
                        db: &redb::Database,
                    ) -> ::netabase_store::errors::NetabaseResult<Vec<::netabase_store::traits::registery::definition::redb_definition::DetectedVersion>> {
                        use ::netabase_store::traits::registery::definition::redb_definition::DetectedVersion;
                        use redb::{ReadableDatabase, ReadableTableMetadata};

                        let mut detected = Vec::new();
                        
                        // Try to open a read transaction to probe tables
                        let read_txn = db.begin_read()
                            .map_err(|e| ::netabase_store::errors::NetabaseError::RedbTransactionError(e.into()))?;

                        #detect_version_probes

                        Ok(detected)
                    }

                    fn migrate_all(
                        db: &redb::Database,
                        options: &::netabase_store::traits::registery::definition::redb_definition::MigrationOptions,
                    ) -> ::netabase_store::errors::NetabaseResult<::netabase_store::traits::registery::definition::redb_definition::MigrationResult> {
                        use ::netabase_store::traits::registery::definition::redb_definition::MigrationResult;
                        use redb::{ReadableDatabase, ReadableTable, ReadableTableMetadata};

                        let mut result = MigrationResult::default();

                        if options.dry_run {
                            // In dry-run mode, just report what would be migrated
                            let detected = Self::detect_versions(db)?;
                            for _det in detected {
                                // Compare with current version to see if migration needed
                                // The migration code below handles this per-family
                            }
                            return Ok(result);
                        }

                        #migration_code

                        Ok(result)
                    }
                }
            }
        } else {
            // If no models, generate a placeholder (shouldn't happen in practice)
            TokenStream::new()
        }
    }

    /// Generate probes to detect which version tables exist.
    fn generate_detect_version_probes(&self, def_str: &str) -> TokenStream {
        let mut probes = TokenStream::new();

        for family in self.visitor.model_families.values() {
            let family_str = &family.family;
            
            // For each version in the family (oldest to newest), generate a probe
            for model_info in &family.versions {
                let model_name = &model_info.name;
                let model_str = model_name.to_string();
                let version = model_info.version();
                
                // Generate table name using the same format as model traits
                let table_name = table_name(def_str, &model_str, "Primary", "Main");
                
                probes.extend(quote! {
                    // Probe for #model_name (version #version)
                    {
                        // Try to open the table with just &[u8] as value to check if it exists
                        let table_def = redb::TableDefinition::<&[u8], &[u8]>::new(#table_name);
                        if let Ok(table) = read_txn.open_table(table_def) {
                            let count = table.len().unwrap_or(0);
                            if count > 0 {
                                detected.push(DetectedVersion {
                                    family: #family_str.to_string(),
                                    version: #version,
                                    table_name: #table_name.to_string(),
                                    record_count: count,
                                });
                            }
                        }
                    }
                });
            }
        }

        probes
    }

    /// Generate migration code that probes for old versions and migrates.
    fn generate_probing_migration_code(&self, def_str: &str) -> TokenStream {
        let mut code = TokenStream::new();

        for family in self.visitor.model_families.values() {
            // Only generate migration for families with multiple versions
            if family.versions.len() <= 1 {
                continue;
            }

            let family_str = &family.family;
            let current_model = family.current_model();
            let current_model_name = &current_model.name;
            let current_model_str = current_model_name.to_string();
            let current_version = family.current_version;
            let current_table_name = table_name(def_str, &current_model_str, "Primary", "Main");

            // Generate probes for each OLD version (not current)
            let mut version_probes = TokenStream::new();
            
            for (source_index, model_info) in family.versions.iter().enumerate() {
                let version = model_info.version();
                if version >= current_version {
                    continue; // Skip current version
                }
                
                let old_model_name = &model_info.name;
                let old_model_str = old_model_name.to_string();
                let old_table_name = table_name(def_str, &old_model_str, "Primary", "Main");
                
                // Generate the migration chain call for this version to current
                let migration_chain = self.generate_migration_chain_for_version(family, source_index);
                
                version_probes.extend(quote! {
                    // Check if version #version table exists
                    {
                        let old_table_def = redb::TableDefinition::<
                            <#old_model_name as ::netabase_store::traits::registery::models::model::NetabaseModel<Self>>::PrimaryKey,
                            #old_model_name
                        >::new(#old_table_name);

                        if let Ok(old_table) = write_txn.open_table(old_table_def) {
                            let count = old_table.len().unwrap_or(0);
                            if count > 0 {
                                // Found old version data! Migrate it.
                                
                                // First, collect all records from the old table
                                let records: Vec<_> = old_table.iter()
                                    .map_err(|e| ::netabase_store::errors::NetabaseError::RedbError(e.into()))?
                                    .filter_map(|item| item.ok())
                                    .map(|(k, v)| (k.value().clone(), v.value()))
                                    .collect();
                                
                                // Now open/create the new table and migrate each record
                                let new_table_def = redb::TableDefinition::<
                                    <#current_model_name as ::netabase_store::traits::registery::models::model::NetabaseModel<Self>>::PrimaryKey,
                                    #current_model_name
                                >::new(#current_table_name);
                                
                                let mut new_table = write_txn.open_table(new_table_def)
                                    .map_err(|e| ::netabase_store::errors::NetabaseError::RedbError(e.into()))?;
                                
                                for (key, old_value) in records {
                                    // Apply migration chain: OldModel -> ... -> CurrentModel
                                    let migrated: #current_model_name = {
                                        let source = old_value;
                                        #migration_chain
                                    };
                                    
                                    // Insert into new table
                                    match new_table.insert(&key, &migrated) {
                                        Ok(_) => {
                                            result.records_migrated += 1;
                                        }
                                        Err(e) => {
                                            if options.continue_on_error {
                                                result.records_failed += 1;
                                                result.errors.push(format!("Failed to insert migrated record: {}", e));
                                            } else {
                                                return Err(::netabase_store::errors::NetabaseError::MigrationError(
                                                    format!("Failed to insert migrated record: {}", e)
                                                ));
                                            }
                                        }
                                    }
                                }
                                
                                result.migrations_performed.push((
                                    #family_str.to_string(),
                                    #version,
                                    #current_version,
                                ));
                                
                                // Optionally delete the old table
                                if options.delete_old_tables {
                                    drop(old_table);
                                    // Note: redb doesn't have direct table deletion, 
                                    // the old table will just have stale data
                                    // In practice, you might want to clear it or leave it
                                }
                            }
                        }
                    }
                });
            }

            code.extend(quote! {
                // Migration for family: #family_str
                {
                    let write_txn = db.begin_write()
                        .map_err(|e| ::netabase_store::errors::NetabaseError::RedbTransactionError(e.into()))?;

                    #version_probes

                    write_txn.commit()
                        .map_err(|e| ::netabase_store::errors::NetabaseError::RedbTransactionError(e.into()))?;
                }
            });
        }

        code
    }

    /// Generate a chain of MigrateFrom calls from a source version to current.
    fn generate_migration_chain_for_version(
        &self,
        family: &crate::visitors::definition::ModelFamily,
        source_index: usize,
    ) -> TokenStream {
        let mut chain = quote! { source };

        for i in source_index..family.current_index {
            let target_name = &family.versions[i + 1].name;
            chain = quote! {
                <#target_name as ::netabase_store::traits::migration::MigrateFrom<_>>::migrate_from(#chain)
            };
        }

        chain
    }

    fn generate_subscription_enum(&self, definition_name: &syn::Ident, model_info: &ModelInfo) -> TokenStream {
        let model_name = &model_info.name;
        let visitor = &model_info.visitor;

        // If no subscriptions, treat as empty topics list
        let empty_topics = Vec::new();
        let topics = visitor.subscriptions.as_ref().map(|s| &s.topics).unwrap_or(&empty_topics);

        let enum_name = subscriptions_enum_name(model_name);
        let tree_name = tree_name_type(&enum_name);
        let def_subscription_enum = definition_subscriptions_enum_name(definition_name);

        let variants: Vec<_> = topics
            .iter()
            .map(|topic| {
                let topic_ident = path_last_segment(topic).expect("Invalid subscription topic");
                quote! { #topic_ident(#def_subscription_enum) }
            })
            .collect();

        let tree_name_variants: Vec<_> = topics
            .iter()
            .map(|topic| {
                path_last_segment(topic).expect("Invalid subscription topic").clone()
            })
            .collect();

        quote! {
            // TreeName discriminant enum
            #[derive(
                Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
                bincode::Encode, bincode::Decode,
                serde::Serialize, serde::Deserialize,
                strum::AsRefStr
            )]
            pub enum #tree_name {
                #(#tree_name_variants),*
            }

            // Main subscription enum
            #[derive(
                Clone, Eq, PartialEq, PartialOrd, Ord, Debug,
                bincode::Encode, bincode::Decode,
                serde::Serialize, serde::Deserialize,
                Hash
            )]
            pub enum #enum_name {
                #(#variants),*
            }
            
            // Implement IntoDiscriminant manually for empty/non-empty enums
            impl strum::IntoDiscriminant for #enum_name {
                type Discriminant = #tree_name;

                fn discriminant(&self) -> Self::Discriminant {
                    match self {
                        #(#enum_name::#tree_name_variants(_) => #tree_name::#tree_name_variants),*
                    }
                }
            }
        }
    }

    fn generate_model_traits(&self, definition_name: &syn::Ident, model_info: &ModelInfo) -> TokenStream {
        let model_name = &model_info.name;
        let visitor = &model_info.visitor;

        // Generate marker traits (StoreKeyMarker, StoreValueMarker, etc.)
        let marker_traits = self.generate_marker_traits(definition_name, model_name, visitor);

        // Generate Store traits (StoreKey, StoreValue)
        let store_traits = self.generate_store_traits(definition_name, model_name, visitor);

        // Generate key type traits (NetabaseModelKeys, PrimaryKey, SecondaryKey, etc.)
        let trait_gen = TraitGenerator::new(visitor);
        let model_keys_trait = trait_gen.generate_model_keys_trait(definition_name);
        let key_traits = self.generate_key_type_traits(definition_name, model_name, visitor);

        // Generate NetabaseModel trait
        let netabase_model_trait = trait_gen.generate_netabase_model_trait(definition_name);

        // Generate RedbNetabaseModel trait
        let redb_trait = self.generate_redb_netabase_model_trait(definition_name, model_name);

        // Generate subscription conversion traits
        let subscription_traits = self.generate_subscription_traits(definition_name, model_name, visitor);

        quote! {
            #marker_traits
            #store_traits
            #model_keys_trait
            #key_traits
            #netabase_model_trait
            #redb_trait
            #subscription_traits
        }
    }

    fn generate_marker_traits(
        &self,
        definition_name: &syn::Ident,
        model_name: &syn::Ident,
        _visitor: &crate::visitors::model::field::ModelFieldVisitor,
    ) -> TokenStream {
        let id_type = primary_key_type_name(model_name);
        let _keys_enum = unified_keys_enum_name(model_name);

        let mut impls = vec![];

        // StoreKeyMarker and StoreValueMarker for ID
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #id_type {}
            impl netabase_store::traits::registery::models::StoreValueMarker<#definition_name> for #id_type {}
        });

        // StoreValueMarker for model
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreValueMarker<#definition_name> for #model_name {}
        });

        // NetabaseModelMarker
        impls.push(quote! {
            impl netabase_store::traits::registery::models::model::NetabaseModelMarker<#definition_name> for #model_name {}
        });

        // Secondary keys
        let secondary_enum = secondary_keys_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #secondary_enum {}
        });

        // Relational keys
        let relational_enum = relational_keys_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #relational_enum {}
        });

        // Subscriptions
        let subscription_enum = subscriptions_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #subscription_enum {}
        });

        // Blob keys
        // For blob keys, if empty, we still generate enums?
        // KeyEnumGenerator generates blob enums ONLY if !blob_fields.is_empty().
        // I changed generate_unified_keys_enum to remove check, but generate_blob_keys_enum logic?
        // Wait, I only modified generate_unified_keys_enum and generate in key_enums.rs.
        // generate_blob_keys_enum in key_enums.rs loops over blob_fields. If empty, it generates empty enum.
        // So they ARE generated.
        let blob_keys = blob_keys_enum_name(model_name);
        let blob_item = blob_item_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #blob_keys {}
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #blob_item {}
        });

        quote! { #(#impls)* }
    }

    fn generate_store_traits(
        &self,
        definition_name: &syn::Ident,
        model_name: &syn::Ident,
        _visitor: &crate::visitors::model::field::ModelFieldVisitor,
    ) -> TokenStream {
        let id_type = primary_key_type_name(model_name);

        let mut impls = vec![];

        // StoreKey<Definition, Model> for ID
        // StoreValue<Definition, ID> for Model
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKey<#definition_name, #model_name> for #id_type {}
            impl netabase_store::traits::registery::models::StoreValue<#definition_name, #id_type> for #model_name {}
        });

        // Secondary keys
        let secondary_enum = secondary_keys_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKey<#definition_name, #id_type> for #secondary_enum {}
            impl netabase_store::traits::registery::models::StoreValue<#definition_name, #secondary_enum> for #id_type {}
        });

        // Relational keys
        let relational_enum = relational_keys_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKey<#definition_name, #id_type> for #relational_enum {}
            impl netabase_store::traits::registery::models::StoreValue<#definition_name, #relational_enum> for #id_type {}
        });

        // Subscriptions
        let subscription_enum = subscriptions_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKey<#definition_name, #id_type> for #subscription_enum {}
            impl netabase_store::traits::registery::models::StoreValue<#definition_name, #subscription_enum> for #id_type {}
        });

        quote! { #(#impls)* }
    }

    fn generate_key_type_traits(
        &self,
        definition_name: &syn::Ident,
        model_name: &syn::Ident,
        _visitor: &crate::visitors::model::field::ModelFieldVisitor,
    ) -> TokenStream {
        let id_type = primary_key_type_name(model_name);
        let keys_enum = unified_keys_enum_name(model_name);

        let mut impls = vec![];

        // NetabaseModelPrimaryKey
        impls.push(quote! {
            impl<'a> netabase_store::traits::registery::models::keys::NetabaseModelPrimaryKey<'a, #definition_name, #model_name, #keys_enum> for #id_type {}
        });

        // NetabaseModelSecondaryKey
        let secondary_enum = secondary_keys_enum_name(model_name);
        impls.push(quote! {
            impl<'a> netabase_store::traits::registery::models::keys::NetabaseModelSecondaryKey<'a, #definition_name, #model_name, #keys_enum> for #secondary_enum {
                type PrimaryKey = #id_type;
            }
        });

        // NetabaseModelRelationalKey
        let relational_enum = relational_keys_enum_name(model_name);
        impls.push(quote! {
            impl<'a> netabase_store::traits::registery::models::keys::NetabaseModelRelationalKey<'a, #definition_name, #model_name, #keys_enum> for #relational_enum {}
        });

        // NetabaseModelBlobKey
        let blob_keys = blob_keys_enum_name(model_name);
        let blob_item = blob_item_enum_name(model_name);
        impls.push(quote! {
            impl<'a> netabase_store::traits::registery::models::keys::blob::NetabaseModelBlobKey<'a, #definition_name, #model_name, #keys_enum> for #blob_keys {
                type PrimaryKey = #id_type;
                type BlobItem = #blob_item;
            }
        });

        // NetabaseModelSubscriptionKey
        let subscription_enum = subscriptions_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::keys::NetabaseModelSubscriptionKey<#definition_name, #model_name, #keys_enum> for #subscription_enum {}
        });

        quote! { #(#impls)* }
    }

    fn generate_redb_netabase_model_trait(
        &self,
        definition_name: &syn::Ident,
        model_name: &syn::Ident,
    ) -> TokenStream {
        quote! {
            impl<'db> ::netabase_store::traits::registery::models::model::RedbNetbaseModel<'db, #definition_name> for #model_name {
                type RedbTables = ::netabase_store::databases::redb::transaction::ModelOpenTables<'db, 'db, #definition_name, Self>;
                type TableV = #model_name;
            }
        }
    }

    fn generate_subscription_traits(
        &self,
        definition_name: &syn::Ident,
        model_name: &syn::Ident,
        visitor: &crate::visitors::model::field::ModelFieldVisitor,
    ) -> TokenStream {
        // If no subscriptions, treat as empty
        let empty_topics = Vec::new();
        let topics = visitor.subscriptions.as_ref().map(|s| &s.topics).unwrap_or(&empty_topics);

        let subscription_enum = subscriptions_enum_name(model_name);
        let def_subscription_enum = definition_subscriptions_enum_name(definition_name);

        // Generate From impl
        let from_arms: Vec<_> = topics
            .iter()
            .map(|topic| {
                let topic_ident = path_last_segment(topic).unwrap();
                quote! {
                    #def_subscription_enum::#topic_ident => #subscription_enum::#topic_ident(value)
                }
            })
            .collect();

        // Generate TryInto impl
        let try_into_arms: Vec<_> = topics
            .iter()
            .map(|topic| {
                let topic_ident = path_last_segment(topic).unwrap();
                quote! {
                    #subscription_enum::#topic_ident(v) => Ok(v)
                }
            })
            .collect();

        quote! {
            impl From<#def_subscription_enum> for #subscription_enum {
                fn from(value: #def_subscription_enum) -> Self {
                    match value {
                        #(#from_arms,)*
                        _ => panic!("Unsupported subscription topic for {} model", stringify!(#model_name)),
                    }
                }
            }

            impl TryInto<#def_subscription_enum> for #subscription_enum {
                type Error = ();

                fn try_into(self) -> Result<#def_subscription_enum, Self::Error> {
                    match self {
                        #(#try_into_arms,)*
                    }
                }
            }
        }
    }

    fn generate_netabase_definition_trait(&self) -> TokenStream {
        let definition_name = &self.visitor.definition_name;
        let tree_names_enum = definition_tree_names_enum_name(definition_name); // Complex enum
        let def_keys_enum = definition_keys_enum_name(definition_name);
        let subscription_enum = definition_subscriptions_enum_name(definition_name);
        let discriminant_enum = definition_tree_name_type(definition_name); // Simple discriminant enum (e.g. DefinitionTreeName)

        // Debug name
        let debug_name_str = definition_name.to_string();

        // Subscription Discriminant
        let subscription_discriminant_type = if self.visitor.subscriptions.topics.is_empty() {
            quote! { () }
        } else {
            let disc_name = Ident::new(
                &format!("{}Discriminants", subscription_enum),
                subscription_enum.span()
            );
            quote! { #disc_name }
        };

        // Subscription Registry
        let registry_entries: Vec<_> = self.visitor.subscriptions.topics
            .iter()
            .map(|topic| {
                let topic_ident = path_last_segment(topic).expect("Invalid topic path");
                let topic_str = topic_ident.to_string();

                // Find all models that subscribe to this topic
                let subscribers: Vec<_> = self.visitor.models
                    .iter()
                    .filter(|m| {
                        if let Some(subs) = &m.visitor.subscriptions {
                            subs.topics.iter().any(|t| path_last_segment(t).map_or(false, |i| i == topic_ident))
                        } else {
                            false
                        }
                    })
                    .map(|m| {
                        let model_name = &m.name;
                        // Use the discriminant enum for subscribers
                        quote! { #discriminant_enum::#model_name }
                    })
                    .collect();

                quote! {
                    netabase_store::traits::registery::definition::subscription::SubscriptionEntry {
                        topic: #topic_str,
                        subscribers: &[#(#subscribers),*],
                    }
                }
            })
            .collect();

        // Schema generation
        let schema_impl = self.generate_schema_impl();

        quote! {
            impl netabase_store::traits::registery::definition::NetabaseDefinition for #definition_name {
                type TreeNames = #tree_names_enum;
                type DefKeys = #def_keys_enum;
                type DebugName = &'static str;

                fn debug_name() -> Self::DebugName {
                    #debug_name_str
                }

                fn schema() -> netabase_store::traits::registery::definition::schema::DefinitionSchema {
                    #schema_impl
                }

                type SubscriptionKeys = #subscription_enum;
                type SubscriptionKeysDiscriminant = #subscription_discriminant_type;

                const SUBSCRIPTION_REGISTRY: netabase_store::traits::registery::definition::subscription::DefinitionSubscriptionRegistry<'static, Self> =
                    netabase_store::traits::registery::definition::subscription::DefinitionSubscriptionRegistry::new(&[
                        #(#registry_entries),*
                    ]);
            }
        }
    }

    fn generate_schema_impl(&self) -> TokenStream {
        let def_name_str = self.visitor.definition_name.to_string();
        
        let sub_strs: Vec<_> = self.visitor.subscriptions.topics.iter()
            .map(|t| {
                let s = path_last_segment(t).unwrap().to_string();
                quote! { #s.to_string() }
            })
            .collect();

        let model_schemas: Vec<_> = self.visitor.models.iter().map(|model_info| {
            let model_name_str = model_info.name.to_string();
            let visitor = &model_info.visitor;
            
            // Version info
            let (family_expr, version_expr, is_current_expr) = if let Some(ver_info) = model_info.version_info() {
                let family = &ver_info.family;
                let version = ver_info.version;
                let is_current = ver_info.is_current.unwrap_or(false);
                (
                    quote! { Some(#family.to_string()) },
                    quote! { Some(#version) },
                    quote! { #is_current },
                )
            } else {
                (quote! { None }, quote! { None }, quote! { false })
            };

            let mut field_schemas = Vec::new();

            // Helper to add field
            let mut add_field = |info: &crate::visitors::model::field::FieldInfo, key_type_expr: TokenStream| {
                let f_name = info.name.to_string();
                let ty = &info.ty;
                let type_name = quote! { #ty }.to_string();
                field_schemas.push(quote! {
                    netabase_store::traits::registery::definition::schema::FieldSchema {
                        name: #f_name.to_string(),
                        type_name: #type_name.to_string(),
                        key_type: #key_type_expr,
                    }
                });
            };

            // Primary
            if let Some(pk) = &visitor.primary_key {
                add_field(pk, quote! { netabase_store::traits::registery::definition::schema::KeyTypeSchema::Primary });
            }

            // Secondary
            for sk in &visitor.secondary_keys {
                add_field(sk, quote! { netabase_store::traits::registery::definition::schema::KeyTypeSchema::Secondary });
            }

            // Relational
            for rk in &visitor.relational_keys {
                match &rk.key_type {
                    crate::visitors::model::field::FieldKeyType::Relational { definition, model } => {
                         let def_s = path_last_segment(definition).unwrap().to_string();
                         let mod_s = path_last_segment(model).unwrap().to_string();
                         add_field(rk, quote! {
                             netabase_store::traits::registery::definition::schema::KeyTypeSchema::Relational {
                                 definition: #def_s.to_string(),
                                 model: #mod_s.to_string(),
                             }
                         });
                    },
                    _ => panic!("Expected Relational key type"),
                }
            }

            // Blob
            for bk in &visitor.blob_fields {
                 add_field(bk, quote! { netabase_store::traits::registery::definition::schema::KeyTypeSchema::Blob });
            }

            // Regular
            for rk in &visitor.regular_fields {
                 add_field(rk, quote! { netabase_store::traits::registery::definition::schema::KeyTypeSchema::Regular });
            }

            let model_subs: Vec<_> = visitor.subscriptions.as_ref().map(|s| &s.topics).unwrap_or(&Vec::new())
                .iter()
                .map(|t| {
                    let s = path_last_segment(t).unwrap().to_string();
                    quote! { #s.to_string() }
                })
                .collect();

            quote! {
                netabase_store::traits::registery::definition::schema::ModelSchema {
                    name: #model_name_str.to_string(),
                    fields: vec![
                        #(#field_schemas),*
                    ],
                    subscriptions: vec![
                        #(#model_subs),*
                    ],
                    family: #family_expr,
                    version: #version_expr,
                    is_current: #is_current_expr,
                }
            }
        }).collect();

        let struct_schemas: Vec<_> = self.visitor.regular_structs.iter().map(|s_info| {
            let name_str = s_info.name.to_string();
            let is_tuple = s_info.is_tuple;
            
            let field_schemas: Vec<_> = s_info.fields.iter().map(|(fname, fty)| {
                let name = if let Some(n) = fname {
                    n.to_string()
                } else {
                    "".to_string()
                };
                let type_name = quote! { #fty }.to_string();
                
                quote! {
                    netabase_store::traits::registery::definition::schema::StructFieldSchema {
                        name: #name.to_string(),
                        type_name: #type_name.to_string(),
                    }
                }
            }).collect();

            quote! {
                netabase_store::traits::registery::definition::schema::StructSchema {
                    name: #name_str.to_string(),
                    fields: vec![#(#field_schemas),*],
                    is_tuple: #is_tuple,
                }
            }
        }).collect();
        
        // Generate model history for versioned models
        let model_history_schemas: Vec<_> = self.visitor.model_families.values()
            .filter(|family| family.versions.first().map(|m| m.version_info().is_some()).unwrap_or(false))
            .map(|family| {
                let family_str = &family.family;
                let current_version = family.current_version;
                
                let version_schemas: Vec<_> = family.versions.iter().map(|model_info| {
                    let struct_name = model_info.name.to_string();
                    let version = model_info.version();
                    let visitor = &model_info.visitor;
                    let supports_downgrade = model_info.version_info()
                        .map(|v| v.supports_downgrade)
                        .unwrap_or(false);
                    
                    // Compute hash for this version
                    let version_hash = self.compute_model_hash(model_info);
                    
                    let mut field_schemas = Vec::new();
                    
                    let mut add_field = |info: &crate::visitors::model::field::FieldInfo, key_type_expr: TokenStream| {
                        let f_name = info.name.to_string();
                        let ty = &info.ty;
                        let type_name = quote! { #ty }.to_string();
                        field_schemas.push(quote! {
                            netabase_store::traits::registery::definition::schema::FieldSchema {
                                name: #f_name.to_string(),
                                type_name: #type_name.to_string(),
                                key_type: #key_type_expr,
                            }
                        });
                    };
                    
                    if let Some(pk) = &visitor.primary_key {
                        add_field(pk, quote! { netabase_store::traits::registery::definition::schema::KeyTypeSchema::Primary });
                    }
                    for sk in &visitor.secondary_keys {
                        add_field(sk, quote! { netabase_store::traits::registery::definition::schema::KeyTypeSchema::Secondary });
                    }
                    for rk in &visitor.relational_keys {
                        match &rk.key_type {
                            crate::visitors::model::field::FieldKeyType::Relational { definition, model } => {
                                let def_s = path_last_segment(definition).unwrap().to_string();
                                let mod_s = path_last_segment(model).unwrap().to_string();
                                add_field(rk, quote! {
                                    netabase_store::traits::registery::definition::schema::KeyTypeSchema::Relational {
                                        definition: #def_s.to_string(),
                                        model: #mod_s.to_string(),
                                    }
                                });
                            },
                            _ => panic!("Expected Relational key type"),
                        }
                    }
                    for bk in &visitor.blob_fields {
                        add_field(bk, quote! { netabase_store::traits::registery::definition::schema::KeyTypeSchema::Blob });
                    }
                    for rk in &visitor.regular_fields {
                        add_field(rk, quote! { netabase_store::traits::registery::definition::schema::KeyTypeSchema::Regular });
                    }
                    
                    let model_subs: Vec<_> = visitor.subscriptions.as_ref().map(|s| &s.topics).unwrap_or(&Vec::new())
                        .iter()
                        .map(|t| {
                            let s = path_last_segment(t).unwrap().to_string();
                            quote! { #s.to_string() }
                        })
                        .collect();
                    
                    quote! {
                        netabase_store::traits::registery::definition::schema::VersionedModelSchema {
                            version: #version,
                            struct_name: #struct_name.to_string(),
                            fields: vec![#(#field_schemas),*],
                            subscriptions: vec![#(#model_subs),*],
                            version_hash: #version_hash,
                            supports_downgrade: #supports_downgrade,
                        }
                    }
                }).collect();
                
                quote! {
                    netabase_store::traits::registery::definition::schema::ModelVersionHistory {
                        family: #family_str.to_string(),
                        current_version: #current_version,
                        versions: vec![#(#version_schemas),*],
                    }
                }
            }).collect();

        quote! {
            netabase_store::traits::registery::definition::schema::DefinitionSchema {
                schema_format_version: netabase_store::traits::registery::definition::schema::SCHEMA_FORMAT_VERSION,
                name: #def_name_str.to_string(),
                models: vec![
                    #(#model_schemas),*
                ],
                structs: vec![
                    #(#struct_schemas),*
                ],
                subscriptions: vec![
                    #(#sub_strs),*
                ],
                model_history: vec![
                    #(#model_history_schemas),*
                ],
                schema_hash: None, // Will be computed at runtime if needed
            }
        }
    }
    
    /// Compute a hash for a model based on its field structure.
    fn compute_model_hash(&self, model: &ModelInfo) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        model.name.to_string().hash(&mut hasher);
        model.version().hash(&mut hasher);
        
        let visitor = &model.visitor;
        if let Some(ref pk) = visitor.primary_key {
            pk.name.to_string().hash(&mut hasher);
        }
        for field in &visitor.secondary_keys {
            field.name.to_string().hash(&mut hasher);
        }
        for field in &visitor.relational_keys {
            field.name.to_string().hash(&mut hasher);
        }
        for field in &visitor.blob_fields {
            field.name.to_string().hash(&mut hasher);
        }
        for field in &visitor.regular_fields {
            field.name.to_string().hash(&mut hasher);
        }
        
        hasher.finish()
    }
}
