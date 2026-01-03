use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Result, parse2};

/// Implementation of the NetabaseBlobItem derive macro
///
/// This macro automatically implements the NetabaseBlobItem trait for blob types.
/// By default, it treats the entire type as a single blob (no splitting).
///
/// # Example
/// ```
/// #[derive(NetabaseBlobItem)]
/// pub struct MyBlob {
///     pub data: Vec<u8>,
/// }
/// ```
pub fn netabase_blob_item_derive(input: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;
    let name = &input.ident;

    // For now, we generate a simple implementation that treats the whole type as one blob
    // Future enhancement: support splitting large blobs into chunks

    match &input.data {
        Data::Struct(data_struct) => {
            // Check if it's a struct with named fields or a tuple struct
            let has_data_field = match &data_struct.fields {
                Fields::Named(fields) => fields
                    .named
                    .iter()
                    .any(|f| f.ident.as_ref().map(|i| i == "data").unwrap_or(false)),
                Fields::Unnamed(_) => true,
                Fields::Unit => false,
            };

            let (wrap_impl, unwrap_impl) = if has_data_field {
                // If there's a data field, use it
                match &data_struct.fields {
                    Fields::Named(_) => (
                        quote! { Self { data, ..Default::default() } },
                        quote! { Some((0, blob.data.clone())) },
                    ),
                    Fields::Unnamed(_) => {
                        (quote! { Self(data) }, quote! { Some((0, blob.0.clone())) })
                    }
                    Fields::Unit => unreachable!(),
                }
            } else {
                // Generic implementation
                (
                    quote! {
                        postcard::from_bytes(&data).unwrap_or_default()
                    },
                    quote! {
                        Some((0, postcard::to_allocvec(blob).unwrap_or_default()))
                    },
                )
            };

            Ok(quote! {
                impl netabase_store::blob::NetabaseBlobItem for #name {
                    type Blobs = Self;

                    fn split_into_blobs(&self) -> Vec<Self::Blobs> {
                        vec![self.clone()]
                    }

                    fn reconstruct_from_blobs(blobs: Vec<Self::Blobs>) -> Self {
                        blobs.into_iter().next().unwrap_or_default()
                    }

                    fn wrap_blob(_index: u8, data: Vec<u8>) -> Self::Blobs {
                        #wrap_impl
                    }

                    fn unwrap_blob(blob: &Self::Blobs) -> Option<(u8, Vec<u8>)> {
                        #unwrap_impl
                    }
                }
            })
        }
        Data::Enum(_) => Err(syn::Error::new_spanned(
            name,
            "NetabaseBlobItem can only be derived for structs, not enums",
        )),
        Data::Union(_) => Err(syn::Error::new_spanned(
            name,
            "NetabaseBlobItem can only be derived for structs, not unions",
        )),
    }
}
