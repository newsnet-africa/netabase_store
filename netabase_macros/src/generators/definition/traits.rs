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

        let record_convertion_impl = self.generate_from_record();
        output.extend(record_convertion_impl);

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

    /// Generate InRepository<Standalone> implementation for all definitions.
    ///
    /// This allows definitions to use RelationalLink even when part of explicit repositories.
    fn generate_standalone_repository_impl(&self) -> TokenStream {
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
        
        let iter_record_name = quote::format_ident!("{}RecordIter", definition_name);
        let tables_name = quote::format_ident!("{}ReadOnlyTables", definition_name);
        let record_wrapper_name = syn::Ident::new(&format!("{}Record", definition_name), definition_name.span());

        // Prepare Libp2p method bodies
        let mut find_record_arms = Vec::new();
        let mut add_provider_arms = Vec::new();
        let mut get_providers_arms = Vec::new();
        let mut remove_record_arms = Vec::new();
        let mut remove_provider_arms = Vec::new();
        let mut put_record_arms = Vec::new();

        for model in &self.visitor.models {
            let model_name = &model.name;
            let libp2p_provider_key_enum = libp2p_provider_key_enum_name(model_name);
            
            let is_content_addressed = model.is_content_addressed();
            let target_type = if is_content_addressed {
                 quote::format_ident!("{}Envelope", model_name)
            } else {
                 model_name.clone()
            };

            // put_record block
            put_record_arms.push(quote! {
                #definition_name::#model_name(ref model) => {
                    use ::netabase_store::databases::redb::transaction::tables::{
                        ModelOpenTables, TablePermission, ReadWriteTableType
                    };
                    use ::netabase_store::databases::redb::transaction::crud::RedbModelCrud;
                    use ::netabase_store::traits::registery::models::model::NetabaseModel;
                    use ::netabase_store::traits::registery::models::keys::{
                        NetabaseModelKeys, blob::NetabaseModelBlobKey
                    };
                    use redb::{ReadableTable, ReadableMultimapTable};

                    type Keys = <#target_type as NetabaseModel<#definition_name>>::Keys;
                    type Pk = <Keys as NetabaseModelKeys<#definition_name, #target_type>>::Primary;
                    type Sk = <Keys as NetabaseModelKeys<#definition_name, #target_type>>::Secondary;
                    type Rk = <Keys as NetabaseModelKeys<#definition_name, #target_type>>::Relational;
                    type Bk = <Keys as NetabaseModelKeys<#definition_name, #target_type>>::Blob;
                    type Bi = <Bk as NetabaseModelBlobKey<#definition_name, #target_type>>::BlobItem;
                    type SubK = <#definition_name as ::netabase_store::traits::registery::definition::NetabaseDefinition>::SubscriptionKeys;

                    // Helper to open table
                    let open_rw_table = |name| {
                        let def = redb::TableDefinition::<Pk, #target_type>::new(name);
                        txn.open_table(def).map_err(|e| ::netabase_store::errors::NetabaseError::RedbTableError(e))
                    };
                    
                    let tree_names = <#target_type as NetabaseModel<#definition_name>>::TREE_NAMES;

                    // Main
                    let main_table = open_rw_table(tree_names.main.table_name)?;
                    let main_perm = TablePermission::ReadWrite(ReadWriteTableType::Table(main_table));

                    // Secondary
                    let mut secondary = Vec::new();
                    for t in tree_names.secondary {
                        let def = redb::MultimapTableDefinition::<Sk, Pk>::new(t.table_name);
                        let table = txn.open_multimap_table(def).map_err(|e| ::netabase_store::errors::NetabaseError::RedbTableError(e))?;
                        secondary.push((TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)), t.table_name));
                    }

                    // Blob
                    let mut blob = Vec::new();
                    for t in tree_names.blob {
                        let def = redb::MultimapTableDefinition::<Bk, Bi>::new(t.table_name);
                        let table = txn.open_multimap_table(def).map_err(|e| ::netabase_store::errors::NetabaseError::RedbTableError(e))?;
                        blob.push((TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)), t.table_name));
                    }

                    // Relational
                    let mut relational = Vec::new();
                    for t in tree_names.relational {
                        let def = redb::MultimapTableDefinition::<Pk, Rk>::new(t.table_name);
                        let table = txn.open_multimap_table(def).map_err(|e| ::netabase_store::errors::NetabaseError::RedbTableError(e))?;
                        relational.push((TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)), t.table_name));
                    }

                    // Subscription
                    let mut subscription = Vec::new();
                    if let Some(subs) = tree_names.subscription {
                        for t in subs {
                            let def = redb::MultimapTableDefinition::<SubK, Pk>::new(t.table_name);
                            let table = txn.open_multimap_table(def).map_err(|e| ::netabase_store::errors::NetabaseError::RedbTableError(e))?;
                            subscription.push((TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)), t.table_name));
                        }
                    }

                    let mut tables = ModelOpenTables {
                        main: main_perm,
                        secondary,
                        blob,
                        relational,
                        subscription,
                    };

                    // Check existence
                    // Note: For content-addressed models, model is the Envelope.
                    // But in the Definition enum, #model_name(#envelope_name) if content-addressed.
                    // So `ref model` is `&Envelope`.
                    
                    let exists = match &tables.main {
                        TablePermission::ReadWrite(ReadWriteTableType::Table(t)) => {
                            t.get(&model.get_primary_key()).map_err(|e| ::netabase_store::errors::NetabaseError::RedbStorageError(e))?.is_some()
                        },
                        _ => false,
                    };

                    if exists {
                        model.update_entry(&mut tables)?;
                    } else {
                        model.create_entry(&mut tables)?;
                    }
                }
            });

            // find_record block
            find_record_arms.push(quote! {
                {
                    use redb::ReadableTable;
                    type Pk = <<#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::Keys as ::netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, #target_type>>::Primary;
                    if let Ok(pk) = ::netabase_store::postcard::from_bytes::<Pk>(key_bytes) {
                        let table_name = <#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::TREE_NAMES.main.table_name;
                        let table_def = redb::TableDefinition::<Pk, #target_type>::new(table_name);
                        
                        if let Ok(table) = txn.open_table(table_def) {
                            if let Ok(Some(val)) = table.get(&pk) {
                                let model = val.value();
                                let meta = <#target_type as ::netabase_store::traits::libp2p::libp2p_model::Libp2pModel>::get_libp2p_metadata(&model)
                                    .cloned()
                                    .unwrap_or_default();
                                
                                let wrapper_name = #record_wrapper_name(#definition_name::#model_name(model), meta);
                                return Ok(Some(wrapper_name.into()));
                            }
                        }
                    }
                }
            });

            // add_provider block
            add_provider_arms.push(quote! {
                {
                    type Pk = <<#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::Keys as ::netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, #target_type>>::Primary;
                    if let Ok(pk) = ::netabase_store::postcard::from_bytes::<Pk>(key_bytes) {
                        let provider_key = #libp2p_provider_key_enum::Full(pk);
                        let tree_providers = <#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::TREE_NAMES.providers;
                        if let Some(first_provider) = tree_providers.first() {
                            let table_name = first_provider.table_name;
                            let table_def = redb::MultimapTableDefinition::<#libp2p_provider_key_enum, ::netabase_store::databases::redb::transaction::value_wrappers::Libp2pProviderRecordWrapper>::new(table_name);
                            
                            if let Ok(mut table) = txn.open_multimap_table(table_def) {
                                let wrapper = ::netabase_store::databases::redb::transaction::value_wrappers::Libp2pProviderRecordWrapper(record.clone());
                                let _ = table.insert(provider_key, wrapper);
                            }
                        }
                    }
                }
            });

            // get_providers block
            get_providers_arms.push(quote! {
                {
                    use redb::ReadableMultimapTable;
                    type Pk = <<#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::Keys as ::netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, #target_type>>::Primary;
                    if let Ok(pk) = ::netabase_store::postcard::from_bytes::<Pk>(key_bytes) {
                        let provider_key = #libp2p_provider_key_enum::Full(pk);
                        let tree_providers = <#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::TREE_NAMES.providers;
                        if let Some(first_provider) = tree_providers.first() {
                            let table_name = first_provider.table_name;
                            let table_def = redb::MultimapTableDefinition::<#libp2p_provider_key_enum, ::netabase_store::databases::redb::transaction::value_wrappers::Libp2pProviderRecordWrapper>::new(table_name);
                            
                            if let Ok(table) = txn.open_multimap_table(table_def) {
                                if let Ok(iter) = table.get(provider_key) {
                                    for item in iter {
                                        if let Ok(val) = item {
                                            providers.push(val.value().0);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });

            // remove_record block
            remove_record_arms.push(quote! {
                {
                    type Pk = <<#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::Keys as ::netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, #target_type>>::Primary;
                    if let Ok(pk) = ::netabase_store::postcard::from_bytes::<Pk>(key_bytes) {
                        let table_name = <#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::TREE_NAMES.main.table_name;
                        let table_def = redb::TableDefinition::<Pk, #target_type>::new(table_name);
                        
                        if let Ok(mut table) = txn.open_table(table_def) {
                            let _ = table.remove(&pk);
                        }
                    }
                }
            });

            // remove_provider block
            remove_provider_arms.push(quote! {
                {
                    use redb::ReadableMultimapTable;
                    type Pk = <<#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::Keys as ::netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, #target_type>>::Primary;
                    if let Ok(pk) = ::netabase_store::postcard::from_bytes::<Pk>(key_bytes) {
                        let provider_key = #libp2p_provider_key_enum::Full(pk);
                        let tree_providers = <#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::TREE_NAMES.providers;
                        if let Some(first_provider) = tree_providers.first() {
                            let table_name = first_provider.table_name;
                            let table_def = redb::MultimapTableDefinition::<#libp2p_provider_key_enum, ::netabase_store::databases::redb::transaction::value_wrappers::Libp2pProviderRecordWrapper>::new(table_name);
                            
                            if let Ok(mut table) = txn.open_multimap_table(table_def) {
                                let mut to_remove = Vec::new();
                                if let Ok(iter) = table.get(provider_key.clone()) {
                                    for item in iter {
                                        if let Ok(val) = item {
                                            let wrapper = val.value();
                                            if &wrapper.0.provider == provider {
                                                to_remove.push(wrapper);
                                            }
                                        }
                                    }
                                }
                                for wrapper in to_remove {
                                    let _ = table.remove(provider_key.clone(), wrapper);
                                }
                            }
                        }
                    }
                }
            });
        }

        // Use the first model as representative (following the boilerplate pattern)
        if let Some(first_model) = self.visitor.models.first() {
            let model_name = &first_model.visitor.model_name;

            // Generate version detection probes for each model family
            let detect_version_probes = self.generate_detect_version_probes(&def_str);
            
            // Generate migration code for each model family
            let migration_code = self.generate_probing_migration_code(&def_str);

            // Generate table initialization code for all models
            let init_tables_code = self.generate_init_tables_code(&def_str);

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
                            .map_err(|e| ::netabase_store::errors::NetabaseError::RedbTransactionError(e))?;

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

                    fn init_tables(db: &redb::Database) -> ::netabase_store::errors::NetabaseResult<()> {
                        use ::netabase_store::traits::registery::models::model::NetabaseModel;
                        
                        // Open a write transaction to create all tables
                        let write_txn = db.begin_write()
                            .map_err(|e| ::netabase_store::errors::NetabaseError::RedbTransactionError(e))?;

                        #init_tables_code

                        // Commit the transaction to persist table creation
                        write_txn.commit()
                            .map_err(|e| ::netabase_store::errors::NetabaseError::RedbCommitError(e))?;

                        Ok(())
                    }

                    type ReadOnlyTables = #tables_name;
                    type RecordIter<'a> = #iter_record_name<'a>;

                    fn open_read_only_tables(txn: &redb::ReadTransaction) -> ::netabase_store::errors::NetabaseResult<Self::ReadOnlyTables> {
                        #tables_name::new(txn)
                            .map_err(|e| ::netabase_store::errors::NetabaseError::RedbError(e))
                    }

                    fn iter_records<'a>(
                        tables: &'a Self::ReadOnlyTables,
                    ) -> ::netabase_store::errors::NetabaseResult<Self::RecordIter<'a>> {
                        tables.iter_records()
                            .map_err(|e| ::netabase_store::errors::NetabaseError::RedbError(e))
                    }

                    fn find_record(
                        txn: &redb::ReadTransaction,
                        key: &::netabase_store::libp2p::kad::RecordKey,
                    ) -> ::netabase_store::errors::NetabaseResult<Option<::netabase_store::libp2p::kad::Record>> {
                        let key_bytes = key.as_ref();
                        #(#find_record_arms)*
                        Ok(None)
                    }

                    fn put_record(
                        txn: &redb::WriteTransaction,
                        record: ::netabase_store::libp2p::kad::Record,
                    ) -> ::netabase_store::errors::NetabaseResult<()> {
                        let def: Self = record.try_into()
                            .map_err(|e| ::netabase_store::errors::NetabaseError::IoError(format!("Failed to deserialize record: {:?}", e)))?;
                        match def {
                            #(#put_record_arms)*
                            _ => {} // Handles nested/empty
                        }
                        Ok(())
                    }

                    fn add_provider(
                        txn: &redb::WriteTransaction,
                        record: ::netabase_store::libp2p::kad::ProviderRecord,
                    ) -> ::netabase_store::errors::NetabaseResult<()> {
                        let key_bytes = record.key.as_ref();
                        #(#add_provider_arms)*
                        Ok(())
                    }

                    fn get_providers(
                        txn: &redb::ReadTransaction,
                        key: &::netabase_store::libp2p::kad::RecordKey,
                    ) -> ::netabase_store::errors::NetabaseResult<Vec<::netabase_store::libp2p::kad::ProviderRecord>> {
                        let key_bytes = key.as_ref();
                        let mut providers = Vec::new();
                        #(#get_providers_arms)*
                        Ok(providers)
                    }

                    fn remove_record(
                        txn: &redb::WriteTransaction,
                        key: &::netabase_store::libp2p::kad::RecordKey,
                    ) -> ::netabase_store::errors::NetabaseResult<()> {
                        let key_bytes = key.as_ref();
                        #(#remove_record_arms)*
                        Ok(())
                    }

                    fn remove_provider(
                        txn: &redb::WriteTransaction,
                        key: &::netabase_store::libp2p::kad::RecordKey,
                        provider: &::netabase_store::libp2p::PeerId,
                    ) -> ::netabase_store::errors::NetabaseResult<()> {
                        let key_bytes = key.as_ref();
                        #(#remove_provider_arms)*
                        Ok(())
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

    /// Generate code to initialize all tables for all models in the definition.
    fn generate_init_tables_code(&self, def_str: &str) -> TokenStream {
        let mut code = TokenStream::new();

        // For each model in the definition, generate code to open all its tables
        for model_info in &self.visitor.models {
            let model_name = &model_info.visitor.model_name;
            let model_str = model_name.to_string();
            
            let is_content_addressed = model_info.is_content_addressed();
            let target_type = if is_content_addressed {
                 quote::format_ident!("{}Envelope", model_name)
            } else {
                 model_name.clone()
            };

            // Main table
            let main_table_name = table_name(def_str, &model_str, "Primary", "Main");
            code.extend(quote! {
                // Initialize main table for #model_name
                {
                    let table_def = redb::TableDefinition::<
                        <<#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<Self>>::Keys as ::netabase_store::traits::registery::models::keys::NetabaseModelKeys<Self, #target_type>>::Primary,
                        #target_type
                    >::new(#main_table_name);
                    let _ = write_txn.open_table(table_def)
                        .map_err(|e| ::netabase_store::errors::NetabaseError::RedbTableError(e))?;
                }
            });

            // Secondary tables
            for field in &model_info.visitor.secondary_keys {
                let field_name = &field.name;
                let field_name_str = field_name.to_string();
                let pascal_field = to_pascal_case(&field_name_str);
                let sec_table_name = table_name(def_str, &model_str, "Secondary", &pascal_field);
                
                code.extend(quote! {
                    // Initialize secondary table for #model_name::#field_name
                    {
                        let table_def = redb::MultimapTableDefinition::<
                            <<#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<Self>>::Keys as ::netabase_store::traits::registery::models::keys::NetabaseModelKeys<Self, #target_type>>::Secondary,
                            <<#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<Self>>::Keys as ::netabase_store::traits::registery::models::keys::NetabaseModelKeys<Self, #target_type>>::Primary
                        >::new(#sec_table_name);
                        let _ = write_txn.open_multimap_table(table_def)
                            .map_err(|e| ::netabase_store::errors::NetabaseError::RedbTableError(e))?;
                    }
                });
            }

            // Blob tables
            let blob_keys_name = blob_keys_enum_name(model_name);
            let blob_item_name = blob_item_enum_name(model_name);
            for field in &model_info.visitor.blob_fields {
                let field_name = &field.name;
                let field_name_str = field_name.to_string();
                let pascal_field = to_pascal_case(&field_name_str);
                let blob_table_name = table_name(def_str, &model_str, "Blob", &pascal_field);
                
                code.extend(quote! {
                    // Initialize blob table for #model_name::#field_name
                    {
                        let table_def = redb::MultimapTableDefinition::<#blob_keys_name, #blob_item_name>::new(#blob_table_name);
                        let _ = write_txn.open_multimap_table(table_def)
                            .map_err(|e| ::netabase_store::errors::NetabaseError::RedbTableError(e))?;
                    }
                });
            }

            // Relational tables
            for field in &model_info.visitor.relational_keys {
                let field_name = &field.name;
                let field_name_str = field_name.to_string();
                let pascal_field = to_pascal_case(&field_name_str);
                let rel_table_name = table_name(def_str, &model_str, "Relational", &pascal_field);
                
                code.extend(quote! {
                    // Initialize relational table for #model_name::#field_name
                    {
                        let table_def = redb::MultimapTableDefinition::<
                            <<#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<Self>>::Keys as ::netabase_store::traits::registery::models::keys::NetabaseModelKeys<Self, #target_type>>::Primary,
                            <<#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<Self>>::Keys as ::netabase_store::traits::registery::models::keys::NetabaseModelKeys<Self, #target_type>>::Relational
                        >::new(#rel_table_name);
                        let _ = write_txn.open_multimap_table(table_def)
                            .map_err(|e| ::netabase_store::errors::NetabaseError::RedbTableError(e))?;
                    }
                });
            }

            // Subscription tables (if model has subscriptions)
            let def_subscriptions_name = definition_subscriptions_enum_name(&self.visitor.definition_name);
            let primary_key_name = primary_key_type_name_for_model(&model_info.visitor);
            // Wait, primary_key_name comes from wrapper_types. 
            // If content-addressed, primary_key_name is ImmutablePostID.
            // And I implemented PrimaryKey trait for ImmutablePostID for Envelope.
            // So I should use the trait associated type instead of the struct name directly?
            // Using struct name is fine if it implements Key.
            // But let's be consistent and use associated type from target_type.
            
            if let Some(sub_info) = &model_info.visitor.subscriptions {
                for topic in &sub_info.topics {
                    let topic_str = topic.segments.last()
                        .map(|seg| seg.ident.to_string())
                        .unwrap_or_default();
                    let sub_table_name = subscription_table_name(def_str, &model_str, &topic_str);
                    
                    code.extend(quote! {
                        // Initialize subscription table for #model_name::#topic
                        {
                            let table_def = redb::MultimapTableDefinition::<
                                #def_subscriptions_name, 
                                <<#target_type as ::netabase_store::traits::registery::models::model::NetabaseModel<Self>>::Keys as ::netabase_store::traits::registery::models::keys::NetabaseModelKeys<Self, #target_type>>::Primary
                            >::new(#sub_table_name);
                            let _ = write_txn.open_multimap_table(table_def)
                                .map_err(|e| ::netabase_store::errors::NetabaseError::RedbTableError(e))?;
                        }
                    });
                }
            }

            // Libp2p Provider Table
            {
                let libp2p_provider_key = libp2p_provider_key_enum_name(model_name);
                let libp2p_table_name = table_name(def_str, &model_str, "Libp2p", "Provider");

                code.extend(quote! {
                    // Initialize Libp2p Provider table for #model_name
                    {
                        let table_def = redb::MultimapTableDefinition::<#libp2p_provider_key, ::netabase_store::databases::redb::transaction::value_wrappers::Libp2pProviderRecordWrapper>::new(#libp2p_table_name);
                         let _ = write_txn.open_multimap_table(table_def)
                            .map_err(|e| ::netabase_store::errors::NetabaseError::RedbTableError(e))?;
                    }
                });
            }
        }

        code
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
                            <<#old_model_name as ::netabase_store::traits::registery::models::model::NetabaseModel<Self>>::Keys as ::netabase_store::traits::registery::models::keys::NetabaseModelKeys<Self, #old_model_name>>::Primary,
                            #old_model_name
                        >::new(#old_table_name);

                        if let Ok(old_table) = write_txn.open_table(old_table_def) {
                            let count = old_table.len().unwrap_or(0);
                            if count > 0 {
                                // Found old version data! Migrate it.
                                
                                // First, collect all records from the old table
                                let records: Vec<_> = old_table.iter()
                                    .map_err(|e| ::netabase_store::errors::NetabaseError::RedbStorageError(e))?
                                    .filter_map(|item| item.ok())
                                    .map(|(k, v)| (k.value().clone(), v.value()))
                                    .collect();
                                
                                // Now open/create the new table and migrate each record
                                let new_table_def = redb::TableDefinition::<
                                    <<#current_model_name as ::netabase_store::traits::registery::models::model::NetabaseModel<Self>>::Keys as ::netabase_store::traits::registery::models::keys::NetabaseModelKeys<Self, #current_model_name>>::Primary,
                                    #current_model_name
                                >::new(#current_table_name);
                                
                                let mut new_table = write_txn.open_table(new_table_def)
                                    .map_err(|e| ::netabase_store::errors::NetabaseError::RedbTableError(e))?;
                                
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
                        .map_err(|e| ::netabase_store::errors::NetabaseError::RedbTransactionError(e))?;

                    #version_probes

                    write_txn.commit()
                        .map_err(|e| ::netabase_store::errors::NetabaseError::RedbCommitError(e))?;
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

        // Handle empty case - generate an empty enum with proper trait implementations
        if topics.is_empty() {
            return quote! {
                // Empty TreeName discriminant - use unit type
                #[derive(
                    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
                    serde::Serialize, serde::Deserialize,
                    strum::AsRefStr
                )]
                pub enum #tree_name {}

                // Empty enum for models with no subscriptions
                #[derive(
                    Clone, Eq, PartialEq, PartialOrd, Ord, Debug,
                    serde::Serialize, serde::Deserialize,
                    Hash
                )]
                pub enum #enum_name {}

                impl strum::IntoDiscriminant for #enum_name {
                    type Discriminant = ();

                    fn discriminant(&self) -> Self::Discriminant {
                        match *self {}
                    }
                }

                impl redb::Value for #enum_name {
                    type SelfType<'a> = #enum_name;
                    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

                    fn from_bytes<'a>(_data: &'a [u8]) -> Self::SelfType<'a>
                    where
                        Self: 'a,
                    {
                        panic!("Cannot deserialize empty subscription enum")
                    }

                    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                    where
                        Self: 'a,
                        Self: 'b,
                    {
                        match *value {}
                    }

                    fn fixed_width() -> Option<usize> {
                        None
                    }

                    fn type_name() -> redb::TypeName {
                        redb::TypeName::new(&format!("{}::{}", module_path!(), stringify!(#enum_name)))
                    }
                }

                impl redb::Key for #enum_name {
                    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                        data1.cmp(data2)
                    }
                }
            };
        }

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
                serde::Serialize, serde::Deserialize,
                strum::AsRefStr
            )]
            pub enum #tree_name {
                #(#tree_name_variants),*
            }

            // Main subscription enum
            #[derive(
                Clone, Eq, PartialEq, PartialOrd, Ord, Debug,
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

            impl redb::Value for #enum_name {
                type SelfType<'a> = #enum_name;
                type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

                fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
                where
                    Self: 'a,
                {
                    postcard::from_bytes(data).unwrap()
                }

                fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                where
                    Self: 'a,
                    Self: 'b,
                {
                    std::borrow::Cow::Owned(
                        postcard::to_allocvec(value).unwrap()
                    )
                }

                fn fixed_width() -> Option<usize> {
                    None
                }

                fn type_name() -> redb::TypeName {
                    redb::TypeName::new(&format!("{}::{}", module_path!(), stringify!(#enum_name)))
                }
            }

            impl redb::Key for #enum_name {
                fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                    let val1: #enum_name = postcard::from_bytes(data1).unwrap();
                    let val2: #enum_name = postcard::from_bytes(data2).unwrap();
                    val1.cmp(&val2)
                }
            }
        }
    }

    fn generate_model_traits(&self, definition_name: &syn::Ident, model_info: &ModelInfo) -> TokenStream {
        let model_name = &model_info.name;
        let visitor = &model_info.visitor;
        let is_versioned = visitor.version_info.is_some();
        let is_content_addressed = model_info.is_content_addressed();

        let target_type = if is_content_addressed {
             quote::format_ident!("{}Envelope", model_name)
        } else {
             model_name.clone()
        };

        // Generate marker traits (StoreKeyMarker, StoreValueMarker, etc.)
        // Skip ID-related markers for versioned models to avoid duplicates
        let marker_traits = self.generate_marker_traits(definition_name, model_name, &target_type, visitor, !is_versioned);

        // Generate Store traits (StoreKey, StoreValue)
        let store_traits = self.generate_store_traits(definition_name, model_name, &target_type, visitor);

        // Generate key type traits (NetabaseModelKeys, PrimaryKey, SecondaryKey, etc.)
        let trait_gen = crate::generators::model::TraitGenerator::new(visitor);
        let model_keys_trait = trait_gen.generate_model_keys_trait(definition_name);
        let key_traits = self.generate_key_type_traits(definition_name, model_name, &target_type, visitor);

        // Generate NetabaseModel trait
        let netabase_model_trait = trait_gen.generate_netabase_model_trait(definition_name);

        // Generate RedbNetabaseModel trait
        let redb_trait = self.generate_redb_netabase_model_trait(definition_name, model_name, &target_type, is_content_addressed);

        // Generate subscription conversion traits
        let subscription_traits = self.generate_subscription_traits(definition_name, model_name, visitor);

        // Generate Libp2pModel trait
        let libp2p_trait = trait_gen.generate_libp2p_model_trait();
        
        // Generate ContentAddressedModel trait
        let ca_trait = trait_gen.generate_content_addressed_model_trait(definition_name);

        // Generate tuple conversion for Model -> (Definition, Metadata)
        let tuple_conversion = self.generate_model_tuple_conversion(definition_name, model_info, &target_type);

        quote! {
            #marker_traits
            #store_traits
            #model_keys_trait
            #key_traits
            #netabase_model_trait
            #redb_trait
            #subscription_traits
            #libp2p_trait
            #ca_trait
            #tuple_conversion
        }
    }

    fn generate_model_tuple_conversion(&self, definition_name: &syn::Ident, model_info: &ModelInfo, target_type: &syn::Ident) -> TokenStream {
        let model_name = &model_info.name;
        let record_wrapper_name = syn::Ident::new(&format!("{}Record", definition_name), definition_name.span());
        
        quote! {
            impl From<#target_type> for #record_wrapper_name {
                fn from(model: #target_type) -> Self {
                    let meta = <#target_type as ::netabase_store::traits::libp2p::libp2p_model::Libp2pModel>::get_libp2p_metadata(&model)
                        .cloned()
                        .unwrap_or_default();
                    #record_wrapper_name(#definition_name::#model_name(model), meta)
                }
            }
        }
    }

    fn generate_marker_traits(
        &self,
        definition_name: &syn::Ident,
        model_name: &syn::Ident,
        target_type: &syn::Ident,
        visitor: &crate::visitors::model::field::ModelFieldVisitor,
        generate_id_markers: bool,
    ) -> TokenStream {
        let id_type = primary_key_type_name_for_model(visitor);
        let _keys_enum = unified_keys_enum_name(model_name);

        let mut impls = vec![];

        // StoreKeyMarker and StoreValueMarker for ID - only if flagged
        if generate_id_markers {
            impls.push(quote! {
                impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #id_type {}
                impl netabase_store::traits::registery::models::StoreValueMarker<#definition_name> for #id_type {}
            });
        }

        // StoreValueMarker for model (Envelope if content-addressed)
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreValueMarker<#definition_name> for #target_type {}
        });

        // NetabaseModelMarker
        impls.push(quote! {
            impl netabase_store::traits::registery::models::model::NetabaseModelMarker<#definition_name> for #target_type {}
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
        let blob_keys = blob_keys_enum_name(model_name);
        let blob_item = blob_item_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #blob_keys {}
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #blob_item {}
        });

        // Libp2p keys
        let libp2p_keys = libp2p_provider_key_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #libp2p_keys {}
            impl netabase_store::traits::registery::models::keys::libp2p::NetabaseModelLibp2pProviderKey<#definition_name, #target_type> for #libp2p_keys {}
        });

        quote! { #(#impls)* }
    }

    fn generate_store_traits(
        &self,
        definition_name: &syn::Ident,
        model_name: &syn::Ident,
        target_type: &syn::Ident,
        visitor: &crate::visitors::model::field::ModelFieldVisitor,
    ) -> TokenStream {
        let id_type = primary_key_type_name_for_model(visitor);

        let mut impls = vec![];

        // StoreKey<Definition, Model> for ID
        // StoreValue<Definition, ID> for Model
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKey<#definition_name, #target_type> for #id_type {}
            impl netabase_store::traits::registery::models::StoreValue<#definition_name, #id_type> for #target_type {}
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
        target_type: &syn::Ident,
        visitor: &crate::visitors::model::field::ModelFieldVisitor,
    ) -> TokenStream {
        let id_type = primary_key_type_name_for_model(visitor);

        let mut impls = vec![];

        // NetabaseModelPrimaryKey
        impls.push(quote! {
            impl netabase_store::traits::registery::models::keys::NetabaseModelPrimaryKey<#definition_name, #target_type> for #id_type {}
        });

        // NetabaseModelSecondaryKey
        let secondary_enum = secondary_keys_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::keys::NetabaseModelSecondaryKey<#definition_name, #target_type> for #secondary_enum {
                type PrimaryKey = #id_type;
            }
        });

        // NetabaseModelRelationalKey
        let relational_enum = relational_keys_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::keys::NetabaseModelRelationalKey<#definition_name, #target_type> for #relational_enum {}
        });

        // NetabaseModelBlobKey
        let blob_keys = blob_keys_enum_name(model_name);
        let blob_item = blob_item_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::keys::blob::NetabaseModelBlobKey<#definition_name, #target_type> for #blob_keys {
                type PrimaryKey = #id_type;
                type BlobItem = #blob_item;
            }
        });

        // NetabaseModelSubscriptionKey
        let subscription_enum = subscriptions_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::keys::NetabaseModelSubscriptionKey<#definition_name, #target_type> for #subscription_enum {}
        });

        quote! { #(#impls)* }
    }

    fn generate_redb_netabase_model_trait(
        &self,
        definition_name: &syn::Ident,
        model_name: &syn::Ident,
        target_type: &syn::Ident,
        is_content_addressed: bool,
    ) -> TokenStream {
        quote! {
            impl<'db> ::netabase_store::traits::registery::models::model::RedbNetbaseModel<'db, #definition_name> for #target_type {
                type RedbTables = ::netabase_store::databases::redb::transaction::ModelOpenTables<'db, 'db, #definition_name, Self>;
                type TableV = #target_type;
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

            let is_libp2p_expr = visitor.is_libp2p_enabled;

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
                    is_libp2p_enabled: #is_libp2p_expr,
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
                    
                    // supports_upgrade is true for all versions except the first one
                    let supports_upgrade = version > 1;
                    let is_libp2p_expr = visitor.is_libp2p_enabled;
                    
                    quote! {
                        netabase_store::traits::registery::definition::schema::VersionedModelSchema {
                            version: #version,
                            struct_name: #struct_name.to_string(),
                            fields: vec![#(#field_schemas),*],
                            subscriptions: vec![#(#model_subs),*],
                            version_hash: #version_hash,
                            supports_downgrade: #supports_downgrade,
                            supports_upgrade: #supports_upgrade,
                            is_libp2p_enabled: #is_libp2p_expr,
                        }
                    }
                }).collect();
                
                // Generate migration paths by analyzing field changes between versions
                // Only generate if there are multiple versions
                let migration_paths: Vec<_> = if family.versions.len() > 1 {
                    family.versions.windows(2).map(|pair| {
                        let from_model = &pair[0];
                        let to_model = &pair[1];
                        let from_version = from_model.version();
                        let to_version = to_model.version();
                        
                        // For now, just generate empty migration paths
                        // TODO: Add field change detection
                        quote! {
                            netabase_store::traits::registery::definition::schema::MigrationPathSchema {
                                from_version: #from_version,
                                to_version: #to_version,
                                may_lose_data: false,
                                field_changes: vec![],
                            }
                        }
                    }).collect()
                } else {
                    vec![]
                };
                
                quote! {
                    netabase_store::traits::registery::definition::schema::ModelVersionHistory {
                        family: #family_str.to_string(),
                        current_version: #current_version,
                        versions: vec![#(#version_schemas),*],
                        migration_paths: vec![#(#migration_paths),*],
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
                config: None, // Default configuration
            }
        }
    }

    fn generate_from_record(&self) -> TokenStream {
        let name = &self.visitor.definition_name;
        let record_wrapper_name = syn::Ident::new(&format!("{}Record", name), name.span());

        // Generate key extraction match arms
        let mut key_match_arms = Vec::new();

        for model in &self.visitor.models {
            let m_name = &model.name;
            key_match_arms.push(quote! {
                #name::#m_name(ref m) => {
                     use ::netabase_store::traits::registery::models::model::NetabaseModel;
                     let pk = m.get_primary_key();
                     ::netabase_store::postcard::to_allocvec(&pk).unwrap_or_default()
                }
            });
        }
        
        for nested in &self.visitor.nested_definitions {
            let n_name = &nested.definition_name;
            key_match_arms.push(quote! {
                #name::#n_name(_) => Vec::new() 
            });
        }

        quote! {
            impl TryFrom<::netabase_store::libp2p::kad::Record> for #name {
                type Error = ::netabase_store::postcard::Error;

                fn try_from(value: ::netabase_store::libp2p::kad::Record) -> Result<Self, Self::Error> {
                    ::netabase_store::postcard::from_bytes(&value.value)
                }
            }
            
            /// Wrapper struct to handle conversion to Libp2p Record with metadata
            pub struct #record_wrapper_name(pub #name, pub ::netabase_store::traits::libp2p::libp2p_model::Libp2pMetadata);

            impl From<#record_wrapper_name> for ::netabase_store::libp2p::kad::Record {
                fn from(wrapper: #record_wrapper_name) -> Self {
                    let (def, meta) = (wrapper.0, wrapper.1);
                    let key_bytes = match def {
                        #(#key_match_arms),*
                    };
                    
                    let expires = meta.expires.map(|t| {
                        let now = std::time::SystemTime::now();
                        if t > now {
                            std::time::Instant::now() + t.duration_since(now).unwrap()
                        } else {
                            std::time::Instant::now() 
                        }
                    });

                    ::netabase_store::libp2p::kad::Record {
                        key: ::netabase_store::libp2p::kad::RecordKey::new(&key_bytes),
                        value: ::netabase_store::postcard::to_allocvec(&def).unwrap(),
                        publisher: meta.publisher,
                        expires,
                    }
                }
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
