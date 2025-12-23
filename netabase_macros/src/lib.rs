use proc_macro::TokenStream;

mod utils;
mod visitors;
mod generators;
mod macros;

#[proc_macro_derive(NetabaseModel, attributes(primary_key, secondary_key, relation, blob, subscribe))]
pub fn netabase_model(input: TokenStream) -> TokenStream {
    macros::netabase_model::netabase_model_derive(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn netabase_definition(attr: TokenStream, item: TokenStream) -> TokenStream {
    macros::netabase_definition::netabase_definition_attribute(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn netabase(attr: TokenStream, item: TokenStream) -> TokenStream {
    macros::netabase::netabase_attribute(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(NetabaseBlobItem)]
pub fn netabase_blob_item(input: TokenStream) -> TokenStream {
    macros::netabase_blob_item::netabase_blob_item_derive(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
