//! Subscription streams generator for netabase_store
//!
//! This module generates a comprehensive subscription system that follows the same
//! architectural patterns as the main tree API, using associated types and concrete
//! implementations rather than trait objects.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

/// Generate a subscription enum with proper trait implementations
pub fn generate_streams_enum(
    enum_name: &Ident,
    definition_name: &Ident,
    topics: &[String],
) -> TokenStream {
    let variants: Vec<Ident> = topics
        .iter()
        .map(|topic| format_ident!("{}", topic))
        .collect();

    let variant_strings: Vec<String> = topics.iter().cloned().collect();

    quote! {
        /// Generated subscription topics enum
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        #[derive(::bincode::Encode, ::bincode::Decode)]
        #[derive(::strum::EnumIter)]
        pub enum #enum_name {
            #(
                #[doc = #variant_strings]
                #variants,
            )*
        }

        impl #enum_name {
            /// Get the topic name as a string
            pub fn as_str(&self) -> &'static str {
                match self {
                    #(Self::#variants => #variant_strings,)*
                }
            }

            /// Get all topic names
            pub fn all_topics() -> Vec<&'static str> {
                vec![#(#variant_strings,)*]
            }

            /// Get the number of topics
            pub fn count() -> usize {
                [#(stringify!(#variants),)*].len()
            }
        }

        impl std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl std::str::FromStr for #enum_name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#variant_strings => Ok(Self::#variants),)*
                    _ => Err(format!("Unknown subscription topic: {}", s)),
                }
            }
        }

        impl ::netabase_store::traits::subscription::Subscriptions for #definition_name {
            type Subscriptions = #enum_name;

            fn subscriptions() -> <Self::Subscriptions as ::strum::IntoEnumIterator>::Iterator {
                <#enum_name as ::strum::IntoEnumIterator>::iter()
            }

            fn all_subscriptions() -> Vec<Self::Subscriptions> {
                <#enum_name as ::strum::IntoEnumIterator>::iter().collect()
            }

            fn topic_name(topic: Self::Subscriptions) -> String {
                topic.to_string()
            }
        }
    }
}

/// Generate subscription tree implementations for each topic
pub fn generate_subscription_tree_impls(
    enum_name: &Ident,
    definition_name: &Ident,
    topics: &[String],
) -> TokenStream {
    let mut tree_impls = TokenStream::new();

    for topic in topics {
        let variant_name = format_ident!("{}", topic);
        let tree_struct_name = format_ident!("{}SubscriptionTree", topic);
        let tree_doc = format!("Concrete subscription tree for {} topic", topic);

        let tree_impl = quote! {
            #[doc = #tree_doc]
            #[derive(Clone)]
            pub struct #tree_struct_name {
                inner: ::netabase_store::subscription::subscription_tree::MerkleSubscriptionTree<#definition_name>,
            }

            impl #tree_struct_name {
                /// Create a new subscription tree for this topic
                pub fn new() -> Self {
                    Self {
                        inner: ::netabase_store::subscription::subscription_tree::MerkleSubscriptionTree::new(
                            #enum_name::#variant_name
                        ),
                    }
                }

                /// Get the inner merkle subscription tree
                pub fn inner(&self) -> &::netabase_store::subscription::subscription_tree::MerkleSubscriptionTree<#definition_name> {
                    &self.inner
                }

                /// Get the inner merkle subscription tree (mutable)
                pub fn inner_mut(&mut self) -> &mut ::netabase_store::subscription::subscription_tree::MerkleSubscriptionTree<#definition_name> {
                    &mut self.inner
                }
            }

            impl Default for #tree_struct_name {
                fn default() -> Self {
                    Self::new()
                }
            }

            impl ::netabase_store::traits::subscription::SubscriptionTree<#definition_name> for #tree_struct_name {
                type Topic = #enum_name;

                fn topic(&self) -> Self::Topic {
                    #enum_name::#variant_name
                }

                fn put_item(&mut self, key: Vec<u8>, hash: ::netabase_store::traits::subscription::ModelHash) -> Result<(), ::netabase_store::error::NetabaseError> {
                    self.inner.put_item(key, hash)
                }

                fn remove_item(&mut self, key: &[u8]) -> Result<Option<::netabase_store::traits::subscription::ModelHash>, ::netabase_store::error::NetabaseError> {
                    self.inner.remove_item(key)
                }

                fn get_all_hashes(&self) -> Result<Vec<::netabase_store::traits::subscription::ModelHash>, ::netabase_store::error::NetabaseError> {
                    self.inner.get_all_hashes()
                }

                fn merkle_root(&mut self) -> Result<Option<[u8; 32]>, ::netabase_store::error::NetabaseError> {
                    self.inner.merkle_root()
                }

                fn len(&self) -> usize {
                    self.inner.len()
                }

                fn clear(&mut self) -> Result<(), ::netabase_store::error::NetabaseError> {
                    self.inner.clear()
                }

                fn contains_key(&self, key: &[u8]) -> bool {
                    self.inner.contains_key(key)
                }

                fn get_hash(&self, key: &[u8]) -> Option<&::netabase_store::traits::subscription::ModelHash> {
                    self.inner.get_hash(key)
                }

                fn get_all_keys(&self) -> Vec<Vec<u8>> {
                    self.inner.get_all_keys()
                }

                fn get_all_items(&self) -> Vec<(Vec<u8>, ::netabase_store::traits::subscription::ModelHash)> {
                    self.inner.get_all_items()
                }

                fn rebuild_merkle_tree(&mut self) -> Result<(), ::netabase_store::error::NetabaseError> {
                    self.inner.rebuild_merkle_tree()
                }
            }
        };

        tree_impls.extend(tree_impl);
    }

    tree_impls
}

/// Generate subscription manager implementation
pub fn generate_store_subscription_impls(
    definition_name: &Ident,
    enum_name: &Ident,
    topics: &[String],
) -> TokenStream {
    let variants: Vec<Ident> = topics
        .iter()
        .map(|topic| format_ident!("{}", topic))
        .collect();
    let variant_fields: Vec<Ident> = topics
        .iter()
        .map(|topic| format_ident!("{}_tree", topic.to_lowercase()))
        .collect();
    let tree_struct_names: Vec<Ident> = topics
        .iter()
        .map(|topic| format_ident!("{}SubscriptionTree", topic))
        .collect();

    let manager_name = format_ident!("{}SubscriptionManager", definition_name);
    let manager_doc = format!(
        "Concrete subscription manager for {} schema",
        definition_name
    );

    quote! {
        #[doc = #manager_doc]
        #[derive(Clone)]
        pub struct #manager_name {
            #(
                #variant_fields: #tree_struct_names,
            )*
        }

        impl #manager_name {
            /// Create a new subscription manager with all topics initialized
            pub fn new() -> Self {
                Self {
                    #(
                        #variant_fields: #tree_struct_names::new(),
                    )*
                }
            }

            /// Get a subscription tree for a specific topic (immutable)
            pub fn get_tree(&self, topic: #enum_name) -> Option<&dyn ::netabase_store::traits::subscription::SubscriptionTree<#definition_name, Topic = #enum_name>> {
                match topic {
                    #(
                        #enum_name::#variants => Some(&self.#variant_fields),
                    )*
                }
            }

            /// Get a subscription tree for a specific topic (mutable)
            pub fn get_tree_mut(&mut self, topic: #enum_name) -> Option<&mut dyn ::netabase_store::traits::subscription::SubscriptionTree<#definition_name, Topic = #enum_name>> {
                match topic {
                    #(
                        #enum_name::#variants => Some(&mut self.#variant_fields),
                    )*
                }
            }

            /// Add an item to a specific topic's subscription tree
            pub fn subscribe_item<T>(&mut self, topic: #enum_name, key: Vec<u8>, data: &T) -> Result<(), ::netabase_store::error::NetabaseError>
            where
                T: AsRef<[u8]>,
            {
                use ::netabase_store::traits::subscription::SubscriptionTree;
                let hash = ::netabase_store::traits::subscription::ModelHash::from_key_and_data(&key, data);
                match topic {
                    #(
                        #enum_name::#variants => self.#variant_fields.put_item(key, hash),
                    )*
                }
            }

            /// Remove an item from a specific topic's subscription tree
            pub fn unsubscribe_item(&mut self, topic: #enum_name, key: &[u8]) -> Result<Option<::netabase_store::traits::subscription::ModelHash>, ::netabase_store::error::NetabaseError> {
                use ::netabase_store::traits::subscription::SubscriptionTree;
                match topic {
                    #(
                        #enum_name::#variants => self.#variant_fields.remove_item(key),
                    )*
                }
            }

            /// Get the merkle root for a specific topic
            pub fn topic_merkle_root(&mut self, topic: #enum_name) -> Result<Option<[u8; 32]>, ::netabase_store::error::NetabaseError> {
                use ::netabase_store::traits::subscription::SubscriptionTree;
                match topic {
                    #(
                        #enum_name::#variants => self.#variant_fields.merkle_root(),
                    )*
                }
            }

            /// Get statistics about this subscription manager
            pub fn stats(&self) -> ::netabase_store::traits::subscription::SubscriptionStats {
                use ::netabase_store::traits::subscription::SubscriptionTree;
                let mut stats = ::netabase_store::traits::subscription::SubscriptionStats::new();
                #(
                    if self.#variant_fields.len() > 0 {
                        stats.add_topic_count(self.#variant_fields.len());
                    }
                )*
                stats
            }

            /// Compare with another subscription manager and get all topic differences
            pub fn compare_with(&mut self, other: &mut Self) -> Result<Vec<(#enum_name, ::netabase_store::subscription::subscription_tree::SubscriptionDiff<#definition_name>)>, ::netabase_store::error::NetabaseError> {
                let mut diffs = Vec::new();
                #(
                    let diff = self.#variant_fields.inner.compare_with(&mut other.#variant_fields.inner)?;
                    if diff.has_differences() {
                        diffs.push((#enum_name::#variants, diff));
                    }
                )*
                Ok(diffs)
            }

            /// Rebuild all merkle trees
            pub fn rebuild_all_merkle_trees(&mut self) -> Result<(), ::netabase_store::error::NetabaseError> {
                use ::netabase_store::traits::subscription::SubscriptionTree;
                #(
                    self.#variant_fields.rebuild_merkle_tree()?;
                )*
                Ok(())
            }

            /// Clear all subscription trees
            pub fn clear_all(&mut self) -> Result<(), ::netabase_store::error::NetabaseError> {
                use ::netabase_store::traits::subscription::SubscriptionTree;
                #(
                    self.#variant_fields.clear()?;
                )*
                Ok(())
            }

            /// Check if a key exists in any topic
            pub fn contains_key_in_any_topic(&self, key: &[u8]) -> bool {
                use ::netabase_store::traits::subscription::SubscriptionTree;
                #(
                    self.#variant_fields.contains_key(key) ||
                )* false
            }

            /// Get all topics containing a specific key
            pub fn topics_containing_key(&self, key: &[u8]) -> Vec<#enum_name> {
                use ::netabase_store::traits::subscription::SubscriptionTree;
                let mut topics = Vec::new();
                #(
                    if self.#variant_fields.contains_key(key) {
                        topics.push(#enum_name::#variants);
                    }
                )*
                topics
            }
        }

        impl Default for #manager_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl ::netabase_store::traits::subscription::SubscriptionManager<#definition_name> for #manager_name {
            type TopicType = #enum_name;

            fn subscribe_item<T>(&mut self, topic: Self::TopicType, key: Vec<u8>, data: &T) -> Result<(), ::netabase_store::error::NetabaseError>
            where
                T: AsRef<[u8]>,
            {
                self.subscribe_item(topic, key, data)
            }

            fn unsubscribe_item(&mut self, topic: Self::TopicType, key: &[u8]) -> Result<Option<::netabase_store::traits::subscription::ModelHash>, ::netabase_store::error::NetabaseError> {
                self.unsubscribe_item(topic, key)
            }

            fn topic_merkle_root(&mut self, topic: Self::TopicType) -> Result<Option<[u8; 32]>, ::netabase_store::error::NetabaseError> {
                self.topic_merkle_root(topic)
            }

            fn stats(&self) -> ::netabase_store::traits::subscription::SubscriptionStats {
                self.stats()
            }
        }

        /// Helper method for creating subscription managers
        impl #definition_name {
            /// Create a new subscription manager for this definition
            pub fn new_subscription_manager() -> #manager_name {
                #manager_name::new()
            }
        }
    }
}

/// Generate subscription utility types and functions
pub fn generate_subscription_utilities(
    definition_name: &Ident,
    enum_name: &Ident,
    _topics: &[String],
) -> TokenStream {
    let sync_helper_name = format_ident!("{}SyncHelper", definition_name);
    let manager_name = format_ident!("{}SubscriptionManager", definition_name);

    quote! {
        /// Helper for synchronizing subscription data between nodes
        pub struct #sync_helper_name;

        impl #sync_helper_name {
            /// Create a sync plan based on subscription differences
            pub fn create_sync_plan(
                diffs: &[(#enum_name, ::netabase_store::subscription::subscription_tree::SubscriptionDiff<#definition_name>)],
            ) -> Vec<(#enum_name, Vec<Vec<u8>>)> {
                diffs.iter().map(|(topic, diff)| (*topic, diff.keys_needed_by_self().into_iter().cloned().collect())).collect()
            }

            /// Get all merkle roots from a subscription manager
            pub fn get_all_roots(
                manager: &mut #manager_name,
            ) -> Result<std::collections::HashMap<#enum_name, Option<[u8; 32]>>, ::netabase_store::error::NetabaseError> {
                use ::netabase_store::traits::subscription::{Subscriptions, SubscriptionManager};
                let mut roots = std::collections::HashMap::new();
                for topic in #definition_name::all_subscriptions() {
                    roots.insert(topic, manager.topic_merkle_root(topic)?);
                }
                Ok(roots)
            }

            /// Compare roots between two subscription managers
            pub fn compare_roots(
                roots_a: &std::collections::HashMap<#enum_name, Option<[u8; 32]>>,
                roots_b: &std::collections::HashMap<#enum_name, Option<[u8; 32]>>,
            ) -> Vec<#enum_name> {
                use ::netabase_store::traits::subscription::Subscriptions;
                let mut differing_topics = Vec::new();

                for topic in #definition_name::all_subscriptions() {
                    let root_a = roots_a.get(&topic);
                    let root_b = roots_b.get(&topic);

                    match (root_a, root_b) {
                        (Some(Some(a)), Some(Some(b))) if a != b => {
                            differing_topics.push(topic);
                        }
                        (Some(None), Some(Some(_))) | (Some(Some(_)), Some(None)) => {
                            differing_topics.push(topic);
                        }
                        _ => {}
                    }
                }

                differing_topics
            }
        }

        /// Result of a synchronization operation
        #[derive(Debug, Clone)]
        pub struct SyncResult {
            pub topics_synced: usize,
            pub items_downloaded: usize,
            pub items_uploaded: usize,
            pub conflicts_resolved: usize,
            pub errors: Vec<String>,
        }

        impl SyncResult {
            /// Create a new sync result
            pub fn new() -> Self {
                Self {
                    topics_synced: 0,
                    items_downloaded: 0,
                    items_uploaded: 0,
                    conflicts_resolved: 0,
                    errors: Vec::new(),
                }
            }

            /// Check if the sync was successful
            pub fn is_success(&self) -> bool {
                self.errors.is_empty()
            }

            /// Get total number of operations performed
            pub fn total_operations(&self) -> usize {
                self.items_downloaded + self.items_uploaded + self.conflicts_resolved
            }
        }

        impl Default for SyncResult {
            fn default() -> Self {
                Self::new()
            }
        }
    }
}

/// Generate database integration utilities
pub fn generate_database_integration(
    definition_name: &Ident,
    enum_name: &Ident,
    _topics: &[String],
) -> TokenStream {
    let manager_name = format_ident!("{}SubscriptionManager", definition_name);

    quote! {
        /// Trait for database stores that support subscriptions
        pub trait SubscriptionStore {
            type SubscriptionManager;

            /// Get a reference to the subscription manager
            fn subscription_manager(&self) -> &Self::SubscriptionManager;

            /// Get a mutable reference to the subscription manager
            fn subscription_manager_mut(&mut self) -> &mut Self::SubscriptionManager;

            /// Automatically subscribe an item to a topic on put
            fn auto_subscribe<T>(&mut self, topic: #enum_name, key: Vec<u8>, data: &T) -> Result<(), ::netabase_store::error::NetabaseError>
            where
                T: AsRef<[u8]>;

            /// Automatically unsubscribe an item from a topic on remove
            fn auto_unsubscribe(&mut self, topic: #enum_name, key: &[u8]) -> Result<Option<::netabase_store::traits::subscription::ModelHash>, ::netabase_store::error::NetabaseError>;
        }

        /// Helper for working with subscriptions in a database context
        pub struct SubscriptionHelper;

        impl SubscriptionHelper {
            /// Get all subscription topics for a definition
            pub fn get_subscription_topics() -> Vec<#enum_name> {
                use ::netabase_store::traits::subscription::Subscriptions;
                #definition_name::all_subscriptions()
            }

            /// Create a default subscription filter
            pub fn create_default_filter() -> ::netabase_store::traits::subscription::DefaultSubscriptionFilter {
                ::netabase_store::traits::subscription::DefaultSubscriptionFilter
            }
        }
    }
}
