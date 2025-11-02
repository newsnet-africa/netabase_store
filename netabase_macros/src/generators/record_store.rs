//! RecordStore trait implementation generator
//!
//! This module generates the libp2p RecordStore implementation for a Definition enum.
//! It uses the StoreOps traits to efficiently route operations and store models directly.
//!
//! # Design
//!
//! The generated RecordStore implementation:
//! 1. Decodes record keys to extract discriminant (determines which tree/model)
//! 2. Decodes record values directly as models (not wrapped in Definition)
//! 3. Routes operations to the correct tree using discriminant matching
//! 4. Uses StoreOps traits for actual storage operations
//! 5. Wraps models in Definition only when returning to Kad network
//!
//! # Key Format
//!
//! Record keys use the format: `<discriminant_bytes>:<primary_key_bytes>`
//! - The discriminant is the Definition enum discriminant (serialized)
//! - The primary key is the model's primary key (serialized)
//!
//! This allows us to route to the correct tree without decoding the value.

use quote::quote;
use syn::Ident;

use crate::item_info::netabase_definitions::ModuleInfo;

/// Generate helper functions needed by RecordStoreExt trait methods
///
/// These are free functions that need to be in scope when the trait methods are called
pub fn generate_helper_functions(
    definition: &Ident,
) -> proc_macro2::TokenStream {
    quote! {
        // Helper function to decode record key format: <discriminant_bytes>:<key_bytes>
        #[cfg(feature = "libp2p")]
        fn decode_record_key<D>(
            key: &::libp2p::kad::RecordKey
        ) -> Option<(<D as ::netabase_deps::strum::IntoDiscriminant>::Discriminant, Vec<u8>)>
        where
            D: ::netabase_deps::strum::IntoDiscriminant,
            <D as ::netabase_deps::strum::IntoDiscriminant>::Discriminant: ::netabase_deps::bincode::Decode<()>,
        {
            let bytes = key.to_vec();
            let separator_pos = bytes.iter().position(|&b| b == b':')?;

            // Decode discriminant
            let disc_bytes = &bytes[..separator_pos];
            let (discriminant, _): (<D as ::netabase_deps::strum::IntoDiscriminant>::Discriminant, _) =
                ::netabase_deps::bincode::decode_from_slice(
                    disc_bytes,
                    ::netabase_deps::bincode::config::standard()
                ).ok()?;

            let key_bytes = bytes[separator_pos + 1..].to_vec();
            Some((discriminant, key_bytes))
        }
    }
}

/// Generate trait method implementations for RecordStoreExt trait
///
/// Returns just the method definitions (not wrapped in an impl block) to be
/// inserted into the trait impl block in module_definition.rs
///
/// Generates separate methods for each store backend (sled, redb, memory, indexeddb)
/// to avoid trait bound issues with generic S parameters.
pub fn generate_trait_methods(
    modules: &[ModuleInfo],
    definition: &Ident,
) -> proc_macro2::TokenStream {
    let instance_put_match_arms = generate_instance_put_match_arms(modules);
    let instance_get_match_arms = generate_instance_get_match_arms(modules, definition);
    let remove_match_arms = generate_remove_match_arms(modules);

    // Generate OpenTree bounds for all model types
    let open_tree_bounds = generate_open_tree_bounds(modules, definition);

    quote! {
        // Sled store methods
        #[cfg(feature = "sled")]
        fn handle_sled_put(&self, store: &::netabase_store::databases::sled_store::SledStore<Self>) -> ::libp2p::kad::store::Result<()>
        where
            #open_tree_bounds
        {
            use ::libp2p::kad::store::Error;
            use ::netabase_store::traits::store_ops::StoreOps;

            // Match on self variant to extract inner model and store it
            #instance_put_match_arms

            Err(Error::MaxRecords)
        }

        #[cfg(feature = "sled")]
        fn handle_sled_get(store: &::netabase_store::databases::sled_store::SledStore<Self>, key: &::libp2p::kad::RecordKey) -> Option<(Self, ::libp2p::kad::Record)>
        where
            #open_tree_bounds
        {
            use ::netabase_store::traits::definition::NetabaseDefinitionTrait;
            use ::netabase_store::traits::store_ops::StoreOps;
            use ::netabase_deps::strum::IntoDiscriminant;

            // Decode key to get discriminant and primary key bytes
            let (discriminant, key_bytes) = decode_record_key::<#definition>(key)?;

            // Match discriminant to route to correct tree and wrap in Definition
            #instance_get_match_arms

            None
        }

        #[cfg(feature = "sled")]
        fn handle_sled_remove(store: &::netabase_store::databases::sled_store::SledStore<Self>, key: &::libp2p::kad::RecordKey)
        where
            #open_tree_bounds
        {
            use ::netabase_store::traits::definition::NetabaseDefinitionTrait;
            use ::netabase_store::traits::store_ops::StoreOps;
            use ::netabase_deps::strum::IntoDiscriminant;

            // Decode key to get discriminant and primary key bytes
            if let Some((discriminant, key_bytes)) = decode_record_key::<#definition>(key) {
                // Match discriminant to route to correct tree
                #remove_match_arms
            }
        }

        // Redb store methods
        #[cfg(feature = "redb")]
        fn handle_redb_put(&self, store: &::netabase_store::databases::redb_store::RedbStore<Self>) -> ::libp2p::kad::store::Result<()>
        where
            #open_tree_bounds
        {
            use ::libp2p::kad::store::Error;
            use ::netabase_store::traits::store_ops::StoreOps;

            // Match on self variant to extract inner model and store it
            #instance_put_match_arms

            Err(Error::MaxRecords)
        }

        #[cfg(feature = "redb")]
        fn handle_redb_get(store: &::netabase_store::databases::redb_store::RedbStore<Self>, key: &::libp2p::kad::RecordKey) -> Option<(Self, ::libp2p::kad::Record)>
        where
            #open_tree_bounds
        {
            use ::netabase_store::traits::definition::NetabaseDefinitionTrait;
            use ::netabase_store::traits::store_ops::StoreOps;
            use ::netabase_deps::strum::IntoDiscriminant;

            // Decode key to get discriminant and primary key bytes
            let (discriminant, key_bytes) = decode_record_key::<#definition>(key)?;

            // Match discriminant to route to correct tree and wrap in Definition
            #instance_get_match_arms

            None
        }

        #[cfg(feature = "redb")]
        fn handle_redb_remove(store: &::netabase_store::databases::redb_store::RedbStore<Self>, key: &::libp2p::kad::RecordKey)
        where
            #open_tree_bounds
        {
            use ::netabase_store::traits::definition::NetabaseDefinitionTrait;
            use ::netabase_store::traits::store_ops::StoreOps;
            use ::netabase_deps::strum::IntoDiscriminant;

            // Decode key to get discriminant and primary key bytes
            if let Some((discriminant, key_bytes)) = decode_record_key::<#definition>(key) {
                // Match discriminant to route to correct tree
                #remove_match_arms
            }
        }

        // Memory store methods
        #[cfg(feature = "memory")]
        fn handle_memory_put(&self, store: &::netabase_store::databases::memory_store::MemoryStore<Self>) -> ::libp2p::kad::store::Result<()>
        where
            #open_tree_bounds
        {
            use ::libp2p::kad::store::Error;
            use ::netabase_store::traits::store_ops::StoreOps;

            // Match on self variant to extract inner model and store it
            #instance_put_match_arms

            Err(Error::MaxRecords)
        }

        #[cfg(feature = "memory")]
        fn handle_memory_get(store: &::netabase_store::databases::memory_store::MemoryStore<Self>, key: &::libp2p::kad::RecordKey) -> Option<(Self, ::libp2p::kad::Record)>
        where
            #open_tree_bounds
        {
            use ::netabase_store::traits::definition::NetabaseDefinitionTrait;
            use ::netabase_store::traits::store_ops::StoreOps;
            use ::netabase_deps::strum::IntoDiscriminant;

            // Decode key to get discriminant and primary key bytes
            let (discriminant, key_bytes) = decode_record_key::<#definition>(key)?;

            // Match discriminant to route to correct tree and wrap in Definition
            #instance_get_match_arms

            None
        }

        #[cfg(feature = "memory")]
        fn handle_memory_remove(store: &::netabase_store::databases::memory_store::MemoryStore<Self>, key: &::libp2p::kad::RecordKey)
        where
            #open_tree_bounds
        {
            use ::netabase_store::traits::definition::NetabaseDefinitionTrait;
            use ::netabase_store::traits::store_ops::StoreOps;
            use ::netabase_deps::strum::IntoDiscriminant;

            // Decode key to get discriminant and primary key bytes
            if let Some((discriminant, key_bytes)) = decode_record_key::<#definition>(key) {
                // Match discriminant to route to correct tree
                #remove_match_arms
            }
        }

        // IndexedDB store methods
        #[cfg(all(feature = "indexeddb", target_arch = "wasm32"))]
        fn handle_indexeddb_put(&self, store: &::netabase_store::databases::indexeddb_store::IndexedDBStore<Self>) -> ::libp2p::kad::store::Result<()>
        where
            #open_tree_bounds
        {
            use ::libp2p::kad::store::Error;
            use ::netabase_store::traits::store_ops::StoreOps;

            // Match on self variant to extract inner model and store it
            #instance_put_match_arms

            Err(Error::MaxRecords)
        }

        #[cfg(all(feature = "indexeddb", target_arch = "wasm32"))]
        fn handle_indexeddb_get(store: &::netabase_store::databases::indexeddb_store::IndexedDBStore<Self>, key: &::libp2p::kad::RecordKey) -> Option<(Self, ::libp2p::kad::Record)>
        where
            #open_tree_bounds
        {
            use ::netabase_store::traits::definition::NetabaseDefinitionTrait;
            use ::netabase_store::traits::store_ops::StoreOps;
            use ::netabase_deps::strum::IntoDiscriminant;

            // Decode key to get discriminant and primary key bytes
            let (discriminant, key_bytes) = decode_record_key::<#definition>(key)?;

            // Match discriminant to route to correct tree and wrap in Definition
            #instance_get_match_arms

            None
        }

        #[cfg(all(feature = "indexeddb", target_arch = "wasm32"))]
        fn handle_indexeddb_remove(store: &::netabase_store::databases::indexeddb_store::IndexedDBStore<Self>, key: &::libp2p::kad::RecordKey)
        where
            #open_tree_bounds
        {
            use ::netabase_store::traits::definition::NetabaseDefinitionTrait;
            use ::netabase_store::traits::store_ops::StoreOps;
            use ::netabase_deps::strum::IntoDiscriminant;

            // Decode key to get discriminant and primary key bytes
            if let Some((discriminant, key_bytes)) = decode_record_key::<#definition>(key) {
                // Match discriminant to route to correct tree
                #remove_match_arms
            }
        }
    }
}

/// Generate OpenTree trait bounds for all model types
///
/// Generates: `Self: Sized` (store-specific bounds are implicit in method signatures)
fn generate_open_tree_bounds(
    _modules: &[ModuleInfo],
    _definition: &Ident,
) -> proc_macro2::TokenStream {
    // We don't need explicit OpenTree bounds in where clauses because:
    // 1. The method parameter type (e.g., &SledStore<Self>) already constrains the store type
    // 2. OpenTree is implemented for all store types with the model types in this definition
    // 3. Adding explicit bounds causes circular dependency issues with the discriminant
    quote! {
        Self: Sized
    }
}

/// Generate RecordStore implementation for a Definition enum
///
/// This generates code that:
/// 1. Routes operations based on discriminant from record key
/// 2. Uses StoreOps traits for actual storage operations
/// 3. Stores models directly (not wrapped in Definition)
/// 4. Wraps models in Definition only when returning to Kad network
pub fn generate_record_store_impl(
    modules: &[ModuleInfo],
    definition: &Ident,
    _definition_key: &Ident,
) -> proc_macro2::TokenStream {
    let records_iter_impl = generate_records_iter_impl(modules, definition);

    // Generate RedbStore-specific implementations
    let redb_impl = generate_redb_record_store_impl(modules, definition);

    quote! {

            /// Generic records iterator for sled stores
            #[cfg(feature = "sled")]
            pub fn record_store_records_sled(
                store: &::netabase_store::databases::sled_store::SledStore<#definition>
            ) -> RecordsIterGenerated<'_, #definition> {
                RecordsIterGenerated::new(store)
            }

            /// Generic add_provider for sled stores
            #[cfg(feature = "sled")]
            pub fn record_store_add_provider_sled(
                store: &mut ::netabase_store::databases::sled_store::SledStore<#definition>,
                record: ::libp2p::kad::ProviderRecord
            ) -> ::libp2p::kad::store::Result<()> {
                store.add_provider_internal(record)
            }

            /// Generic providers for sled stores
            #[cfg(feature = "sled")]
            pub fn record_store_providers_sled(
                store: &::netabase_store::databases::sled_store::SledStore<#definition>,
                key: &::libp2p::kad::RecordKey
            ) -> Vec<::libp2p::kad::ProviderRecord> {
                store.providers_internal(key).unwrap_or_default()
            }

            /// Generic provided for sled stores
            #[cfg(feature = "sled")]
            pub fn record_store_provided_sled(
                store: &::netabase_store::databases::sled_store::SledStore<#definition>
            ) -> ProvidedIterGenerated<'_> {
                ProvidedIterGenerated::new(store)
            }

            /// Generic remove_provider for sled stores
            #[cfg(feature = "sled")]
            pub fn record_store_remove_provider_sled(
                store: &mut ::netabase_store::databases::sled_store::SledStore<#definition>,
                key: &::libp2p::kad::RecordKey,
                provider: &::libp2p::PeerId
            ) {
                store.remove_provider_internal(key, provider)
            }

            /// Generic add_provider for redb stores
            #[cfg(feature = "redb")]
            pub fn record_store_add_provider_redb(
                store: &mut ::netabase_store::databases::redb_store::RedbStore<#definition>,
                record: ::libp2p::kad::ProviderRecord
            ) -> ::libp2p::kad::store::Result<()> {
                store.add_provider_internal(record)
            }

            /// Generic providers for redb stores
            #[cfg(feature = "redb")]
            pub fn record_store_providers_redb(
                store: &::netabase_store::databases::redb_store::RedbStore<#definition>,
                key: &::libp2p::kad::RecordKey
            ) -> Vec<::libp2p::kad::ProviderRecord> {
                store.providers_internal(key).unwrap_or_default()
            }

            /// Generic provided for redb stores
            #[cfg(feature = "redb")]
            pub fn record_store_provided_redb(
                store: &::netabase_store::databases::redb_store::RedbStore<#definition>
            ) -> ProvidedIterRedb<'_> {
                ProvidedIterRedb::new(store)
            }

            /// Generic remove_provider for redb stores
            #[cfg(feature = "redb")]
            pub fn record_store_remove_provider_redb(
                store: &mut ::netabase_store::databases::redb_store::RedbStore<#definition>,
                key: &::libp2p::kad::RecordKey,
                provider: &::libp2p::PeerId
            ) {
                store.remove_provider_internal(key, provider)
            }

            /// Generic records for redb stores
            #[cfg(feature = "redb")]
            pub fn record_store_records_redb(
                store: &::netabase_store::databases::redb_store::RedbStore<#definition>
            ) -> RecordsIterRedb<'_, #definition> {
                RecordsIterRedb::new(store)
            }

        // RecordStore implementations removed - these should be implemented generically in netabase
        // instead of being generated for each concrete Definition type
        // The helper methods above (handle_record_store_put, etc.) support the generic implementations

        // Helper function to encode record key format: <discriminant_bytes>:<key_bytes>
        fn encode_record_key<D>(
            discriminant: <D as ::netabase_deps::strum::IntoDiscriminant>::Discriminant,
            key_bytes: &[u8]
        ) -> ::libp2p::kad::RecordKey
        where
            D: ::netabase_store::traits::definition::NetabaseDefinitionTrait,
        {
            // Encode discriminant as bytes
            let disc_bytes = ::netabase_deps::bincode::encode_to_vec(
                &discriminant,
                ::netabase_deps::bincode::config::standard()
            ).expect("Discriminant encoding should not fail");

            // Combine: <discriminant_bytes>:<key_bytes>
            let mut combined = disc_bytes;
            combined.push(b':');
            combined.extend_from_slice(key_bytes);

            ::libp2p::kad::RecordKey::from(combined)
        }

        // Helper function to decode record key format: <discriminant_bytes>:<key_bytes>
        fn decode_record_key<D>(
            key: &::libp2p::kad::RecordKey
        ) -> Option<(<D as ::netabase_deps::strum::IntoDiscriminant>::Discriminant, Vec<u8>)>
        where
            D: ::netabase_store::traits::definition::NetabaseDefinitionTrait,
        {
            let bytes = key.to_vec();
            let separator_pos = bytes.iter().position(|&b| b == b':')?;

            // Decode discriminant
            let disc_bytes = &bytes[..separator_pos];
            let (discriminant, _): (<D as ::netabase_deps::strum::IntoDiscriminant>::Discriminant, _) =
                ::netabase_deps::bincode::decode_from_slice(
                    disc_bytes,
                    ::netabase_deps::bincode::config::standard()
                ).ok()?;

            let key_bytes = bytes[separator_pos + 1..].to_vec();
            Some((discriminant, key_bytes))
        }

        #records_iter_impl

        // Provider records iterator
        pub struct ProvidedIterGenerated<'a> {
            inner: ::sled::Iter,
            _phantom: std::marker::PhantomData<&'a ()>,
        }

        impl<'a> ProvidedIterGenerated<'a> {
            fn new<D>(store: &'a ::netabase_store::databases::sled_store::SledStore<D>) -> Self
            where
                D: ::netabase_store::traits::definition::NetabaseDefinitionTrait,
            {
                let tree = store.db().open_tree("__libp2p_provided")
                    .expect("Failed to open provided tree");
                ProvidedIterGenerated {
                    inner: tree.iter(),
                    _phantom: std::marker::PhantomData,
                }
            }
        }

        impl<'a> Iterator for ProvidedIterGenerated<'a> {
            type Item = std::borrow::Cow<'a, ::libp2p::kad::ProviderRecord>;

            fn next(&mut self) -> Option<Self::Item> {
                self.inner.next().and_then(|result| {
                    result.ok().and_then(|(_, v)| {
                        use ::netabase_store::databases::record_store::utils::decode_provider;
                        decode_provider(&v).ok().map(std::borrow::Cow::Owned)
                    })
                })
            }
        }

        // RedbStore RecordStore implementation
        #redb_impl
    }
}

/// Generate match arms for instance put operations
/// Matches on self (the Definition enum variant) to extract and store the inner model
fn generate_instance_put_match_arms(modules: &[ModuleInfo]) -> proc_macro2::TokenStream {
    let arms: Vec<_> = modules
        .iter()
        .flat_map(|module| {
            module.models.iter().map(|model| {
                let model_name = &model.ident;
                let model_path = if module.path.is_empty() {
                    quote! { #model_name }
                } else {
                    let path = &module.path;
                    quote! { #path::#model_name }
                };

                quote! {
                    Self::#model_name(model) => {
                        // Open the tree for this model
                        let tree = store.open_tree::<#model_path>();

                        // Use StoreOps::put_raw to store the model directly
                        tree.put_raw(model.clone()).map_err(|_| Error::MaxRecords)?;

                        return Ok(());
                    }
                }
            })
        })
        .collect();

    quote! {
        match self {
            #(#arms)*
        }
    }
}

/// Generate match arms for instance get operations
/// Returns both the Definition and the Record for the Kad network
fn generate_instance_get_match_arms(modules: &[ModuleInfo], definition: &Ident) -> proc_macro2::TokenStream {
    let arms: Vec<_> = modules
        .iter()
        .flat_map(|module| {
            module.models.iter().map(|model| {
                let model_name = &model.ident;
                let model_path = if module.path.is_empty() {
                    quote! { #model_name }
                } else {
                    let path = &module.path;
                    quote! { #path::#model_name }
                };

                quote! {
                    disc if disc.to_string() == stringify!(#model_name) => {
                        // Decode the primary key from key_bytes
                        let (primary_key, _): (<#model_path as ::netabase_store::traits::model::NetabaseModelTrait<#definition>>::PrimaryKey, _) =
                            ::netabase_deps::bincode::decode_from_slice(
                                &key_bytes,
                                ::netabase_deps::bincode::config::standard()
                            ).ok()?;

                        // Open the tree for this model
                        let tree = store.open_tree::<#model_path>();

                        // Use StoreOps::get_raw to fetch the model
                        let model = tree.get_raw(primary_key).ok()??;

                        // Wrap in Definition for Kad network
                        let definition = #definition::#model_name(model);

                        // Encode as Definition for the Record value
                        let value = ::netabase_deps::bincode::encode_to_vec(
                            &definition,
                            ::netabase_deps::bincode::config::standard()
                        ).ok()?;

                        // Return Definition and Record
                        return Some((definition.clone(), ::libp2p::kad::Record {
                            key: key.clone(),
                            value,
                            publisher: None,
                            expires: None,
                        }));
                    }
                }
            })
        })
        .collect();

    quote! {
        match discriminant {
            #(#arms)*
            _ => {}
        }
    }
}

/// Generate match arms for put operations (legacy - being phased out)
///
/// Routes to correct tree based on discriminant and uses StoreOps::put_raw
fn generate_put_match_arms(modules: &[ModuleInfo], definition: &Ident) -> proc_macro2::TokenStream {
    let arms: Vec<_> = modules
        .iter()
        .flat_map(|module| {
            module.models.iter().map(|model| {
                let model_name = &model.ident;
                let model_path = if module.path.is_empty() {
                    quote! { #model_name }
                } else {
                    let path = &module.path;
                    quote! { #path::#model_name }
                };

                quote! {
                    disc if disc.to_string() == stringify!(#model_name) => {
                        // Decode the value directly as the model
                        let (model, _): (#model_path, _) = ::netabase_deps::bincode::decode_from_slice(
                            &record.value,
                            ::netabase_deps::bincode::config::standard()
                        ).map_err(|_| Error::ValueTooLarge)?;

                        // Open the tree for this model
                        let tree = self.open_tree::<#model_path>();

                        // Use StoreOps::put_raw to store the model directly
                        tree.put_raw(model).map_err(|_| Error::MaxRecords)?;

                        return Ok(());
                    }
                }
            })
        })
        .collect();

    quote! {
        match discriminant {
            #(#arms)*
            _ => {}
        }
    }
}

/// Generate match arms for get operations
///
/// Routes to correct tree based on discriminant, fetches model, wraps in Definition
fn generate_get_match_arms(modules: &[ModuleInfo], definition: &Ident) -> proc_macro2::TokenStream {
    let arms: Vec<_> = modules
        .iter()
        .flat_map(|module| {
            module.models.iter().map(|model| {
                let model_name = &model.ident;
                let model_path = if module.path.is_empty() {
                    quote! { #model_name }
                } else {
                    let path = &module.path;
                    quote! { #path::#model_name }
                };

                quote! {
                    disc if disc.to_string() == stringify!(#model_name) => {
                        // Decode the primary key from key_bytes
                        let (primary_key, _): (<#model_path as ::netabase_store::traits::model::NetabaseModelTrait<#definition>>::PrimaryKey, _) =
                            ::netabase_deps::bincode::decode_from_slice(
                                &key_bytes,
                                ::netabase_deps::bincode::config::standard()
                            ).ok()?;

                        // Open the tree for this model
                        let tree = self.open_tree::<#model_path>();

                        // Use StoreOps::get_raw to fetch the model
                        let model = tree.get_raw(primary_key).ok()??;

                        // Wrap in Definition for Kad network
                        let definition = #definition::#model_name(model);

                        // Encode as Definition for the Record value
                        let value = ::netabase_deps::bincode::encode_to_vec(
                            &definition,
                            ::netabase_deps::bincode::config::standard()
                        ).ok()?;

                        // Return as Record
                        return Some(std::borrow::Cow::Owned(::libp2p::kad::Record {
                            key: key.clone(),
                            value,
                            publisher: None,
                            expires: None,
                        }));
                    }
                }
            })
        })
        .collect();

    quote! {
        match discriminant {
            #(#arms)*
            _ => {}
        }
    }
}

/// Generate match arms for remove operations
///
/// Routes to correct tree based on discriminant and uses StoreOps::remove_raw
fn generate_remove_match_arms(modules: &[ModuleInfo]) -> proc_macro2::TokenStream {
    let arms: Vec<_> = modules
        .iter()
        .flat_map(|module| {
            module.models.iter().map(|model| {
                let model_name = &model.ident;
                let model_path = if module.path.is_empty() {
                    quote! { #model_name }
                } else {
                    let path = &module.path;
                    quote! { #path::#model_name }
                };

                quote! {
                    disc if disc.to_string() == stringify!(#model_name) => {
                        // Decode the primary key from key_bytes
                        if let Ok((primary_key, _)) = ::netabase_deps::bincode::decode_from_slice::<
                            <#model_path as ::netabase_store::traits::model::NetabaseModelTrait<_>>::PrimaryKey,
                            _
                        >(&key_bytes, ::netabase_deps::bincode::config::standard()) {
                            // Open the tree for this model
                            let tree = store.open_tree::<#model_path>();

                            // Use StoreOps::remove_raw to delete the model
                            let _ = tree.remove_raw(primary_key);
                        }
                    }
                }
            })
        })
        .collect();

    quote! {
        match discriminant {
            #(#arms)*
            _ => {}
        }
    }
}

/// Generate records iterator implementation
///
/// Iterates over all trees and wraps models in Definition
fn generate_records_iter_impl(modules: &[ModuleInfo], definition: &Ident) -> proc_macro2::TokenStream {
    // Generate match arms for decoding based on discriminant
    let decode_arms: Vec<_> = modules
        .iter()
        .flat_map(|module| {
            module.models.iter().map(|model| {
                let model_name = &model.ident;
                let model_path = if module.path.is_empty() {
                    quote! { #model_name }
                } else {
                    let path = &module.path;
                    quote! { #path::#model_name }
                };

                quote! {
                    disc if disc.to_string() == stringify!(#model_name) => {
                        // Decode the model directly
                        if let Ok((model, _)) = ::netabase_deps::bincode::decode_from_slice::<#model_path, _>(
                            &value_bytes,
                            ::netabase_deps::bincode::config::standard()
                        ) {
                            // Get the primary key to build the record key
                            use ::netabase_store::traits::model::NetabaseModelTrait;
                            let primary_key = model.primary_key();
                            let key_bytes = ::netabase_deps::bincode::encode_to_vec(
                                &primary_key,
                                ::netabase_deps::bincode::config::standard()
                            ).ok()?;

                            // Wrap in Definition
                            let definition = #definition::#model_name(model);

                            // Encode as Definition for the Record value
                            let value = ::netabase_deps::bincode::encode_to_vec(
                                &definition,
                                ::netabase_deps::bincode::config::standard()
                            ).ok()?;

                            // Build record key
                            let record_key = encode_record_key::<#definition>(disc.clone(), &key_bytes);

                            return Some(std::borrow::Cow::Owned(::libp2p::kad::Record {
                                key: record_key,
                                value,
                                publisher: None,
                                expires: None,
                            }));
                        }
                    }
                }
            })
        })
        .collect();

    quote! {
        // Iterator over all records, wrapping models in Definition
        pub struct RecordsIterGenerated<'a, D> {
            discriminants: Vec<<D as ::netabase_deps::strum::IntoDiscriminant>::Discriminant>,
            current_discriminant_index: usize,
            current_tree_iter: Option<::sled::Iter>,
            store: &'a ::netabase_store::databases::sled_store::SledStore<D>,
            _phantom: std::marker::PhantomData<D>,
        }

        impl<'a, D> RecordsIterGenerated<'a, D>
        where
            D: ::netabase_store::traits::definition::NetabaseDefinitionTrait,
        {
            fn new(store: &'a ::netabase_store::databases::sled_store::SledStore<D>) -> Self {
                use ::netabase_deps::strum::IntoEnumIterator;

                let discriminants: Vec<_> = <<D as ::netabase_deps::strum::IntoDiscriminant>::Discriminant as IntoEnumIterator>::iter()
                    .collect();

                RecordsIterGenerated {
                    discriminants,
                    current_discriminant_index: 0,
                    current_tree_iter: None,
                    store,
                    _phantom: std::marker::PhantomData,
                }
            }
        }

        impl<'a> Iterator for RecordsIterGenerated<'a, #definition> {
            type Item = std::borrow::Cow<'a, ::libp2p::kad::Record>;

            fn next(&mut self) -> Option<Self::Item> {
                use ::netabase_deps::strum::IntoDiscriminant;

                loop {
                    // If we don't have a current iterator, try to get the next tree
                    if self.current_tree_iter.is_none() {
                        if self.current_discriminant_index >= self.discriminants.len() {
                            return None;
                        }

                        let disc = &self.discriminants[self.current_discriminant_index];
                        if let Ok(tree) = self.store.db().open_tree(disc.to_string()) {
                            self.current_tree_iter = Some(tree.iter());
                        } else {
                            self.current_discriminant_index += 1;
                            continue;
                        }
                    }

                    // Try to get next item from current iterator
                    if let Some(ref mut iter) = self.current_tree_iter {
                        match iter.next() {
                            Some(Ok((key_bytes, value_bytes))) => {
                                // Get current discriminant
                                let disc = &self.discriminants[self.current_discriminant_index];

                                // Decode and wrap based on discriminant
                                #(#decode_arms)*
                            }
                            Some(Err(_)) => continue,
                            None => {
                                // Move to next tree
                                self.current_tree_iter = None;
                                self.current_discriminant_index += 1;
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Generate RecordStore implementation for RedbStore
///
/// This generates code specifically for RedbStore's transaction-based API
fn generate_redb_record_store_impl(
    modules: &[ModuleInfo],
    definition: &Ident,
) -> proc_macro2::TokenStream {
    let put_match_arms = generate_put_match_arms(modules, definition);
    let get_match_arms = generate_get_match_arms(modules, definition);
    let remove_match_arms = generate_remove_match_arms(modules);
    let redb_records_iter_impl = generate_redb_records_iter_impl(modules, definition);

    quote! {
        // RecordStore implementation removed - should be implemented generically in netabase

        #redb_records_iter_impl
    }
}

/// Generate RedbStore-specific records iterator implementation
///
/// RedbStore requires collecting records into a Vec first due to transaction constraints
fn generate_redb_records_iter_impl(modules: &[ModuleInfo], definition: &Ident) -> proc_macro2::TokenStream {
    // Generate match arms for decoding based on discriminant  
    let decode_arms: Vec<_> = modules
        .iter()
        .flat_map(|module| {
            module.models.iter().map(|model| {
                let model_name = &model.ident;
                let model_path = if module.path.is_empty() {
                    quote! { #model_name }
                } else {
                    let path = &module.path;
                    quote! { #path::#model_name }
                };

                quote! {
                    disc if disc.to_string() == stringify!(#model_name) => {
                        // Decode the model directly
                        if let Ok((model, _)) = ::netabase_deps::bincode::decode_from_slice::<#model_path, _>(
                            &value_bytes,
                            ::netabase_deps::bincode::config::standard()
                        ) {
                            // Get the primary key to build the record key
                            use ::netabase_store::traits::model::NetabaseModelTrait;
                            let primary_key = model.primary_key();
                            let key_bytes = ::netabase_deps::bincode::encode_to_vec(
                                &primary_key,
                                ::netabase_deps::bincode::config::standard()
                            ).ok()?;

                            // Wrap in Definition
                            let definition = #definition::#model_name(model);

                            // Encode as Definition for the Record value
                            let value = ::netabase_deps::bincode::encode_to_vec(
                                &definition,
                                ::netabase_deps::bincode::config::standard()
                            ).ok()?;

                            // Build record key
                            let record_key = encode_record_key::<#definition>(disc.clone(), &key_bytes);

                            records.push(::libp2p::kad::Record {
                                key: record_key,
                                value,
                                publisher: None,
                                expires: None,
                            });
                        }
                    }
                }
            })
        })
        .collect();

    quote! {
        // RedbStore iterator over all records
        pub struct RecordsIterRedb<'a, D> {
            records: std::vec::IntoIter<::libp2p::kad::Record>,
            _phantom: std::marker::PhantomData<&'a D>,
        }

        impl<'a, D> RecordsIterRedb<'a, D>
        where
            D: ::netabase_store::traits::definition::NetabaseDefinitionTrait,
        {
            fn new(store: &'a ::netabase_store::databases::redb_store::RedbStore<D>) -> Self {
                use ::netabase_deps::strum::IntoEnumIterator;
                use ::netabase_deps::strum::IntoDiscriminant;

                let mut records = Vec::new();

                // Iterate through all discriminants
                let discriminants: Vec<_> = <<D as ::netabase_deps::strum::IntoDiscriminant>::Discriminant as IntoEnumIterator>::iter()
                    .collect();

                // Open read transaction
                if let Ok(read_txn) = store.db().begin_read() {
                    for disc in discriminants {
                        // Get table name from discriminant
                        let table_name = disc.to_string();
                        let static_name: &'static str = Box::leak(table_name.into_boxed_str());
                        let table_def = ::redb::TableDefinition::<&[u8], &[u8]>::new(static_name);

                        // Try to open table
                        if let Ok(table) = read_txn.open_table(table_def) {
                            // Iterate through table entries
                            if let Ok(iter) = table.iter() {
                                for item in iter {
                                    if let Ok((k, v)) = item {
                                        let _key_bytes = k.value();
                                        let value_bytes = v.value();

                                        // Decode and wrap based on discriminant
                                        #(#decode_arms)*
                                    }
                                }
                            }
                        }
                    }
                }

                RecordsIterRedb {
                    records: records.into_iter(),
                    _phantom: std::marker::PhantomData,
                }
            }
        }

        impl<'a> Iterator for RecordsIterRedb<'a, #definition> {
            type Item = std::borrow::Cow<'a, ::libp2p::kad::Record>;

            fn next(&mut self) -> Option<Self::Item> {
                self.records.next().map(std::borrow::Cow::Owned)
            }
        }

        // RedbStore provider records iterator
        pub struct ProvidedIterRedb<'a> {
            records: std::vec::IntoIter<::libp2p::kad::ProviderRecord>,
            _phantom: std::marker::PhantomData<&'a ()>,
        }

        impl<'a> ProvidedIterRedb<'a> {
            fn new<D>(store: &'a ::netabase_store::databases::redb_store::RedbStore<D>) -> Self
            where
                D: ::netabase_store::traits::definition::NetabaseDefinitionTrait,
            {
                let mut records = Vec::new();

                // Open read transaction
                if let Ok(read_txn) = store.db().begin_read() {
                    let table_def = ::redb::TableDefinition::<&[u8], &[u8]>::new("__libp2p_provided");

                    // Try to open provided table
                    if let Ok(table) = read_txn.open_table(table_def) {
                        if let Ok(iter) = table.iter() {
                            for item in iter {
                                if let Ok((_, v)) = item {
                                    use ::netabase_store::databases::record_store::utils::decode_provider;
                                    if let Ok(provider) = decode_provider(v.value()) {
                                        records.push(provider);
                                    }
                                }
                            }
                        }
                    }
                }

                ProvidedIterRedb {
                    records: records.into_iter(),
                    _phantom: std::marker::PhantomData,
                }
            }
        }

        impl<'a> Iterator for ProvidedIterRedb<'a> {
            type Item = std::borrow::Cow<'a, ::libp2p::kad::ProviderRecord>;

            fn next(&mut self) -> Option<Self::Item> {
                self.records.next().map(std::borrow::Cow::Owned)
            }
        }
    }
}
