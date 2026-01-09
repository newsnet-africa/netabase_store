//! CLI argument generation for Store schemas
//!
//! This module generates Clap CLI argument structures based on Definition or Repository schemas.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generate CLI commands for a Definition
pub fn generate_definition_cli(_def_name: &Ident, models: &[Ident]) -> TokenStream {
    let model_subcommands = models.iter().map(|model| {
        let variant_name = quote::format_ident!("{}", model);

        quote! {
            #[command(subcommand)]
            #variant_name(#model::Commands)
        }
    });

    let model_modules = models.iter().map(|model| {
        quote! {
            pub mod #model {
                use clap::{Args, Subcommand};

                #[derive(Subcommand, Debug, Clone)]
                pub enum Commands {
                    /// Create a new record
                    Create(CreateArgs),
                    /// Read a record by ID
                    Read(ReadArgs),
                    /// Update a record
                    Update(UpdateArgs),
                    /// Delete a record
                    Delete(DeleteArgs),
                    /// List all records
                    List,
                }

                #[derive(Args, Debug, Clone)]
                pub struct CreateArgs {
                    /// JSON string of the record to create
                    #[arg(short, long)]
                    pub json: String,
                }

                #[derive(Args, Debug, Clone)]
                pub struct ReadArgs {
                    /// Primary key of the record to read
                    #[arg(short, long)]
                    pub id: String,
                }

                #[derive(Args, Debug, Clone)]
                pub struct UpdateArgs {
                    /// Primary key of the record to update
                    #[arg(short, long)]
                    pub id: String,
                    /// JSON string of the updated record
                    #[arg(short, long)]
                    pub json: String,
                }

                #[derive(Args, Debug, Clone)]
                pub struct DeleteArgs {
                    /// Primary key of the record to delete
                    #[arg(short, long)]
                    pub id: String,
                }
            }
        }
    });

    quote! {
        #(#model_subcommands,)*

        #(#model_modules)*
    }
}

/// Generate CLI structure for a Repository
pub fn generate_repository_cli(
    repo_name: &Ident,
    definitions: &[(Ident, Vec<Ident>)],
) -> TokenStream {
    let def_subcommands = definitions.iter().map(|(def_name, _models)| {
        let def_lower = def_name.to_string().to_lowercase();

        quote! {
            #[command(name = #def_lower, subcommand)]
            #def_name(#def_name)
        }
    });

    let definition_cli_gens = definitions
        .iter()
        .map(|(def_name, models)| generate_definition_cli(def_name, models));

    quote! {
        #[derive(clap::Subcommand, Debug, Clone)]
        pub enum #repo_name {
            #(#def_subcommands,)*
        }

        #(#definition_cli_gens)*
    }
}

/// Generate main CLI structure for a store
pub fn generate_store_cli(store_name: &Ident, definitions: &[(Ident, Vec<Ident>)]) -> TokenStream {
    let repo_cli =
        generate_repository_cli(&quote::format_ident!("{}Commands", store_name), definitions);

    quote! {
        use clap::{Parser, Subcommand, Args};

        #[derive(Parser, Debug)]
        #[command(name = #store_name)]
        #[command(about = "CLI for interacting with the database store", long_about = None)]
        pub struct Cli {
            /// Database path
            #[arg(short, long, default_value = "./database")]
            pub db_path: String,

            #[command(subcommand)]
            pub command: Commands,
        }

        #repo_cli

        pub type Commands = #store_name;
    }
}
