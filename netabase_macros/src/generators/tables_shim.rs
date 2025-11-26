// This module provides a way to generate Tables impl without feature gates

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generate the Tables impl that works regardless of user's features
///
/// The trick: emit code that uses a type from netabase_store that resolves
/// to the right thing based on netabase_store's own features, not the user's.
pub fn generate_tables_impl_shim(
    tables_name: &Ident,
) -> TokenStream {
    quote! {
        // Use a type alias from netabase_store that resolves correctly
        type Tables = ::netabase_store::__internal::SelectTables<#tables_name, ()>;

        fn tables() -> Self::Tables {
            ::netabase_store::__internal::make_tables::<#tables_name, Self::Tables>()
        }
    }
}
