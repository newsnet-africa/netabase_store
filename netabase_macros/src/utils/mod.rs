//! Utility modules for the macro system.
//!
//! This module provides shared utilities used across the macro crate.
//!
//! # Module Structure
//!
//! - [`attributes`]: Attribute parsing helpers (`#[primary_key]`, `#[link]`, etc.)
//! - [`naming`]: Identifier naming conventions and transformations
//! - [`schema`]: Schema representation utilities
//! - [`errors`]: Error formatting and diagnostic helpers
//!
//! # Naming Conventions
//!
//! The [`naming`] module defines how generated identifiers are named:
//!
//! - `{Model}ID` - Primary key type
//! - `{Model}SecondaryKeys` - Secondary key enum
//! - `{Model}RelationalKeys` - Relational key enum
//! - `{Definition}TreeNames` - Table name enum
//! - `{Definition}Keys` - Unified key enum
//!
//! # Attribute Parsing
//!
//! The [`attributes`] module handles parsing of:
//!
//! - `#[primary_key]` - Marks primary key field
//! - `#[secondary_key]` - Marks indexed field
//! - `#[link(Def, Model)]` - Marks relational link
//! - `#[blob]` - Marks blob field
//! - `#[subscribe(Topic)]` - Marks subscription
//! - `#[netabase_version(...)]` - Version metadata

pub mod attributes;
pub mod errors;
pub mod naming;
pub mod schema;
