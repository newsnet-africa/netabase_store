use proc_macro::TokenStream;
mod generators;
mod visitors;

#[proc_macro_derive(NetabaseModel, attributes(primary_key, secondary_key, relational))]
pub fn netabase_model(input: TokenStream) -> TokenStream {
    input
}
