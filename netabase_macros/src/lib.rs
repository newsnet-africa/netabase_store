#![feature(phantom_variance_markers)]
pub(crate) mod core;
pub(crate) mod generators;
pub(crate) mod visitors;

use proc_macro::TokenStream;

use crate::generators::key::{KeyGenerator, KeyPlan};
use crate::generators::model::{ModelGenerator, ModelPlan};
use crate::visitors::model::key::KeyVisitor;
use crate::visitors::model::ModelVisitor;

proc_macro_flow::define_derive! {
    name: NetabaseModel,
    helpers: [primary_key, secondary_key, foreign_key, blob, subscribe, version],
    visitor: ModelVisitor,
    planner: ModelPlan,
    generator: ModelGenerator,
    with_lifetime
}

proc_macro_flow::define_derive! {
    name: NetabaseKey,
    helpers: [key],
    visitor: KeyVisitor,
    planner: KeyPlan,
    generator: KeyGenerator,
}

#[proc_macro_attribute]
pub fn netabase_definition(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}

proc_macro_flow::define_derive! {
    name: NetabaseBlob,
    helpers: [
        blob,
        blob_field,
        chunk_scope,
        chunk_size,
        chunk_derives,
        chunk_serialize,
        chunk_deserialize,
        chunk_owner_id,
        chunk_checksum,
        strategy
    ],
    visitor: visitors::blob::BlobVisitor,
    planner: generators::blob::BlobPlan,
    generator: generators::blob::BlobGenerator,
}

#[proc_macro_derive(NetabaseRepository, attributes(primary_key))]
pub fn netabase_repository(input: TokenStream) -> TokenStream {
    input
}
