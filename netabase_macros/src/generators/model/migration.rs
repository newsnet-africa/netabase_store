//! Migration trait generation for versioned models.
//! Updated to fix match arm comma separators.
//! Last modified: 2026-01-03

use crate::visitors::definition::{DefinitionVisitor, ModelFamily, ModelInfo};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Generator for migration-related traits and implementations.
pub struct MigrationGenerator<'a> {
    visitor: &'a DefinitionVisitor,
}

impl<'a> MigrationGenerator<'a> {
    pub fn new(visitor: &'a DefinitionVisitor) -> Self {
        Self { visitor }
    }

    /// Generate all migration-related code for this definition.
    pub fn generate(&self) -> TokenStream {
        let mut output = TokenStream::new();

        // Generate VersionedModel impl for each versioned model
        output.extend(self.generate_versioned_model_impls());

        // Generate CurrentVersion impl for current models
        output.extend(self.generate_current_version_impls());

        // Generate VersionHistory trait impl for each family
        output.extend(self.generate_version_history_impls());

        // Generate MigrationChainExecutor impl for each family
        output.extend(self.generate_migration_chain_impls());

        // Generate VersionedDecode/VersionedEncode impls
        output.extend(self.generate_versioned_codec_impls());

        output
    }

    /// Generate VersionedModel trait impls for all versioned models.
    fn generate_versioned_model_impls(&self) -> TokenStream {
        let mut impls = TokenStream::new();

        for family in self.visitor.model_families.values() {
            for model in &family.versions {
                if model.version_info().is_some() {
                    impls.extend(self.generate_versioned_model_impl(model, family));
                }
            }
        }

        impls
    }

    fn generate_versioned_model_impl(
        &self,
        model: &ModelInfo,
        family: &ModelFamily,
    ) -> TokenStream {
        let model_name = &model.name;
        let family_str = &family.family;
        let version = model.version();
        let is_current = version == family.current_version;

        quote! {
            impl netabase_store::traits::migration::VersionedModel for #model_name {
                const FAMILY: &'static str = #family_str;
                const VERSION: u32 = #version;
                const IS_CURRENT: bool = #is_current;
            }
        }
    }

    /// Generate CurrentVersion trait impls for current models.
    fn generate_current_version_impls(&self) -> TokenStream {
        let mut impls = TokenStream::new();

        for family in self.visitor.model_families.values() {
            let current_model = family.current_model();
            if current_model.version_info().is_some() {
                impls.extend(self.generate_current_version_impl(current_model, family));
            }
        }

        impls
    }

    fn generate_current_version_impl(
        &self,
        model: &ModelInfo,
        family: &ModelFamily,
    ) -> TokenStream {
        let model_name = &model.name;
        let family_str = &family.family;
        let version = family.current_version;

        // Generate a schema hash based on field information
        let hash = self.compute_model_hash(model);

        quote! {
            impl netabase_store::traits::migration::CurrentVersion for #model_name {
                const FAMILY: &'static str = #family_str;
                const VERSION: u32 = #version;

                fn schema_hash() -> u64 {
                    #hash
                }
            }
        }
    }

    /// Generate VersionHistory trait impls.
    fn generate_version_history_impls(&self) -> TokenStream {
        let mut impls = TokenStream::new();

        for family in self.visitor.model_families.values() {
            if family
                .versions
                .first()
                .map(|m| m.version_info().is_some())
                .unwrap_or(false)
            {
                impls.extend(self.generate_version_history_impl(family));
            }
        }

        impls
    }

    fn generate_version_history_impl(&self, family: &ModelFamily) -> TokenStream {
        let current_model = family.current_model();
        let model_name = &current_model.name;
        let family_str = &family.family;
        let current_version = family.current_version;

        let all_versions: Vec<_> = family.all_versions();
        let version_hashes: Vec<u64> = family
            .versions
            .iter()
            .map(|m| self.compute_model_hash(m))
            .collect();

        quote! {
            impl netabase_store::traits::migration::VersionHistory for #model_name {
                const FAMILY: &'static str = #family_str;
                const CURRENT_VERSION: u32 = #current_version;
                const ALL_VERSIONS: &'static [u32] = &[#(#all_versions),*];
                const VERSION_HASHES: &'static [u64] = &[#(#version_hashes),*];
            }
        }
    }

    /// Generate MigrationChainExecutor impls.
    fn generate_migration_chain_impls(&self) -> TokenStream {
        let mut impls = TokenStream::new();

        for family in self.visitor.model_families.values() {
            if family.versions.len() > 1 {
                impls.extend(self.generate_migration_chain_impl(family));
            }
        }

        impls
    }

    fn generate_migration_chain_impl(&self, family: &ModelFamily) -> TokenStream {
        let current_model = family.current_model();
        let model_name = &current_model.name;
        let chain_struct_name = format_ident!("MigrationChain_{}", model_name);
        let family_str = &family.family;
        let all_versions: Vec<_> = family.all_versions();
        let current_version = family.current_version;

        // Generate the migration function that chains From impls
        let migrate_bytes_impl = self.generate_migrate_bytes_impl(family);

        quote! {
            /// Migration chain executor for the model family.
            pub struct #chain_struct_name;

            impl netabase_store::traits::migration::MigrationChainExecutor for #chain_struct_name {
                type Current = #model_name;

                const FAMILY: &'static str = #family_str;
                const VERSIONS: &'static [u32] = &[#(#all_versions),*];

                fn steps_from(source_version: u32) -> Option<Vec<netabase_store::traits::migration::MigrationStep>> {
                    let current = #current_version;
                    if source_version > current {
                        return None;
                    }

                    let mut steps = Vec::new();
                    for v in source_version..current {
                        steps.push(netabase_store::traits::migration::MigrationStep {
                            from_version: v,
                            to_version: v + 1,
                            may_lose_data: false, // TODO: Track this per migration
                        });
                    }
                    Some(steps)
                }

                #migrate_bytes_impl
            }
        }
    }

    fn generate_migrate_bytes_impl(&self, family: &ModelFamily) -> TokenStream {
        let current_model = family.current_model();
        let _model_name = &current_model.name;

        // Generate match arms for each version
        let mut match_arms = Vec::new();

        for (i, model) in family.versions.iter().enumerate() {
            let version = model.version();
            let source_name = &model.name;

            if version == family.current_version {
                // Current version - just decode
                match_arms.push(quote! {
                    #version => {
                        let decoded: #source_name = bincode::decode_from_slice(data, bincode::config::standard())
                            .map_err(|e| netabase_store::traits::migration::MigrationError {
                                record_key: String::new(),
                                error: e.to_string(),
                                at_version: #version,
                            })?
                            .0;
                        Ok(decoded)
                    }
                });
            } else {
                // Need to migrate - build chain of From impls
                let chain = self.generate_migration_chain_call(family, i);
                match_arms.push(quote! {
                    #version => {
                        let decoded: #source_name = bincode::decode_from_slice(data, bincode::config::standard())
                            .map_err(|e| netabase_store::traits::migration::MigrationError {
                                record_key: String::new(),
                                error: e.to_string(),
                                at_version: #version,
                            })?
                            .0;
                        Ok(#chain)
                    }
                });
            }
        }

        quote! {
            fn migrate_bytes(source_version: u32, data: &[u8]) -> Result<Self::Current, netabase_store::traits::migration::MigrationError> {
                match source_version {
                    #(#match_arms),*
                    _ => Err(netabase_store::traits::migration::MigrationError {
                        record_key: String::new(),
                        error: format!("Unknown version: {}", source_version),
                        at_version: source_version,
                    }),
                }
            }
        }
    }

    /// Generate a chain of MigrateFrom calls from a source version to current.
    fn generate_migration_chain_call(
        &self,
        family: &ModelFamily,
        source_index: usize,
    ) -> TokenStream {
        let mut chain = quote! { decoded };

        for i in source_index..family.current_index {
            let target_name = &family.versions[i + 1].name;
            chain = quote! {
                <#target_name as netabase_store::traits::migration::MigrateFrom<_>>::migrate_from(#chain)
            };
        }

        chain
    }

    /// Generate VersionedDecode and VersionedEncode impls.
    fn generate_versioned_codec_impls(&self) -> TokenStream {
        let mut impls = TokenStream::new();

        for family in self.visitor.model_families.values() {
            let current_model = family.current_model();
            if current_model.version_info().is_some() {
                impls.extend(self.generate_versioned_decode_impl(current_model, family));
                impls.extend(self.generate_versioned_encode_impl(current_model, family));
            }
        }

        impls
    }

    fn generate_versioned_decode_impl(
        &self,
        model: &ModelInfo,
        family: &ModelFamily,
    ) -> TokenStream {
        let model_name = &model.name;
        let chain_struct_name = format_ident!("MigrationChain_{}", model_name);
        let current_version = family.current_version;

        quote! {
            impl netabase_store::traits::migration::VersionedDecode for #model_name {
                fn decode_versioned(data: &[u8], ctx: &netabase_store::traits::migration::VersionContext) -> Result<Self, bincode::error::DecodeError> {
                    use netabase_store::traits::migration::VersionHeader;

                    if VersionHeader::is_versioned(data) {
                        let header = VersionHeader::from_bytes(data)
                            .ok_or_else(|| bincode::error::DecodeError::Other("Invalid version header"))?;

                        if header.version == #current_version {
                            // Same version - decode directly
                            let payload = &data[VersionHeader::SIZE..];
                            bincode::decode_from_slice(payload, bincode::config::standard())
                                .map(|(v, _)| v)
                        } else if ctx.auto_migrate {
                            // Different version - need migration
                            let payload = &data[VersionHeader::SIZE..];
                            #chain_struct_name::migrate_bytes(header.version, payload)
                                .map_err(|_e| bincode::error::DecodeError::Other("Migration failed"))
                        } else if ctx.strict {
                            Err(bincode::error::DecodeError::Other("Version mismatch in strict mode"))
                        } else {
                            // Try to decode anyway
                            let payload = &data[VersionHeader::SIZE..];
                            bincode::decode_from_slice(payload, bincode::config::standard())
                                .map(|(v, _)| v)
                        }
                    } else {
                        // Legacy unversioned format
                        Self::decode_unversioned(data)
                    }
                }

                fn decode_unversioned(data: &[u8]) -> Result<Self, bincode::error::DecodeError> {
                    bincode::decode_from_slice(data, bincode::config::standard())
                        .map(|(v, _)| v)
                }
            }
        }
    }

    fn generate_versioned_encode_impl(
        &self,
        model: &ModelInfo,
        family: &ModelFamily,
    ) -> TokenStream {
        let model_name = &model.name;
        let current_version = family.current_version;

        // Generate encode_for_version match arms for each version that supports downgrade
        let mut version_arms = Vec::new();

        for (_i, ver_model) in family.versions.iter().enumerate() {
            let version = ver_model.version();
            let target_name = &ver_model.name;

            if version == family.current_version {
                version_arms.push(quote! {
                    #version => Some(self.encode_versioned())
                });
            } else if ver_model
                .version_info()
                .map(|v| v.supports_downgrade)
                .unwrap_or(false)
            {
                // This version supports downgrade
                version_arms.push(quote! {
                    #version => {
                        let downgraded: #target_name = <Self as netabase_store::traits::migration::MigrateTo<#target_name>>::migrate_to(self);
                        let mut output = netabase_store::traits::migration::VersionHeader::new(#version).to_bytes().to_vec();
                        output.extend(bincode::encode_to_vec(&downgraded, bincode::config::standard()).unwrap());
                        Some(output)
                    }
                });
            }
        }

        quote! {
            impl netabase_store::traits::migration::VersionedEncode for #model_name {
                fn encode_versioned(&self) -> Vec<u8> {
                    use netabase_store::traits::migration::VersionHeader;

                    let mut output = VersionHeader::new(#current_version).to_bytes().to_vec();
                    output.extend(bincode::encode_to_vec(self, bincode::config::standard()).unwrap());
                    output
                }

                fn encode_for_version(&self, target_version: u32) -> Option<Vec<u8>> {
                    match target_version {
                        #(#version_arms),*
                        _ => None,
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

        // Hash field info
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
