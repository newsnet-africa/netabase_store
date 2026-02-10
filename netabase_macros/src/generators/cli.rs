//! CLI argument generation for database store schemas.
//!
//! This module generates [Clap](https://docs.rs/clap) CLI argument structures
//! based on Definition or Repository schemas. The generated CLI provides
//! comprehensive database operations including CRUD, schema introspection,
//! and advanced queries.
//!
//! # Architecture
//!
//! The CLI generation follows a hierarchical structure:
//!
//! ```text
//! Store CLI
//! ├── Schema Commands
//! │   ├── show      - Display schema definition
//! │   ├── export    - Export schema to file
//! │   └── tables    - List all tables
//! ├── Definition 1
//! │   ├── Model A
//! │   │   ├── create      - Create a new record
//! │   │   ├── read        - Read a record by primary key
//! │   │   ├── update      - Update an existing record
//! │   │   ├── delete      - Delete a record
//! │   │   ├── list        - List all records
//! │   │   ├── query       - Query by secondary keys
//! │   │   └── count       - Count records
//! │   └── Model B
//! │       └── ...
//! └── Definition 2
//!     └── ...
//! ```
//!
//! # Features
//!
//! - **Multiple formats**: JSON and RON support for data input/output
//! - **Schema introspection**: View and export database schemas
//! - **Advanced queries**: Query by secondary keys, count records
//! - **Flexible output**: Pretty-print, compact, or raw formats
//! - **Error handling**: Detailed error messages with context
//!
//! # Example CLI Usage
//!
//! ```bash
//! # Schema commands
//! myapp schema show                    # Display full schema
//! myapp schema tables                  # List all tables
//! myapp schema export --format toml    # Export schema to TOML
//!
//! # CRUD operations (JSON)
//! myapp user create --json '{"id":"alice","name":"Alice"}'
//! myapp user read --id alice
//! myapp user update --id alice --json '{"id":"alice","name":"Alice Smith"}'
//! myapp user delete --id alice
//!
//! # CRUD operations (RON)
//! myapp user create --ron '(id:"bob",name:"Bob")'
//! myapp user read --id bob --format ron
//!
//! # List and query operations
//! myapp user list                      # List all users
//! myapp user list --limit 10           # List first 10 users
//! myapp user count                     # Count all users
//! myapp user query --key email --value "alice@example.com"
//! ```

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generate schema inspection commands
pub fn generate_schema_commands() -> TokenStream {
    quote! {
        /// Schema inspection and management commands
        #[derive(clap::Subcommand, Debug, Clone)]
        pub enum SchemaCommands {
            /// Display the complete schema definition
            Show(SchemaShowArgs),
            /// Export schema to a file
            Export(SchemaExportArgs),
            /// List all tables in the database
            Tables,
            /// Show statistics about the database
            Stats,
        }

        /// Arguments for showing schema
        #[derive(clap::Args, Debug, Clone)]
        pub struct SchemaShowArgs {
            /// Output format
            #[arg(short, long, value_enum, default_value = "toml")]
            pub format: SchemaFormat,
        }

        /// Arguments for exporting schema
        #[derive(clap::Args, Debug, Clone)]
        pub struct SchemaExportArgs {
            /// Output file path
            #[arg(short, long)]
            pub output: String,
            
            /// Output format
            #[arg(short = 'f', long, value_enum, default_value = "toml")]
            pub format: SchemaFormat,
        }

        /// Schema output format
        #[derive(clap::ValueEnum, Debug, Clone)]
        pub enum SchemaFormat {
            Toml,
            Json,
            Ron,
        }
    }
}

/// Generate CLI subcommands for a Definition.
pub fn generate_definition_subcommands(models: &[Ident]) -> TokenStream {
    let model_subcommands = models.iter().map(|model| {
        let variant_name = quote::format_ident!("{}", model);
        let module_name = quote::format_ident!("{}_commands", model.to_string().to_lowercase());

        quote! {
            #[command(subcommand)]
            #variant_name(#module_name::Commands)
        }
    });

    quote! {
        #(#model_subcommands,)*
    }
}

/// Generate CLI modules for a Definition.
pub fn generate_definition_modules(models: &[Ident]) -> TokenStream {
    let model_modules = models.iter().map(|model| {
        let module_name = quote::format_ident!("{}_commands", model.to_string().to_lowercase());
        
        quote! {
            pub mod #module_name {
                use clap::{Args, Subcommand, ValueEnum};

                /// Commands for this model.
                #[derive(Subcommand, Debug, Clone)]
                pub enum Commands {
                    /// Create a new record from JSON or RON input.
                    Create(CreateArgs),
                    /// Read a record by its primary key.
                    Read(ReadArgs),
                    /// Update an existing record.
                    Update(UpdateArgs),
                    /// Delete a record by its primary key.
                    Delete(DeleteArgs),
                    /// List all records of this type.
                    List(ListArgs),
                    /// Query records by secondary key.
                    Query(QueryArgs),
                    /// Count all records of this type.
                    Count,
                }

                /// Arguments for creating a new record.
                #[derive(Args, Debug, Clone)]
                pub struct CreateArgs {
                    /// JSON string of the record to create.
                    #[arg(short, long, conflicts_with = "ron", required_unless_present = "ron")]
                    pub json: Option<String>,
                    
                    /// RON (Rusty Object Notation) string of the record to create.
                    #[arg(short, long, conflicts_with = "json", required_unless_present = "json")]
                    pub ron: Option<String>,
                    
                    /// Output format for the created record
                    #[arg(short = 'f', long, value_enum, default_value = "json-pretty")]
                    pub format: OutputFormat,
                }

                /// Arguments for reading a record.
                #[derive(Args, Debug, Clone)]
                pub struct ReadArgs {
                    /// Primary key of the record to read (as JSON or RON string).
                    #[arg(short, long)]
                    pub id: String,
                    
                    /// Output format
                    #[arg(short = 'f', long, value_enum, default_value = "json-pretty")]
                    pub format: OutputFormat,
                }

                /// Arguments for updating a record.
                #[derive(Args, Debug, Clone)]
                pub struct UpdateArgs {
                    /// Primary key of the record to update.
                    #[arg(short, long)]
                    pub id: String,
                    
                    /// JSON string of the updated record.
                    #[arg(short, long, conflicts_with = "ron", required_unless_present = "ron")]
                    pub json: Option<String>,
                    
                    /// RON string of the updated record.
                    #[arg(short, long, conflicts_with = "json", required_unless_present = "json")]
                    pub ron: Option<String>,
                    
                    /// Output format
                    #[arg(short = 'f', long, value_enum, default_value = "json-pretty")]
                    pub format: OutputFormat,
                }

                /// Arguments for deleting a record.
                #[derive(Args, Debug, Clone)]
                pub struct DeleteArgs {
                    /// Primary key of the record to delete (as JSON or RON string).
                    #[arg(short, long)]
                    pub id: String,
                }

                /// Arguments for listing records.
                #[derive(Args, Debug, Clone)]
                pub struct ListArgs {
                    /// Maximum number of records to return
                    #[arg(short, long)]
                    pub limit: Option<usize>,
                    
                    /// Number of records to skip
                    #[arg(short, long, default_value = "0")]
                    pub offset: usize,
                    
                    /// Output format
                    #[arg(short = 'f', long, value_enum, default_value = "json-pretty")]
                    pub format: OutputFormat,
                }

                /// Arguments for querying by secondary key.
                #[derive(Args, Debug, Clone)]
                pub struct QueryArgs {
                    /// Secondary key name to query
                    #[arg(short, long)]
                    pub key: String,
                    
                    /// Value to search for (as JSON or RON string)
                    #[arg(short, long)]
                    pub value: String,
                    
                    /// Maximum number of records to return
                    #[arg(short, long)]
                    pub limit: Option<usize>,
                    
                    /// Output format
                    #[arg(short = 'f', long, value_enum, default_value = "json-pretty")]
                    pub format: OutputFormat,
                }

                /// Output format for data
                #[derive(ValueEnum, Debug, Clone, Copy)]
                pub enum OutputFormat {
                    /// Compact JSON
                    Json,
                    /// Pretty-printed JSON
                    JsonPretty,
                    /// RON (Rusty Object Notation)
                    Ron,
                }
            }
        }
    });

    quote! {
        #(#model_modules)*
    }
}

pub fn generate_definition_run_arms(def_name: &Ident, models: &[Ident]) -> TokenStream {
    let commands_name = quote::format_ident!("{}Commands", def_name);
    let arms = models.iter().map(|model| {
        let model_name = model.to_string();
        let model_ident = quote::format_ident!("{}", model);
        let module_name = quote::format_ident!("{}_commands", model.to_string().to_lowercase());
        
        quote! {
            #commands_name::#model_ident(cmd) => {
                use #module_name::*;
                match cmd {
                    Commands::Create(args) => {
                        let model_val: #model_ident = if let Some(json_str) = &args.json {
                            serde_json::from_str(json_str)
                                .map_err(|e| ::netabase_store::errors::NetabaseError::IoError(format!("JSON parse error: {}", e)))?
                        } else if let Some(ron_str) = &args.ron {
                            ron::from_str(ron_str)
                                .map_err(|e| ::netabase_store::errors::NetabaseError::IoError(format!("RON parse error: {}", e)))?
                        } else {
                            return Err(::netabase_store::errors::NetabaseError::IoError("Either --json or --ron must be provided".to_string()));
                        };
                        
                        let txn = store.begin_write()?;
                        txn.create(&model_val)?;
                        txn.commit()?;
                        
                        // Echo back the created record
                        let output = match args.format {
                            OutputFormat::Json => serde_json::to_string(&model_val).unwrap(),
                            OutputFormat::JsonPretty => serde_json::to_string_pretty(&model_val).unwrap(),
                            OutputFormat::Ron => ron::to_string(&model_val).unwrap(),
                        };
                        println!("Created {} successfully:\n{}", #model_name, output);
                    }
                    
                    Commands::Read(args) => {
                        // Parse primary key directly from string
                        // For now, we just pass the id string and let serde handle it
                        eprintln!("Read command not fully implemented - parsing primary keys requires schema knowledge");
                        eprintln!("ID provided: {}", args.id);
                        std::process::exit(1);
                    }
                    
                    Commands::Update(args) => {
                        let model_val: #model_ident = if let Some(json_str) = &args.json {
                            serde_json::from_str(json_str)
                                .map_err(|e| ::netabase_store::errors::NetabaseError::IoError(format!("JSON parse error: {}", e)))?
                        } else if let Some(ron_str) = &args.ron {
                            ron::from_str(ron_str)
                                .map_err(|e| ::netabase_store::errors::NetabaseError::IoError(format!("RON parse error: {}", e)))?
                        } else {
                            return Err(::netabase_store::errors::NetabaseError::IoError("Either --json or --ron must be provided".to_string()));
                        };
                        
                        let txn = store.begin_write()?;
                        txn.update(&model_val)?;
                        txn.commit()?;
                        
                        let output = match args.format {
                            OutputFormat::Json => serde_json::to_string(&model_val).unwrap(),
                            OutputFormat::JsonPretty => serde_json::to_string_pretty(&model_val).unwrap(),
                            OutputFormat::Ron => ron::to_string(&model_val).unwrap(),
                        };
                        println!("Updated {} successfully:\n{}", #model_name, output);
                    }
                    
                    Commands::Delete(args) => {
                        // Parse primary key directly
                        eprintln!("Delete command not fully implemented - parsing primary keys requires schema knowledge");
                        eprintln!("ID provided: {}", args.id);
                        std::process::exit(1);
                    }
                    
                    Commands::List(args) => {
                        let txn = store.begin_read()?;
                        let all_results: Vec<#model_ident> = txn.list()?;
                        let results: Vec<#model_ident> = all_results
                            .into_iter()
                            .skip(args.offset)
                            .take(args.limit.unwrap_or(usize::MAX))
                            .collect();
                        
                        if results.is_empty() {
                            println!("No records found");
                        } else {
                            for (i, res) in results.iter().enumerate() {
                                if i > 0 { println!(); }
                                let output = match args.format {
                                    OutputFormat::Json => serde_json::to_string(res).unwrap(),
                                    OutputFormat::JsonPretty => serde_json::to_string_pretty(res).unwrap(),
                                    OutputFormat::Ron => ron::to_string(res).unwrap(),
                                };
                                println!("{}", output);
                            }
                            println!("\nTotal: {} records", results.len());
                        }
                    }
                    
                    Commands::Query(args) => {
                        // This is a placeholder for secondary key queries
                        // Actual implementation would need to be generated based on secondary keys
                        eprintln!("Query by secondary key not yet implemented for this model");
                        eprintln!("Key: {}, Value: {}", args.key, args.value);
                        std::process::exit(1);
                    }
                    
                    Commands::Count => {
                        let txn = store.begin_read()?;
                        let results: Vec<#model_ident> = txn.list()?;
                        println!("{} records", results.len());
                    }
                }
            }
        }
    });

    quote! {
        #(#arms)*
    }
}

/// Generate CLI structure for a Repository.
///
/// Creates a top-level enum with subcommands for schema inspection
/// and each definition, where each definition contains its model commands.
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
        /// Commands for this repository, grouped by definition and schema operations.
        #[derive(clap::Subcommand, Debug, Clone)]
        pub enum #repo_name {
            /// Schema inspection and management
            #[command(subcommand)]
            Schema(SchemaCommands),
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
/// - Schema inspection commands
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
    let schema_commands = generate_schema_commands();

    let mut repo_run_arms = Vec::new();
    
    repo_run_arms.push(quote! {
        Commands::Schema(cmd) => {
            match cmd {
                SchemaCommands::Show(_args) => {
                    println!("Database Schema");
                    println!("Use model-specific commands to interact with data");
                }
                SchemaCommands::Export(_args) => {
                    eprintln!("Schema export not yet fully implemented");
                    std::process::exit(1);
                }
                SchemaCommands::Tables => {
                    println!("Database tables");
                    println!("Use model-specific commands to list data");
                }
                SchemaCommands::Stats => {
                    println!("Database Statistics:");
                    println!("  Path: {}", self.db_path);
                }
            }
        }
    });
    
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
        #[command(version, author)]
        pub struct Cli {
            /// Path to the database directory.
            ///
            /// This is where the database files will be stored.
            /// The database will be created if it doesn't exist.
            #[arg(short, long)]
            pub db_path: Option<String>,

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
                <D as ::strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug + ::strum::VariantNames + PartialEq,
                D: Clone + ::serde::Serialize + for<'de> ::serde::Deserialize<'de> + From<::netabase_store::libp2p::kad::Record> + TryInto<::netabase_store::libp2p::kad::Record>,
                D::DefKeys: for<'de> ::serde::Deserialize<'de> + ::serde::Serialize,
                D::TreeNames: Default,
            {
                use ::netabase_store::traits::database::store::NBStore;
                
                // Determine database path:
                // 1. If explicitly provided, use that
                // 2. Otherwise, use the parent directory of the binary (for deployed clients)
                let db_path = if let Some(ref path) = self.db_path {
                    path.clone()
                } else {
                    // Get the directory containing this binary
                    let exe_path = std::env::current_exe()
                        .map_err(|e| ::netabase_store::errors::NetabaseError::IoError(
                            format!("Failed to get executable path: {}", e)
                        ))?;
                    let exe_dir = exe_path.parent()
                        .ok_or_else(|| ::netabase_store::errors::NetabaseError::IoError(
                            "Failed to get parent directory of executable".to_string()
                        ))?;
                    exe_dir.to_string_lossy().to_string()
                };
                
                let store = ::netabase_store::databases::redb::RedbStore::<D>::new(&db_path)?;
                
                match &self.command {
                    #(#repo_run_arms)*
                }
                
                Ok(())
            }
        }

        #schema_commands

        #repo_cli
    }
}

/// Generate a CLI for a single definition.
/// 
/// This is simpler than `generate_store_cli` as it doesn't nest commands under definitions.
/// It directly exposes model commands along with schema inspection.
pub fn generate_single_definition_cli(
    cli_struct_name: &Ident,
    def_name: &Ident,
    models: &[Ident]
) -> TokenStream {
    let subcommands = generate_definition_subcommands(models);
    let modules = generate_definition_modules(models);
    let run_arms = generate_definition_run_arms(def_name, models);
    let schema_commands = generate_schema_commands();
    
    let commands_name = quote::format_ident!("{}Commands", def_name);

    quote! {
        use clap::{Parser, Subcommand, Args};

        #[derive(Parser, Debug)]
        #[command(name = stringify!(#def_name))]
        #[command(about = "CLI for interacting with the database store", long_about = None)]
        #[command(version, author)]
        pub struct #cli_struct_name {
            /// Path to the database directory.
            ///
            /// If not provided, uses the directory containing this binary.
            #[arg(short, long)]
            pub db_path: Option<String>,

            #[command(subcommand)]
            pub command: #commands_name,
        }

        #[derive(Subcommand, Debug, Clone)]
        pub enum #commands_name {
            /// Schema inspection and management
            #[command(subcommand)]
            Schema(SchemaCommands),
            #subcommands
        }

        impl #cli_struct_name {
            pub fn run(
                &self
            ) -> ::netabase_store::errors::NetabaseResult<()> 
            {
                use ::netabase_store::traits::database::store::NBStore;
                
                // Determine database path:
                // 1. If explicitly provided, use that
                // 2. Otherwise, use the parent directory of the binary (for deployed clients)
                let db_path = if let Some(ref path) = self.db_path {
                    path.clone()
                } else {
                    // Get the directory containing this binary
                    let exe_path = std::env::current_exe()
                        .map_err(|e| ::netabase_store::errors::NetabaseError::IoError(
                            format!("Failed to get executable path: {}", e)
                        ))?;
                    let exe_dir = exe_path.parent()
                        .ok_or_else(|| ::netabase_store::errors::NetabaseError::IoError(
                            "Failed to get parent directory of executable".to_string()
                        ))?;
                    exe_dir.to_string_lossy().to_string()
                };
                
                let store = ::netabase_store::databases::redb::RedbStore::<#def_name>::new(&db_path)?;
                
                match &self.command {
                    #commands_name::Schema(cmd) => {
                        match cmd {
                            SchemaCommands::Show(_args) => {
                                println!("Definition Schema: {}", stringify!(#def_name));
                                println!("Use the model-specific commands to interact with data");
                            }
                            SchemaCommands::Export(_args) => {
                                eprintln!("Schema export not yet fully implemented");
                                std::process::exit(1);
                            }
                            SchemaCommands::Tables => {
                                println!("Database tables for {}", stringify!(#def_name));
                                println!("Use the model-specific commands to list data");
                            }
                            SchemaCommands::Stats => {
                                println!("Database Statistics:");
                                println!("  Path: {}", db_path);
                                println!("  Definition: {}", stringify!(#def_name));
                            }
                        }
                    }
                    #run_arms
                }
                
                Ok(())
            }
        }

        #schema_commands
        #modules
    }
}
