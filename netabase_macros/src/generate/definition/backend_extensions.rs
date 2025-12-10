//! Backend-specific extension trait implementations
//!
//! Generates the massive pattern-matching implementations that allow
//! definition enums to interact with Redb and Sled backends.
//!
//! # Generated Traits
//!
//! - `RedbModelAssociatedTypesExt`: Methods for Redb backend operations
//! - `SledModelAssociatedTypesExt`: Methods for Sled backend operations
//!
//! Each trait contains methods with N match arms (one per model in the definition).

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::parse::metadata::ModuleMetadata;

/// Generate RedbModelAssociatedTypesExt implementation for a definition
///
/// This generates a trait with methods that pattern match on the definition enum
/// to perform Redb-specific operations for each model.
///
/// # Generated Methods
///
/// - `insert_model_into_redb`: Insert a model instance into main tree
/// - `insert_hash_into_redb`: Insert model hash into hash tree
/// - `insert_secondary_key_into_redb`: Insert secondary key index
/// - `insert_relational_key_into_redb`: Insert relational link
/// - `delete_model_from_redb`: Delete model and all its indices
/// - `get_model_from_redb`: Retrieve model by primary key
/// - `get_by_secondary_key_from_redb`: Retrieve by secondary key
pub fn generate_redb_extension(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let model_associated_types = quote::format_ident!("{}ModelAssociatedTypes", definition_name);

    // Generate each method with match arms for all models
    let insert_model_method = generate_insert_model_method(module);
    let insert_hash_method = generate_insert_hash_method(module);
    let insert_secondary_method = generate_insert_secondary_method(module);
    let insert_relational_method = generate_insert_relational_method(module);
    let delete_model_method = generate_delete_model_method(module);
    let get_model_method = generate_get_model_method(module);
    let get_by_secondary_method = generate_get_by_secondary_method(module);

    quote! {
        /// Extension trait for Redb backend operations on this definition
        pub trait RedbModelAssociatedTypesExt {
            /// Insert a model into the main tree
            fn insert_model_into_redb(
                &self,
                txn: &redb::WriteTransaction,
            ) -> Result<(), redb::Error>;

            /// Insert model hash into hash tree
            fn insert_hash_into_redb(
                &self,
                txn: &redb::WriteTransaction,
            ) -> Result<(), redb::Error>;

            /// Insert secondary key index
            fn insert_secondary_key_into_redb(
                &self,
                key_type: &#model_associated_types,
                txn: &redb::WriteTransaction,
            ) -> Result<(), redb::Error>;

            /// Insert relational link
            fn insert_relational_key_into_redb(
                &self,
                key_type: &#model_associated_types,
                txn: &redb::WriteTransaction,
            ) -> Result<(), redb::Error>;

            /// Delete model and all its indices
            fn delete_model_from_redb(
                &self,
                txn: &redb::WriteTransaction,
            ) -> Result<(), redb::Error>;

            /// Get model by primary key
            fn get_model_from_redb(
                key: &#model_associated_types,
                txn: &redb::ReadTransaction,
            ) -> Result<Option<#definition_name>, redb::Error>;

            /// Get model by secondary key
            fn get_by_secondary_key_from_redb(
                key: &#model_associated_types,
                txn: &redb::ReadTransaction,
            ) -> Result<Option<#definition_name>, redb::Error>;
        }

        impl RedbModelAssociatedTypesExt for #definition_name {
            #insert_model_method
            #insert_hash_method
            #insert_secondary_method
            #insert_relational_method
            #delete_model_method
            #get_model_method
            #get_by_secondary_method
        }
    }
}

/// Generate insert_model_into_redb method with match arms for each model
fn generate_insert_model_method(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        let tree_name_const = quote! { #model_name::MAIN_TREE_NAME };

        quote! {
            #definition_name::#model_name(model) => {
                let table = txn.open_table::<_, Vec<u8>>(#tree_name_const)?;
                let key = model.primary_key();
                let value = bincode::encode_to_vec(model, bincode::config::standard())
                    .map_err(|e| redb::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Bincode encoding error: {}", e)
                    )))?;
                table.insert(&key, value.as_slice())?;
                Ok(())
            }
        }
    }).collect();

    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let nested_def_name = &nested.definition_name;
        quote! {
            #definition_name::#nested_def_name(_) => {
                Err(redb::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot insert nested definition into parent store"
                )))
            }
        }
    }).collect();

    quote! {
        fn insert_model_into_redb(
            &self,
            txn: &redb::WriteTransaction,
        ) -> Result<(), redb::Error> {
            match self {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate insert_hash_into_redb method
fn generate_insert_hash_method(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        let tree_name_const = quote! { #model_name::HASH_TREE_NAME };

        quote! {
            #definition_name::#model_name(model) => {
                let table = txn.open_table::<[u8; 32], _>(#tree_name_const)?;
                let hash = model.compute_hash();
                let key = model.primary_key();
                table.insert(&hash, &key)?;
                Ok(())
            }
        }
    }).collect();

    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let nested_def_name = &nested.definition_name;
        quote! {
            #definition_name::#nested_def_name(_) => {
                Err(redb::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot insert nested definition hash into parent store"
                )))
            }
        }
    }).collect();

    quote! {
        fn insert_hash_into_redb(
            &self,
            txn: &redb::WriteTransaction,
        ) -> Result<(), redb::Error> {
            match self {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate insert_secondary_key_into_redb method
fn generate_insert_secondary_method(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let model_associated_types = quote::format_ident!("{}ModelAssociatedTypes", definition_name);

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        let secondary_fields = model.secondary_key_fields();

        if secondary_fields.is_empty() {
            // No secondary keys for this model
            quote! {
                #definition_name::#model_name(_) => Ok(())
            }
        } else {
            let secondary_key_enum = quote::format_ident!("{}SecondaryKeys", model_name);

            // Generate inner match for each secondary key
            let secondary_match_arms: Vec<_> = secondary_fields.iter().enumerate().map(|(idx, _field)| {
                let tree_name = quote! { #model_name::SECONDARY_TREE_NAMES[#idx] };

                quote! {
                    #model_associated_types::#secondary_key_enum(key) => {
                        let table = txn.open_table::<_, _>(#tree_name)?;
                        let pk = model.primary_key();
                        table.insert(key, &pk)?;
                        Ok(())
                    }
                }
            }).collect();

            quote! {
                #definition_name::#model_name(model) => {
                    match key_type {
                        #(#secondary_match_arms,)*
                        _ => Ok(())
                    }
                }
            }
        }
    }).collect();

    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let nested_def_name = &nested.definition_name;
        quote! {
            #definition_name::#nested_def_name(_) => {
                Err(redb::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot insert nested definition secondary key into parent store"
                )))
            }
        }
    }).collect();

    quote! {
        fn insert_secondary_key_into_redb(
            &self,
            key_type: &#model_associated_types,
            txn: &redb::WriteTransaction,
        ) -> Result<(), redb::Error> {
            match self {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate insert_relational_key_into_redb method
fn generate_insert_relational_method(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let model_associated_types = quote::format_ident!("{}ModelAssociatedTypes", definition_name);

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        let relational_fields = model.relational_fields();

        if relational_fields.is_empty() {
            quote! {
                #definition_name::#model_name(_) => Ok(())
            }
        } else {
            let relational_key_enum = quote::format_ident!("{}RelationalKeys", model_name);

            let relational_match_arms: Vec<_> = relational_fields.iter().enumerate().map(|(idx, _field)| {
                let tree_name = quote! { #model_name::RELATIONAL_TREE_NAMES[#idx] };

                quote! {
                    #model_associated_types::#relational_key_enum(key) => {
                        let table = txn.open_table::<_, _>(#tree_name)?;
                        let pk = model.primary_key();
                        table.insert(key, &pk)?;
                        Ok(())
                    }
                }
            }).collect();

            quote! {
                #definition_name::#model_name(model) => {
                    match key_type {
                        #(#relational_match_arms,)*
                        _ => Ok(())
                    }
                }
            }
        }
    }).collect();

    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let nested_def_name = &nested.definition_name;
        quote! {
            #definition_name::#nested_def_name(_) => {
                Err(redb::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot insert nested definition relational key into parent store"
                )))
            }
        }
    }).collect();

    quote! {
        fn insert_relational_key_into_redb(
            &self,
            key_type: &#model_associated_types,
            txn: &redb::WriteTransaction,
        ) -> Result<(), redb::Error> {
            match self {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate delete_model_from_redb method
fn generate_delete_model_method(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;

        quote! {
            #definition_name::#model_name(model) => {
                // Delete from main tree
                let main_table = txn.open_table::<_, Vec<u8>>(#model_name::MAIN_TREE_NAME)?;
                let key = model.primary_key();
                main_table.remove(&key)?;

                // Delete from hash tree
                let hash_table = txn.open_table::<[u8; 32], _>(#model_name::HASH_TREE_NAME)?;
                let hash = model.compute_hash();
                hash_table.remove(&hash)?;

                // Delete from secondary key trees
                for tree_name in #model_name::SECONDARY_TREE_NAMES {
                    let table = txn.open_table::<Vec<u8>, _>(tree_name)?;
                    // Note: Would need actual secondary key values to delete properly
                    // This is a simplified version
                }

                // Delete from relational trees
                for tree_name in #model_name::RELATIONAL_TREE_NAMES {
                    let table = txn.open_table::<Vec<u8>, _>(tree_name)?;
                    // Note: Would need actual relational key values to delete properly
                }

                Ok(())
            }
        }
    }).collect();

    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let nested_def_name = &nested.definition_name;
        quote! {
            #definition_name::#nested_def_name(_) => {
                Err(redb::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot delete nested definition from parent store"
                )))
            }
        }
    }).collect();

    quote! {
        fn delete_model_from_redb(
            &self,
            txn: &redb::WriteTransaction,
        ) -> Result<(), redb::Error> {
            match self {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate get_model_from_redb method
fn generate_get_model_method(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let model_associated_types = quote::format_ident!("{}ModelAssociatedTypes", definition_name);

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        let pk_wrapper = quote::format_ident!("{}Id", model_name);

        quote! {
            #model_associated_types::#pk_wrapper(key) => {
                let table = txn.open_table::<_, Vec<u8>>(#model_name::MAIN_TREE_NAME)?;
                match table.get(key)? {
                    Some(value) => {
                        let bytes = value.value();
                        let (model, _) = bincode::decode_from_slice::<#model_name>(
                            bytes,
                            bincode::config::standard()
                        ).map_err(|e| redb::Error::Io(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Bincode decoding error: {}", e)
                        )))?;
                        Ok(Some(#definition_name::#model_name(model)))
                    }
                    None => Ok(None)
                }
            }
        }
    }).collect();

    // Nested modules are not stored in parent's DB
    // Returning Ok(None) effectively says "Not found in this store"

    quote! {
        fn get_model_from_redb(
            key: &#model_associated_types,
            txn: &redb::ReadTransaction,
        ) -> Result<Option<#definition_name>, redb::Error> {
            match key {
                #(#match_arms,)*
                _ => Ok(None)
            }
        }
    }
}

/// Generate get_by_secondary_key_from_redb method
fn generate_get_by_secondary_method(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let model_associated_types = quote::format_ident!("{}ModelAssociatedTypes", definition_name);

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        let secondary_fields = model.secondary_key_fields();

        if secondary_fields.is_empty() {
            return quote! {};
        }

        let secondary_key_enum = quote::format_ident!("{}SecondaryKeys", model_name);

        let secondary_match_arms: Vec<_> = secondary_fields.iter().enumerate().map(|(idx, _field)| {
            let tree_name = quote! { #model_name::SECONDARY_TREE_NAMES[#idx] };
            let pk_wrapper = quote::format_ident!("{}Id", model_name);

            quote! {
                #model_associated_types::#secondary_key_enum(sec_key) => {
                    // First, get primary key from secondary index
                    let sec_table = txn.open_table::<_, #pk_wrapper>(#tree_name)?;
                    match sec_table.get(sec_key)? {
                        Some(pk_guard) => {
                            let pk = pk_guard.value();
                            // Then get the actual model
                            let main_table = txn.open_table::<_, Vec<u8>>(#model_name::MAIN_TREE_NAME)?;
                            match main_table.get(&pk)? {
                                Some(value) => {
                                    let bytes = value.value();
                                    let (model, _) = bincode::decode_from_slice::<#model_name>(
                                        bytes,
                                        bincode::config::standard()
                                    ).map_err(|e| redb::Error::Io(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        format!("Bincode decoding error: {}", e)
                                    )))?;
                                    Ok(Some(#definition_name::#model_name(model)))
                                }
                                None => Ok(None)
                            }
                        }
                        None => Ok(None)
                    }
                }
            }
        }).collect();

        quote! {
            #(#secondary_match_arms)*
        }
    }).collect();

    quote! {
        fn get_by_secondary_key_from_redb(
            key: &#model_associated_types,
            txn: &redb::ReadTransaction,
        ) -> Result<Option<#definition_name>, redb::Error> {
            match key {
                #(#match_arms,)*
                _ => Ok(None)
            }
        }
    }
}

/// Generate SledModelAssociatedTypesExt implementation for a definition
///
/// This generates a trait with methods that pattern match on the definition enum
/// to perform Sled-specific operations for each model.
///
/// # Generated Methods
///
/// Similar to Redb extension but using Sled's Tree API:
/// - `insert_model_into_sled`: Insert a model instance into main tree
/// - `insert_hash_into_sled`: Insert model hash into hash tree
/// - `insert_secondary_key_into_sled`: Insert secondary key index
/// - `insert_relational_key_into_sled`: Insert relational link
/// - `delete_model_from_sled`: Delete model and all its indices
/// - `get_model_from_sled`: Retrieve model by primary key
/// - `get_by_secondary_key_from_sled`: Retrieve by secondary key
pub fn generate_sled_extension(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let model_associated_types = quote::format_ident!("{}ModelAssociatedTypes", definition_name);

    let insert_model_method = generate_sled_insert_model_method(module);
    let insert_hash_method = generate_sled_insert_hash_method(module);
    let insert_secondary_method = generate_sled_insert_secondary_method(module);
    let insert_relational_method = generate_sled_insert_relational_method(module);
    let delete_model_method = generate_sled_delete_model_method(module);
    let get_model_method = generate_sled_get_model_method(module);
    let get_by_secondary_method = generate_sled_get_by_secondary_method(module);

    quote! {
        /// Extension trait for Sled backend operations on this definition
        pub trait SledModelAssociatedTypesExt {
            /// Insert a model into the main tree
            fn insert_model_into_sled(
                &self,
                db: &sled::Db,
            ) -> Result<(), sled::Error>;

            /// Insert model hash into hash tree
            fn insert_hash_into_sled(
                &self,
                db: &sled::Db,
            ) -> Result<(), sled::Error>;

            /// Insert secondary key index
            fn insert_secondary_key_into_sled(
                &self,
                key_type: &#model_associated_types,
                db: &sled::Db,
            ) -> Result<(), sled::Error>;

            /// Insert relational link
            fn insert_relational_key_into_sled(
                &self,
                key_type: &#model_associated_types,
                db: &sled::Db,
            ) -> Result<(), sled::Error>;

            /// Delete model and all its indices
            fn delete_model_from_sled(
                &self,
                db: &sled::Db,
            ) -> Result<(), sled::Error>;

            /// Get model by primary key
            fn get_model_from_sled(
                key: &#model_associated_types,
                db: &sled::Db,
            ) -> Result<Option<#definition_name>, Box<dyn std::error::Error>>;

            /// Get model by secondary key
            fn get_by_secondary_key_from_sled(
                key: &#model_associated_types,
                db: &sled::Db,
            ) -> Result<Option<#definition_name>, Box<dyn std::error::Error>>;
        }

        impl SledModelAssociatedTypesExt for #definition_name {
            #insert_model_method
            #insert_hash_method
            #insert_secondary_method
            #insert_relational_method
            #delete_model_method
            #get_model_method
            #get_by_secondary_method
        }
    }
}

/// Generate insert_model_into_sled method
fn generate_sled_insert_model_method(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;

        quote! {
            #definition_name::#model_name(model) => {
                let tree = db.open_tree(#model_name::MAIN_TREE_NAME)?;
                let key_bytes: Vec<u8> = model.primary_key().try_into()
                    .map_err(|e| sled::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Key conversion error: {:?}", e)
                    )))?;
                let value_bytes = bincode::encode_to_vec(model, bincode::config::standard())
                    .map_err(|e| sled::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Bincode encoding error: {}", e)
                    )))?;
                tree.insert(key_bytes, value_bytes)?;
                Ok(())
            }
        }
    }).collect();

    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let nested_def_name = &nested.definition_name;
        quote! {
            #definition_name::#nested_def_name(_) => {
                Err(sled::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot insert nested definition into parent store"
                )))
            }
        }
    }).collect();

    quote! {
        fn insert_model_into_sled(
            &self,
            db: &sled::Db,
        ) -> Result<(), sled::Error> {
            match self {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate insert_hash_into_sled method
fn generate_sled_insert_hash_method(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;

        quote! {
            #definition_name::#model_name(model) => {
                let tree = db.open_tree(#model_name::HASH_TREE_NAME)?;
                let hash = model.compute_hash();
                let key_bytes: Vec<u8> = model.primary_key().try_into()
                    .map_err(|e| sled::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Key conversion error: {:?}", e)
                    )))?;
                tree.insert(&hash, key_bytes)?;
                Ok(())
            }
        }
    }).collect();

    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let nested_def_name = &nested.definition_name;
        quote! {
            #definition_name::#nested_def_name(_) => {
                Err(sled::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot insert nested definition hash into parent store"
                )))
            }
        }
    }).collect();

    quote! {
        fn insert_hash_into_sled(
            &self,
            db: &sled::Db,
        ) -> Result<(), sled::Error> {
            match self {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate insert_secondary_key_into_sled method
fn generate_sled_insert_secondary_method(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let model_associated_types = quote::format_ident!("{}ModelAssociatedTypes", definition_name);

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        let secondary_fields = model.secondary_key_fields();

        if secondary_fields.is_empty() {
            quote! {
                #definition_name::#model_name(_) => Ok(())
            }
        } else {
            let secondary_key_enum = quote::format_ident!("{}SecondaryKeys", model_name);

            let secondary_match_arms: Vec<_> = secondary_fields.iter().enumerate().map(|(idx, _field)| {
                let tree_name = quote! { #model_name::SECONDARY_TREE_NAMES[#idx] };

                quote! {
                    #model_associated_types::#secondary_key_enum(key) => {
                        let tree = db.open_tree(#tree_name)?;
                        let sec_key_bytes: Vec<u8> = key.clone().try_into()
                            .map_err(|e| sled::Error::Io(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Secondary key conversion error: {:?}", e)
                            )))?;
                        let pk_bytes: Vec<u8> = model.primary_key().try_into()
                            .map_err(|e| sled::Error::Io(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Primary key conversion error: {:?}", e)
                            )))?;
                        tree.insert(sec_key_bytes, pk_bytes)?;
                        Ok(())
                    }
                }
            }).collect();

            quote! {
                #definition_name::#model_name(model) => {
                    match key_type {
                        #(#secondary_match_arms,)*
                        _ => Ok(())
                    }
                }
            }
        }
    }).collect();

    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let nested_def_name = &nested.definition_name;
        quote! {
            #definition_name::#nested_def_name(_) => {
                Err(sled::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot insert nested definition secondary key into parent store"
                )))
            }
        }
    }).collect();

    quote! {
        fn insert_secondary_key_into_sled(
            &self,
            key_type: &#model_associated_types,
            db: &sled::Db,
        ) -> Result<(), sled::Error> {
            match self {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate insert_relational_key_into_sled method
fn generate_sled_insert_relational_method(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let model_associated_types = quote::format_ident!("{}ModelAssociatedTypes", definition_name);

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        let relational_fields = model.relational_fields();

        if relational_fields.is_empty() {
            quote! {
                #definition_name::#model_name(_) => Ok(())
            }
        } else {
            let relational_key_enum = quote::format_ident!("{}RelationalKeys", model_name);

            let relational_match_arms: Vec<_> = relational_fields.iter().enumerate().map(|(idx, _field)| {
                let tree_name = quote! { #model_name::RELATIONAL_TREE_NAMES[#idx] };

                quote! {
                    #model_associated_types::#relational_key_enum(key) => {
                        let tree = db.open_tree(#tree_name)?;
                        let rel_key_bytes: Vec<u8> = key.clone().try_into()
                            .map_err(|e| sled::Error::Io(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Relational key conversion error: {:?}", e)
                            )))?;
                        let pk_bytes: Vec<u8> = model.primary_key().try_into()
                            .map_err(|e| sled::Error::Io(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Primary key conversion error: {:?}", e)
                            )))?;
                        tree.insert(rel_key_bytes, pk_bytes)?;
                        Ok(())
                    }
                }
            }).collect();

            quote! {
                #definition_name::#model_name(model) => {
                    match key_type {
                        #(#relational_match_arms,)*
                        _ => Ok(())
                    }
                }
            }
        }
    }).collect();

    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let nested_def_name = &nested.definition_name;
        quote! {
            #definition_name::#nested_def_name(_) => {
                Err(sled::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot insert nested definition relational key into parent store"
                )))
            }
        }
    }).collect();

    quote! {
        fn insert_relational_key_into_sled(
            &self,
            key_type: &#model_associated_types,
            db: &sled::Db,
        ) -> Result<(), sled::Error> {
            match self {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate delete_model_from_sled method
fn generate_sled_delete_model_method(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;

        quote! {
            #definition_name::#model_name(model) => {
                // Delete from main tree
                let main_tree = db.open_tree(#model_name::MAIN_TREE_NAME)?;
                let key_bytes: Vec<u8> = model.primary_key().try_into()
                    .map_err(|e| sled::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Key conversion error: {:?}", e)
                    )))?;
                main_tree.remove(&key_bytes)?;

                // Delete from hash tree
                let hash_tree = db.open_tree(#model_name::HASH_TREE_NAME)?;
                let hash = model.compute_hash();
                hash_tree.remove(&hash)?;

                // Delete from secondary key trees
                for tree_name in #model_name::SECONDARY_TREE_NAMES {
                    let tree = db.open_tree(tree_name)?;
                    // Note: Would need actual secondary key values
                }

                // Delete from relational trees
                for tree_name in #model_name::RELATIONAL_TREE_NAMES {
                    let tree = db.open_tree(tree_name)?;
                    // Note: Would need actual relational key values
                }

                Ok(())
            }
        }
    }).collect();

    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let nested_def_name = &nested.definition_name;
        quote! {
            #definition_name::#nested_def_name(_) => {
                Err(sled::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Cannot delete nested definition from parent store"
                )))
            }
        }
    }).collect();

    quote! {
        fn delete_model_from_sled(
            &self,
            db: &sled::Db,
        ) -> Result<(), sled::Error> {
            match self {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate get_model_from_sled method
fn generate_sled_get_model_method(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let model_associated_types = quote::format_ident!("{}ModelAssociatedTypes", definition_name);

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        let pk_wrapper = quote::format_ident!("{}Id", model_name);

        quote! {
            #model_associated_types::#pk_wrapper(key) => {
                let tree = db.open_tree(#model_name::MAIN_TREE_NAME)?;
                let key_bytes: Vec<u8> = key.clone().try_into()?;
                match tree.get(&key_bytes)? {
                    Some(value_bytes) => {
                        let (model, _) = bincode::decode_from_slice::<#model_name>(
                            &value_bytes,
                            bincode::config::standard()
                        )?;
                        Ok(Some(#definition_name::#model_name(model)))
                    }
                    None => Ok(None)
                }
            }
        }
    }).collect();

    quote! {
        fn get_model_from_sled(
            key: &#model_associated_types,
            db: &sled::Db,
        ) -> Result<Option<#definition_name>, Box<dyn std::error::Error>> {
            match key {
                #(#match_arms,)*
                _ => Ok(None)
            }
        }
    }
}

/// Generate get_by_secondary_key_from_sled method
fn generate_sled_get_by_secondary_method(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let model_associated_types = quote::format_ident!("{}ModelAssociatedTypes", definition_name);

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        let secondary_fields = model.secondary_key_fields();

        if secondary_fields.is_empty() {
            return quote! {};
        }

        let secondary_key_enum = quote::format_ident!("{}SecondaryKeys", model_name);

        let secondary_match_arms: Vec<_> = secondary_fields.iter().enumerate().map(|(idx, _field)| {
            let tree_name = quote! { #model_name::SECONDARY_TREE_NAMES[#idx] };

            quote! {
                #model_associated_types::#secondary_key_enum(sec_key) => {
                    // First, get primary key from secondary index
                    let sec_tree = db.open_tree(#tree_name)?;
                    let sec_key_bytes: Vec<u8> = sec_key.clone().try_into()?;
                    match sec_tree.get(&sec_key_bytes)? {
                        Some(pk_bytes) => {
                            // Convert pk_bytes back to primary key type
                            let pk = <_ as TryFrom<Vec<u8>>>::try_from(pk_bytes.to_vec())?;
                            // Then get the actual model
                            let main_tree = db.open_tree(#model_name::MAIN_TREE_NAME)?;
                            let pk_key_bytes: Vec<u8> = pk.try_into()?;
                            match main_tree.get(&pk_key_bytes)? {
                                Some(value_bytes) => {
                                    let (model, _) = bincode::decode_from_slice::<#model_name>(
                                        &value_bytes,
                                        bincode::config::standard()
                                    )?;
                                    Ok(Some(#definition_name::#model_name(model)))
                                }
                                None => Ok(None)
                            }
                        }
                        None => Ok(None)
                    }
                }
            }
        }).collect();

        quote! {
            #(#secondary_match_arms)*
        }
    }).collect();

    quote! {
        fn get_by_secondary_key_from_sled(
            key: &#model_associated_types,
            db: &sled::Db,
        ) -> Result<Option<#definition_name>, Box<dyn std::error::Error>> {
            match key {
                #(#match_arms,)*
                _ => Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::metadata::{ModuleMetadata, ModelMetadata, FieldMetadata};
    use syn::parse_quote;

    #[test]
    fn test_generate_redb_extension_minimal() {
        let mut module = ModuleMetadata::new(
            parse_quote!(test_mod),
            parse_quote!(TestDef),
            parse_quote!(TestDefKeys)
        );

        let mut user = ModelMetadata::new(parse_quote!(User), parse_quote!(pub));
        let mut pk = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        pk.is_primary_key = true;
        user.add_field(pk);
        module.add_model(user);

        let result = generate_redb_extension(&module);
        let code = result.to_string();

        assert!(code.contains("trait RedbModelAssociatedTypesExt"));
        assert!(code.contains("fn insert_model_into_redb"));
        assert!(code.contains("fn insert_hash_into_redb"));
        assert!(code.contains("fn delete_model_from_redb"));
        assert!(code.contains("TestDef :: User"));
    }

    #[test]
    fn test_generate_redb_extension_with_secondary_keys() {
        let mut module = ModuleMetadata::new(
            parse_quote!(app_mod),
            parse_quote!(AppDef),
            parse_quote!(AppDefKeys)
        );

        let mut user = ModelMetadata::new(parse_quote!(User), parse_quote!(pub));

        let mut pk = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        pk.is_primary_key = true;
        user.add_field(pk);

        let mut email = FieldMetadata::new(
            parse_quote!(email),
            parse_quote!(String),
            parse_quote!(pub)
        );
        email.is_secondary_key = true;
        user.add_field(email);

        module.add_model(user);

        let result = generate_redb_extension(&module);
        let code = result.to_string();

        assert!(code.contains("fn insert_secondary_key_into_redb"));
        assert!(code.contains("fn get_by_secondary_key_from_redb"));
        assert!(code.contains("UserSecondaryKeys"));
    }

    #[test]
    fn test_generate_sled_extension_minimal() {
        let mut module = ModuleMetadata::new(
            parse_quote!(test_mod),
            parse_quote!(TestDef),
            parse_quote!(TestDefKeys)
        );

        let mut user = ModelMetadata::new(parse_quote!(User), parse_quote!(pub));
        let mut pk = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        pk.is_primary_key = true;
        user.add_field(pk);
        module.add_model(user);

        let result = generate_sled_extension(&module);
        let code = result.to_string();

        assert!(code.contains("trait SledModelAssociatedTypesExt"));
        assert!(code.contains("fn insert_model_into_sled"));
        assert!(code.contains("fn insert_hash_into_sled"));
        assert!(code.contains("fn delete_model_from_sled"));
        assert!(code.contains("TestDef :: User"));
    }

    #[test]
    fn test_generate_sled_extension_with_secondary_keys() {
        let mut module = ModuleMetadata::new(
            parse_quote!(app_mod),
            parse_quote!(AppDef),
            parse_quote!(AppDefKeys)
        );

        let mut user = ModelMetadata::new(parse_quote!(User), parse_quote!(pub));

        let mut pk = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        pk.is_primary_key = true;
        user.add_field(pk);

        let mut email = FieldMetadata::new(
            parse_quote!(email),
            parse_quote!(String),
            parse_quote!(pub)
        );
        email.is_secondary_key = true;
        user.add_field(email);

        module.add_model(user);

        let result = generate_sled_extension(&module);
        let code = result.to_string();

        assert!(code.contains("fn insert_secondary_key_into_sled"));
        assert!(code.contains("fn get_by_secondary_key_from_sled"));
        assert!(code.contains("UserSecondaryKeys"));
    }
}
