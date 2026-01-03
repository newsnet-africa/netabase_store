//! Schema TOML and hash generation for repositories.
//!
//! Generates methods for exporting repository schemas as TOML strings
//! and computing schema hashes for P2P node comparison.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::visitors::repository::RepositoryVisitor;

/// Generator for repository schema methods
pub struct SchemaGenerator<'a> {
    visitor: &'a RepositoryVisitor,
}

impl<'a> SchemaGenerator<'a> {
    pub fn new(visitor: &'a RepositoryVisitor) -> Self {
        Self { visitor }
    }

    /// Generate schema-related code
    pub fn generate(&self) -> TokenStream {
        let schema_impl = self.generate_schema_impl();
        let migration_metadata = self.generate_migration_metadata();

        quote! {
            #migration_metadata
            #schema_impl
        }
    }

    /// Generate migration metadata struct for this repository
    fn generate_migration_metadata(&self) -> TokenStream {
        let repo_name = &self.visitor.repository_name;
        let metadata_name = format_ident!("{}MigrationMetadata", repo_name);

        quote! {
            /// Migration metadata for schema changes.
            ///
            /// This struct captures information about field renames, type changes,
            /// and added/removed fields for future migration support.
            #[derive(Debug, Clone, Default)]
            pub struct #metadata_name {
                /// Field renames: (old_name, new_name, definition, model)
                pub field_renames: Vec<(String, String, String, String)>,
                /// Type changes: (field_name, old_type, new_type, definition, model)
                pub type_changes: Vec<(String, String, String, String, String)>,
                /// Added fields: (field_name, field_type, definition, model)
                pub added_fields: Vec<(String, String, String, String)>,
                /// Removed fields: (field_name, field_type, definition, model)
                pub removed_fields: Vec<(String, String, String, String)>,
            }
        }
    }

    /// Generate schema implementation methods
    fn generate_schema_impl(&self) -> TokenStream {
        let repo_name = &self.visitor.repository_name;
        let metadata_name = format_ident!("{}MigrationMetadata", repo_name);

        // Check if using external definitions (need super:: prefix)
        let is_external = !self.visitor.external_definitions.is_empty();

        // Collect definition names for schema generation
        let def_schema_calls: Vec<_> = self.visitor.definitions
            .iter()
            .map(|def| {
                let def_name = &def.name;
                // Use super:: prefix for external definitions
                let def_path = if is_external {
                    quote! { super::#def_name }
                } else {
                    quote! { #def_name }
                };
                quote! {
                    {
                        let def_schema = <#def_path as netabase_store::traits::registery::definition::NetabaseDefinition>::schema();
                        schemas.push((stringify!(#def_name).to_string(), def_schema));
                    }
                }
            })
            .collect();

        quote! {
            impl #repo_name {
                /// Export the repository schema as a TOML string.
                ///
                /// This aggregates all definition schemas within the repository
                /// and includes migration hint metadata.
                pub fn schema_toml() -> String {
                    use netabase_store::traits::registery::definition::NetabaseDefinition;

                    let mut schemas: Vec<(String, netabase_store::traits::registery::definition::schema::DefinitionSchema)> = Vec::new();
                    #(#def_schema_calls)*

                    // Build repository-level TOML
                    let mut toml_content = format!("[repository]\nname = \"{}\"\n\n", stringify!(#repo_name));

                    for (def_name, schema) in schemas {
                        toml_content.push_str(&format!("[[definitions]]\nname = \"{}\"\n", def_name));
                        // Export each definition's schema
                        toml_content.push_str(&schema.to_toml());
                        toml_content.push('\n');
                    }

                    toml_content
                }

                /// Compute the schema hash using the specified hash algorithm.
                ///
                /// Use `FastHash` for local comparisons, `CryptoHash` for P2P security.
                pub fn schema_hash<H: netabase_store::traits::database::hash::HashAlgorithm>() -> u64 {
                    let toml = Self::schema_toml();
                    H::hash_string(&toml)
                }

                /// Get migration metadata for this repository.
                ///
                /// This is a placeholder that returns empty metadata.
                /// Future versions will populate this from schema history.
                pub fn migration_metadata() -> #metadata_name {
                    #metadata_name::default()
                }

                /// Compare this repository's schema with a remote schema hash.
                ///
                /// Returns `true` if schemas match, `false` otherwise.
                pub fn schemas_match<H: netabase_store::traits::database::hash::HashAlgorithm>(remote_hash: u64) -> bool {
                    Self::schema_hash::<H>() == remote_hash
                }
            }
        }
    }
}
