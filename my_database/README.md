# ExampleDef Database

This folder contains a complete Netabase database with its CLI client.

## Contents

- **data.redb** - The main database file containing all stored data
- **schema.toml** - The database schema definition
- **client** - CLI executable for interacting with the database (if exported)

## Usage

The client binary provides a command-line interface for all database operations.

### Basic Commands

```bash
# Show help
./client --help

# Specify database path
./client --db-path ./ <command>
```

### Model Operations

Each model in the schema has CRUD operations available:

```bash
# Create a record
./client <model> create --json '{...}'

# Read a record by ID
./client <model> read --id <id>

# Update a record
./client <model> update --id <id> --json '{...}'

# Delete a record
./client <model> delete --id <id>

# List all records
./client <model> list
```

## Schema

The database schema is defined in `schema.toml`. You can view it to see:
- Available models and their fields
- Field types and constraints
- Relationships between models

## Shipping the Database

This entire folder is self-contained and can be shipped as-is. Recipients need only:
1. The database folder with all files
2. Execute permissions on the `client` binary (already set on Unix systems)

## Development

This database was generated using Netabase Store.
