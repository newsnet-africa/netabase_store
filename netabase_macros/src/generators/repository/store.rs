//! Repository store trait generator.
//!
//! Generates implementations of `RedbRepositoryDefinitions` for repositories,
//! enabling multi-definition store management.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::visitors::repository::RepositoryVisitor;

/// Generator for repository store trait implementations
pub struct StoreGenerator<'a> {
    visitor: &'a RepositoryVisitor,
}

impl<'a> StoreGenerator<'a> {
    pub fn new(visitor: &'a RepositoryVisitor) -> Self {
        Self { visitor }
    }

    /// Generate all store-related trait implementations
    pub fn generate(&self) -> TokenStream {
        let redb_repository_definitions_impl = self.generate_redb_repository_definitions_impl();
        let definition_stores_struct = self.generate_definition_stores_struct();

        quote! {
            #definition_stores_struct
            #redb_repository_definitions_impl
        }
    }

    /// Generate `RedbRepositoryDefinitions` trait implementation
    ///
    /// ```rust,ignore
    /// impl netabase_store::databases::redb::repository::RedbRepositoryDefinitions for MyRepo {
    ///     fn definition_names() -> &'static [&'static str] {
    ///         &["Employee", "Inventory"]
    ///     }
    ///
    ///     fn init_definition_stores(repo_path: &std::path::Path) -> netabase_store::errors::NetabaseResult<()> {
    ///         // Create each definition's folder and initialize
    ///         Employee::init_store(&repo_path.join("Employee"))?;
    ///         Inventory::init_store(&repo_path.join("Inventory"))?;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn generate_redb_repository_definitions_impl(&self) -> TokenStream {
        let repo_name = &self.visitor.repository_name;

        // Collect definition names as static strings
        let def_names: Vec<_> = self
            .visitor
            .definitions
            .iter()
            .map(|d| d.name.to_string())
            .collect();

        // Check if using external definitions (need super:: prefix)
        let is_external = !self.visitor.external_definitions.is_empty();

        // Generate init calls for each definition
        let init_calls: Vec<_> = self
            .visitor
            .definitions
            .iter()
            .map(|def| {
                let def_name = &def.name;
                let def_name_str = def.name.to_string();
                // Use super:: prefix for external definitions
                let def_path = if is_external {
                    quote! { super::#def_name }
                } else {
                    quote! { #def_name }
                };
                quote! {
                    {
                        let def_path_folder = repo_path.join(#def_name_str);

                        // Create the definition folder
                        if !def_path_folder.exists() {
                            std::fs::create_dir_all(&def_path_folder).map_err(|e| {
                                netabase_store::errors::NetabaseError::IoError(format!(
                                    "Failed to create definition folder {:?}: {}",
                                    def_path_folder, e
                                ))
                            })?;
                        }

                        // Create database file
                        let db_path = def_path_folder.join("data.redb");
                        let db = redb::Database::create(&db_path)
                            .map_err(|e| netabase_store::errors::NetabaseError::RedbError(e.into()))?;

                        // Initialize tables for this definition
                        <#def_path as netabase_store::traits::registery::definition::redb_definition::RedbDefinition>::init_tables(&db)?;

                        // Write schema file
                        let schema_path = def_path_folder.join("schema.toml");
                        let toml = <#def_path as netabase_store::traits::registery::definition::NetabaseDefinition>::export_toml();
                        std::fs::write(&schema_path, &toml).map_err(|e| {
                            netabase_store::errors::NetabaseError::IoError(format!(
                                "Failed to write schema file {:?}: {}",
                                schema_path, e
                            ))
                        })?;
                    }
                }
            })
            .collect();

        quote! {
            impl netabase_store::databases::redb::repository::RedbRepositoryDefinitions for #repo_name {
                fn definition_names() -> &'static [&'static str] {
                    &[#(#def_names),*]
                }

                fn init_definition_stores(repo_path: &std::path::Path) -> netabase_store::errors::NetabaseResult<()> {
                    #(#init_calls)*
                    Ok(())
                }
            }
        }
    }

    /// Generate a type-safe struct holding all definition stores
    ///
    /// ```rust,ignore
    /// pub struct MyRepoStores {
    ///     pub employee: netabase_store::databases::redb::RedbStore<Employee>,
    ///     pub inventory: netabase_store::databases::redb::RedbStore<Inventory>,
    /// }
    /// ```
    fn generate_definition_stores_struct(&self) -> TokenStream {
        let repo_name = &self.visitor.repository_name;
        let stores_struct_name = format_ident!("{}Stores", repo_name);

        // Check if using external definitions (need super:: prefix)
        let is_external = !self.visitor.external_definitions.is_empty();

        // Generate field names (snake_case) and types
        let fields: Vec<_> = self
            .visitor
            .definitions
            .iter()
            .map(|def| {
                let def_name = &def.name;
                let field_name = format_ident!("{}", to_snake_case(&def.name.to_string()));
                // Use super:: prefix for external definitions
                let def_path = if is_external {
                    quote! { super::#def_name }
                } else {
                    quote! { #def_name }
                };
                quote! {
                    pub #field_name: netabase_store::databases::redb::RedbStore<#def_path>
                }
            })
            .collect();

        // Generate constructor logic
        let field_inits: Vec<_> = self
            .visitor
            .definitions
            .iter()
            .map(|def| {
                let def_name_str = def.name.to_string();
                let field_name = format_ident!("{}", to_snake_case(&def.name.to_string()));
                quote! {
                    #field_name: netabase_store::databases::redb::RedbStore::new(
                        repo_path.join(#def_name_str)
                    )?
                }
            })
            .collect();

        let field_names: Vec<_> = self
            .visitor
            .definitions
            .iter()
            .map(|def| format_ident!("{}", to_snake_case(&def.name.to_string())))
            .collect();

        quote! {
            /// Type-safe container for all definition stores in this repository.
            ///
            /// This provides direct access to each definition's store for
            /// fine-grained transaction control.
            #[derive(Clone)]
            pub struct #stores_struct_name {
                #(#fields),*
            }

            impl #stores_struct_name {
                /// Create all definition stores from the repository path.
                ///
                /// This opens or creates each definition's database and schema files.
                pub fn new<P: AsRef<std::path::Path>>(repo_path: P) -> netabase_store::errors::NetabaseResult<Self> {
                    use netabase_store::traits::database::store::NBStore;
                    let repo_path = repo_path.as_ref();

                    // Ensure repository folder exists
                    if !repo_path.exists() {
                        std::fs::create_dir_all(repo_path).map_err(|e| {
                            netabase_store::errors::NetabaseError::IoError(format!(
                                "Failed to create repository folder {:?}: {}",
                                repo_path, e
                            ))
                        })?;
                    }

                    Ok(Self {
                        #(#field_inits),*
                    })
                }

                /// Get a list of all store field names.
                pub fn store_names() -> &'static [&'static str] {
                    &[#(stringify!(#field_names)),*]
                }
            }
        }
    }
}

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}
