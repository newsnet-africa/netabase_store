//! Core trait system for Netabase.
//!
//! This module defines the foundational traits that power the Netabase storage system.
//! These traits provide abstraction layers for database operations, migrations, and
//! type registration.
//!
//! # Module Organization
//!
//! ## `database` - Storage Abstractions
//!
//! Core traits for database backends:
//! - `NBStore` - Database store lifecycle (open, transactions)
//! - `NBTransaction` - Transaction operations (read, write, commit)
//! - `NBHash` - Content hashing strategies
//!
//! ## `registry` - Type System
//!
//! Compile-time type registration:
//! - `models` - Model trait definitions and key types
//! - `definition` - Definition (schema) trait definitions  
//! - `repository` - Repository isolation and access control
//!
//! ## `migration` - Schema Evolution (feature-gated)
//!
//! Version management and data migration:
//! - `MigrateFrom` - Upgrade from older versions
//! - `MigrateTo` - Downgrade for P2P compatibility
//! - `VersionContext` - Version-aware deserialization
//!
//! ## `libp2p` - Networking (feature-gated)
//!
//! P2P networking trait integrations.
//!
//! # Design Philosophy
//!
//! Traits are organized by concern:
//! - **Storage traits** abstract the database backend
//! - **Registry traits** define the type system
//! - **Migration traits** handle schema evolution
//!
//! This separation allows:
//! - Multiple backend implementations (redb, memory, indexeddb)
//! - Compile-time schema validation
//! - Feature-gated functionality

pub mod database;
#[cfg(feature = "migration")]
pub mod migration;
pub mod registry;
#[cfg(feature = "libp2p")]
pub mod libp2p;
#[cfg(feature = "libp2p")]
pub mod node_metadata;
