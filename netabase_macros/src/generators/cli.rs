//! CLI argument generation for database store schemas.
//!
//! This module generates [Clap](https://docs.rs/clap) CLI argument structures
//! based on Definition or Repository schemas. The generated CLI provides
//! CRUD (Create, Read, Update, Delete) operations for each model.
//!
//! # Architecture
//!
//! The CLI generation follows a hierarchical structure:
//!
//! ```text
//! Store CLI
//! ├── Definition 1
//! │   ├── Model A
//! │   │   ├── create
//! │   │   ├── read
//! │   │   ├── update
//! │   │   ├── delete
//! │   │   └── list
//! │   └── Model B
//! │       └── ...
//! └── Definition 2
//!     └── ...
//! ```
//!
//! # Usage
//!
//! The CLI can be generated from a schema file:
//!
//! ```rust,ignore
//! // In your build.rs or main.rs
//! netabase_macros::generate_cli!("schema.toml");
//!
//! fn main() {
//!     let cli = Cli::parse();
//!     
//!     match cli.command {
//!         Commands::User(cmd) => match cmd {
//!             User::Commands::Create(args) => {
//!                 let user: User = serde_json::from_str(&args.json)?;
//!                 // Store the user...
//!             }
//!             User::Commands::Read(args) => {
//!                 // Read user by args.id...
//!             }
//!             // ...
//!         }
//!     }
//! }
//! ```
//!
//! # Generated Commands
//!
//! For each model, the following commands are generated:
//!
//! | Command | Description | Arguments |
//! |---------|-------------|-----------|
//! | `create` | Create a new record | `--json <JSON>` |
//! | `read` | Read a record by ID | `--id <ID>` |
//! | `update` | Update an existing record | `--id <ID> --json <JSON>` |
//! | `delete` | Delete a record | `--id <ID>` |
//! | `list` | List all records | (none) |
//!
//! # Example CLI Usage
//!
//! ```bash
//! # Create a user
//! myapp user create --json '{"id":"alice","name":"Alice","email":"alice@example.com"}'
//!
//! # Read a user
//! myapp user read --id alice
//!
//! # Update a user
//! myapp user update --id alice --json '{"id":"alice","name":"Alice Smith","email":"alice@example.com"}'
//!
//! # Delete a user
//! myapp user delete --id alice
//!
//! # List all users
//! myapp user list
//! ```
//!
//! # Status
//!
//! The CLI generation is currently in development. The basic structure is in place
//! but integration with the store operations is not yet complete.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generate CLI commands for a Definition.
///
/// Creates a module with subcommands for each model in the definition.
/// Each model gets CRUD operations (Create, Read, Update, Delete, List).
///
/// # Arguments
///
/// * `_def_name` - The definition name (currently unused but reserved for future use)
/// * `models` - List of model names to generate commands for
///
/// # Returns
///
/// TokenStream containing the CLI enum variants and model command modules.
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

                /// Commands for this model.
                #[derive(Subcommand, Debug, Clone)]
                pub enum Commands {
                    /// Create a new record from JSON input.
                    Create(CreateArgs),
                    /// Read a record by its primary key.
                    Read(ReadArgs),
                    /// Update an existing record.
                    Update(UpdateArgs),
                    /// Delete a record by its primary key.
                    Delete(DeleteArgs),
                    /// List all records of this type.
                    List,
                }

                /// Arguments for creating a new record.
                #[derive(Args, Debug, Clone)]
                pub struct CreateArgs {
                    /// JSON string of the record to create.
                    ///
                    /// The JSON must match the model's structure exactly.
                    #[arg(short, long)]
                    pub json: String,
                }

                /// Arguments for reading a record.
                #[derive(Args, Debug, Clone)]
                pub struct ReadArgs {
                    /// Primary key of the record to read.
                    #[arg(short, long)]
                    pub id: String,
                }

                /// Arguments for updating a record.
                #[derive(Args, Debug, Clone)]
                pub struct UpdateArgs {
                    /// Primary key of the record to update.
                    #[arg(short, long)]
                    pub id: String,
                    /// JSON string of the updated record.
                    ///
                    /// The JSON must contain the complete record, not just changed fields.
                    #[arg(short, long)]
                    pub json: String,
                }

                /// Arguments for deleting a record.
                #[derive(Args, Debug, Clone)]
                pub struct DeleteArgs {
                    /// Primary key of the record to delete.
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

/// Generate CLI structure for a Repository.
///
/// Creates a top-level enum with subcommands for each definition,
/// where each definition contains its model commands.
///
/// # Arguments
///
/// * `repo_name` - Name for the repository commands enum
/// * `definitions` - List of (definition_name, model_names) tuples
///
/// # Returns
///
/// TokenStream containing the repository CLI enum and all nested command structures.
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
        /// Commands for this repository, grouped by definition.
        #[derive(clap::Subcommand, Debug, Clone)]
        pub enum #repo_name {
            #(#def_subcommands,)*
        }

        #(#definition_cli_gens)*
    }
}

/// Generate main CLI structure for a store.
///
/// Creates the complete CLI application with:
/// - A `Cli` struct as the entry point
/// - Database path configuration
/// - All definition and model commands
///
/// # Arguments
///
/// * `store_name` - Name for the store (used in CLI name and help text)
/// * `definitions` - List of (definition_name, model_names) tuples
///
/// # Returns
///
/// TokenStream containing the complete CLI structure.
///
/// # Example Generated Code
///
/// ```rust,ignore
/// use clap::{Parser, Subcommand, Args};
///
/// #[derive(Parser, Debug)]
/// #[command(name = "MyStore")]
/// #[command(about = "CLI for interacting with the database store")]
/// pub struct Cli {
///     #[arg(short, long, default_value = "./database")]
///     pub db_path: String,
///
///     #[command(subcommand)]
///     pub command: Commands,
/// }
///
/// #[derive(Subcommand, Debug, Clone)]
/// pub enum Commands {
///     User(user::Commands),
///     Product(product::Commands),
/// }
/// ```
pub fn generate_store_cli(store_name: &Ident, definitions: &[(Ident, Vec<Ident>)]) -> TokenStream {
    let repo_cli =
        generate_repository_cli(&quote::format_ident!("{}Commands", store_name), definitions);

    quote! {
        use clap::{Parser, Subcommand, Args};

        /// Main CLI entry point for the database store.
        #[derive(Parser, Debug)]
        #[command(name = #store_name)]
        #[command(about = "CLI for interacting with the database store", long_about = None)]
        pub struct Cli {
            /// Path to the database directory.
            ///
            /// This is where the database files will be stored.
            /// Defaults to "./database" in the current directory.
            #[arg(short, long, default_value = "./database")]
            pub db_path: String,

            /// The command to execute.
            #[command(subcommand)]
            pub command: Commands,
        }

        #repo_cli

        /// Type alias for the top-level commands enum.
        pub type Commands = #store_name;
    }
}
