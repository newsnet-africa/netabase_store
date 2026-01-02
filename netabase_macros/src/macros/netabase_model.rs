use proc_macro2::TokenStream;
use syn::Result;

/// Implementation of the NetabaseModel derive macro
/// This is now a no-op because the netabase_definition attribute macro
/// handles all the code generation and struct mutation.
/// This derive macro mainly serves as a marker for the visitor.
pub fn netabase_model_derive(_input: TokenStream) -> Result<TokenStream> {
    Ok(TokenStream::new())
}