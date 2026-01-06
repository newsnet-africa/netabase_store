use proc_macro2::TokenStream;
use syn::{ItemStruct, parse_quote, Result};
use crate::utils::attributes::{has_attribute, remove_attribute};

/// Implementation of the netabase_libp2p attribute macro.
///
/// This macro is used to mark a model as supporting Libp2p features.
/// It injects the `libp2p_metadata` field into the struct.
///
/// When used inside a `#[netabase_definition]` module, the definition macro
/// handles this transformation manually. This macro definition primarily serves
/// as a marker and potential standalone implementation.
pub fn netabase_libp2p_attribute(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let mut input: ItemStruct = syn::parse2(item)?;
    
    // Inject the field
    inject_libp2p_field(&mut input);
    
    // Remove the attribute itself to prevent recursion if it wasn't already consumed
    // (though the compiler calls this because the attribute IS present)
    // Note: The attribute driving this macro execution is already consumed by the compiler
    // before calling this function. We only need to remove inner occurrences if any.
    
    use quote::quote;
    Ok(quote! { #input })
}

/// Helper to inject the libp2p_metadata field into a struct if the attribute is present
pub fn inject_libp2p_field(item_struct: &mut ItemStruct) {
    // Check if the struct has the attribute (for when called from ModelMutator)
    // or if we should just inject it (when called from the macro entry point)
    // The ModelMutator logic checks for the attribute first.
    
    // Inject libp2p_metadata field
    let libp2p_field: syn::Field = parse_quote! {
        pub libp2p_metadata: Option<netabase_store::traits::libp2p::libp2p_model::Libp2pMetadata>
    };

    if let syn::Fields::Named(fields) = &mut item_struct.fields {
        fields.named.push(libp2p_field);
    }
}

/// Check and process libp2p attribute
pub fn process_libp2p_attribute(item_struct: &mut ItemStruct) -> bool {
    let is_libp2p = has_attribute(&item_struct.attrs, "netabase_libp2p");
    
    if is_libp2p {
        inject_libp2p_field(item_struct);
        remove_attribute(&mut item_struct.attrs, "netabase_libp2p");
    }
    
    is_libp2p
}
