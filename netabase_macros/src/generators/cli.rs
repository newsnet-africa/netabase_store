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

/// Generate CLI subcommands for a Definition.
pub fn generate_definition_subcommands(models: &[Ident]) -> TokenStream {
    let model_subcommands = models.iter().map(|model| {
        let variant_name = quote::format_ident!("{}", model);

        quote! {
            #[command(subcommand)]
            #variant_name(#model::Commands)
        }
    });

    quote! {
        #(#model_subcommands,)*
    }
}

/// Generate CLI modules for a Definition.
pub fn generate_definition_modules(models: &[Ident]) -> TokenStream {
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
        #(#model_modules)*
    }
}

pub fn generate_definition_run_arms(def_name: &Ident, models: &[Ident]) -> TokenStream {
    let arms = models.iter().map(|model| {
        let model_name = model.to_string();
        let model_ident = quote::format_ident!("{}", model);
        
        quote! {
            #model_ident::Commands::Create(args) => {
                let model_val: #def_name = serde_json::from_str(&args.json)
                    .map_err(|e| ::netabase_store::errors::NetabaseError::IoError(format!("JSON error: {}", e)))?;
                let txn = store.begin_write()?;
                txn.create(&model_val)?;
                txn.commit()?;
                println!("Created {} successfully.", #model_name);
            }
            #model_ident::Commands::Read(args) => {
                let key: <#def_name as ::netabase_store::traits::registry::definition::NetabaseDefinition>::DefKeys = serde_json::from_str(&args.id)
                    .map_err(|e| ::netabase_store::errors::NetabaseError::IoError(format!("JSON error: {}", e)))?;
                let txn = store.begin_read()?;
                let result = txn.read(&key)?;
                if let Some(val) = result {
                    println!("{}", serde_json::to_string_pretty(&val).unwrap());
                } else {
                    println!("Not found.");
                }
            }
            #model_ident::Commands::Update(args) => {
                let model_val: #def_name = serde_json::from_str(&args.json)
                    .map_err(|e| ::netabase_store::errors::NetabaseError::IoError(format!("JSON error: {}", e)))?;
                let txn = store.begin_write()?;
                txn.update(&model_val)?;
                txn.commit()?;
                println!("Updated {} successfully.", #model_name);
            }
            #model_ident::Commands::Delete(args) => {
                let key: <#def_name as ::netabase_store::traits::registry::definition::NetabaseDefinition>::DefKeys = serde_json::from_str(&args.id)
                    .map_err(|e| ::netabase_store::errors::NetabaseError::IoError(format!("JSON error: {}", e)))?;
                let txn = store.begin_write()?;
                txn.delete(&key)?;
                txn.commit()?;
                println!("Deleted {} successfully.", #model_name);
            }
            #model_ident::Commands::List => {
                let txn = store.begin_read()?;
                // Use D::Discriminant to list
                let discriminant = <#def_name as ::strum::IntoDiscriminant>::Discriminant::#model_ident;
                let results = txn.list(discriminant)?;
                for res in results {
                    println!("{}", serde_json::to_string_pretty(&res).unwrap());
                }
            }
        }
    });

    quote! {
        #(#arms)*
        _ => {
            // For nested definitions or empty ones
            return Err(::netabase_store::errors::NetabaseError::Other);
        }
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
        .map(|(def_name, models)| {
            let subcommands = generate_definition_subcommands(models);
            let modules = generate_definition_modules(models);
            quote! {
                #[derive(clap::Subcommand, Debug, Clone)]
                pub enum #def_name {
                    #subcommands
                }
                #modules
            }
        });

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
pub fn generate_store_cli(store_name: &Ident, definitions: &[(Ident, Vec<Ident>)]) -> TokenStream {
    let repo_cli =
        generate_repository_cli(&quote::format_ident!("{}Commands", store_name), definitions);

    let mut repo_run_arms = Vec::new();
    
    // For each definition, generate match arms for its models
    for (def_name, models) in definitions {
        let def_ident = quote::format_ident!("{}", def_name);
        let model_arms = generate_definition_run_arms(&def_ident, models);
        
        repo_run_arms.push(quote! {
            Commands::#def_ident(cmd) => {
                match cmd {
                    #model_arms
                }
            }
        });
    }

    quote! {
        use clap::{Parser, Subcommand, Args};

        /// Main CLI entry point for the database store.
        #[derive(Parser, Debug)]
        #[command(name = stringify!(#store_name))]
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

        impl Cli {
            /// Execute the parsed CLI command.
            pub fn run<D: ::netabase_store::traits::registry::definition::NetabaseDefinition + ::netabase_store::traits::registry::definition::redb_definition::RedbDefinition + 'static>(
                &self
            ) -> ::netabase_store::errors::NetabaseResult<()> 
            where
                <D as ::strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug + ::strum::VariantNames,
                D: Clone + ::serde::Serialize + for<'de> ::serde::Deserialize<'de> + From<::netabase_store::libp2p::kad::Record> + TryInto<::netabase_store::libp2p::kad::Record>,
                D::DefKeys: for<'de> ::serde::Deserialize<'de> + ::serde::Serialize,
                D::TreeNames: Default,
            {
                use ::netabase_store::traits::database::store::NBStore;
                
                let store = ::netabase_store::databases::redb::RedbStore::<D>::new(&self.db_path)?;
                
                match &self.command {
                    #(#repo_run_arms)*
                }
                
                Ok(())
            }
        }

        #repo_cli

        /// Type alias for the top-level commands enum.
        pub type Commands = #store_name;
    }
}

/// Generate a CLI for a single definition.
/// 
/// This is simpler than `generate_store_cli` as it doesn't nest commands under definitions.
/// It directly exposes model commands.
pub fn generate_single_definition_cli(
    cli_struct_name: &Ident,
    def_name: &Ident,
    models: &[Ident]
) -> TokenStream {
    let subcommands = generate_definition_subcommands(models);
    let modules = generate_definition_modules(models);
    let run_arms = generate_definition_run_arms(def_name, models);
    
    let commands_name = quote::format_ident!("{}Commands", def_name);

    quote! {
        use clap::{Parser, Subcommand, Args};

        #[derive(Parser, Debug)]
        #[command(name = stringify!(#def_name))]
        #[command(about = "CLI for interacting with the database store", long_about = None)]
        pub struct #cli_struct_name {
            /// Path to the database directory.
            #[arg(short, long, default_value = "./database")]
            pub db_path: String,

            #[command(subcommand)]
            pub command: #commands_name,
        }

        #[derive(Subcommand, Debug, Clone)]
        pub enum #commands_name {
            #subcommands
        }

        impl #cli_struct_name {
            pub fn run<D: ::netabase_store::traits::registry::definition::NetabaseDefinition + ::netabase_store::traits::registry::definition::redb_definition::RedbDefinition + 'static>(
                &self
            ) -> ::netabase_store::errors::NetabaseResult<()> 
            where
                <D as ::strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug + ::strum::VariantNames,
                D: Clone + ::serde::Serialize + for<'de> ::serde::Deserialize<'de> + From<::netabase_store::libp2p::kad::Record> + TryInto<::netabase_store::libp2p::kad::Record>,
                D::DefKeys: for<'de> ::serde::Deserialize<'de> + ::serde::Serialize,
                D::TreeNames: Default,
            {
                use ::netabase_store::traits::database::store::NBStore;
                
                let store = ::netabase_store::databases::redb::RedbStore::<D>::new(&self.db_path)?;
                
                match &self.command {
                    #run_arms
                }
                
                Ok(())
            }
        }

        #modules
    }
}
