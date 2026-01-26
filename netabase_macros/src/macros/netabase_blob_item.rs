use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Result, parse2};

use crate::utils::{attributes::{find_attribute, get_data_fields, has_attribute}, naming::to_pascal_case};

/// Implementation of the NetabaseBlobItem derive macro
///
/// This macro automatically implements the NetabaseBlobItem trait for blob types.
/// Two modes are supported:
/// 
/// 1. **Default (Whole Struct)**: If no fields are marked with `#[blob]`, the entire struct
///    is serialized and chunked. A tuple struct `{Name}Blobs` is generated to hold chunks.
/// 
/// 2. **Field Level**: If any field is marked with `#[blob]`, a `{Name}Blobs` enum is generated.
///    - Fields marked `#[blob]` are chunked (variants: `Variant(u8, Vec<u8>)`).
///    - Other fields are stored whole (variants: `Variant(Vec<u8>)`).
pub fn netabase_blob_item_derive(input: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;
    let name = &input.ident;
    let blob_name = format_ident!("{}Blobs", name);

    let (blobbed_fields, regular_fields) =
        get_data_fields(&input.data, |f| has_attribute(&f.attrs, "blob"));
        
    // If no fields are explicitly marked with #[blob], we default to "Whole Struct" mode
    let has_blob_fields = !blobbed_fields.is_empty();

    let get_variant_name = |f: &syn::Field| {
        if let Some(fld) = &f.ident {
            to_pascal_case(&fld.to_string())
        } else {
            let mut name = String::new();
            if let Some(att_name) = find_attribute(&f.attrs, "blob_as") {
                att_name
                    .parse_nested_meta(|m| {
                        name = m
                            .path
                            .get_ident()
                            .expect("This attribute needs a name")
                            .to_string();
                        Ok(())
                    })
                    .expect("Failed to parse blob_as attribute");
            } else {
                panic!("Unnamed fields should have a `blob_as(name)` attribute");
            }
            to_pascal_case(&name)
        }
    };

    if !has_blob_fields {
        // --- Mode 1: Whole Struct ---
        // Treat the entire struct as one blob item that gets chunked.
        
        let blob_struct_def = quote! {
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, std::hash::Hash)]
            pub struct #blob_name(pub u8, pub Vec<u8>);
            
            impl netabase_store::blob::NetabaseBlobItem for #blob_name {
                type Blobs = Self;
                
                fn split_into_blobs(&self) -> Vec<Self::Blobs> {
                    vec![self.clone()]
                }
                
                fn reconstruct_from_blobs(blobs: Vec<Self::Blobs>) -> Self {
                    blobs.into_iter().next().unwrap()
                }
                
                fn get_blob_index(&self) -> Option<u8> {
                    Some(self.0)
                }
            }
        };
        
        Ok(quote! {
            #blob_struct_def

            impl netabase_store::blob::NetabaseBlobItem for #name {
                type Blobs = #blob_name;

                fn split_into_blobs(&self) -> Vec<Self::Blobs> {
                    let serialized = postcard::to_allocvec(self).expect("Failed to serialize blob item");
                    if serialized.is_empty() {
                        return Vec::new();
                    }
                    serialized
                        .chunks(60000)
                        .enumerate()
                        .map(|(i, chunk)| #blob_name(i as u8, chunk.to_vec()))
                        .collect()
                }

                fn reconstruct_from_blobs(blobs: Vec<Self::Blobs>) -> Self {
                    if blobs.is_empty() {
                        return postcard::from_bytes(&[]).expect("Failed to deserialize empty blob");
                    }
                    let mut parts: Vec<(u8, Vec<u8>)> = blobs.into_iter()
                        .map(|b| (b.0, b.1))
                        .collect();
                    parts.sort_by_key(|(i, _)| *i);
                    
                    let mut result = Vec::new();
                    for (_, part) in parts {
                        result.extend(part);
                    }
                    postcard::from_bytes(&result).expect("Failed to reconstruct blob item")
                }
            }
        })
    } else {
        // --- Mode 2: Field Level ---
        // Split specific fields into chunks, keep others whole.
        
        let all_fields = blobbed_fields.iter().chain(regular_fields.iter());
        
        // Generate Enum Variants
        let mut variants = Vec::new();
        let mut split_logic = Vec::new();
        let mut reconstruct_vars = Vec::new();
        let mut reconstruct_match_arms = Vec::new();
        let mut reconstruct_build = Vec::new();
        let mut field_constructions = Vec::new();
        let mut get_index_arms = Vec::new();

        for f in all_fields {
            let v_name = format_ident!("{}", get_variant_name(f));
            let f_name = f.ident.as_ref().expect("Field mode requires named fields (or implement support)");
            
            let f_type = &f.ty;
            let is_blobbed = has_attribute(&f.attrs, "blob");
            
            // 1. Variant & Index Logic
            if is_blobbed {
                variants.push(quote! { #v_name(u8, Vec<u8>) });
                get_index_arms.push(quote! { #blob_name::#v_name(i, _) => Some(*i), });
            } else {
                variants.push(quote! { #v_name(Vec<u8>) });
                // Unchunked fields are treated as index 0 (header/single chunk)
                get_index_arms.push(quote! { #blob_name::#v_name(_) => Some(0), });
            }
            
            // 2. Split Logic
            if is_blobbed {
                split_logic.push(quote! {
                    let data = postcard::to_allocvec(&self.#f_name).expect("Failed to serialize field");
                    for (i, chunk) in data.chunks(60000).enumerate() {
                        blobs.push(#blob_name::#v_name(i as u8, chunk.to_vec()));
                    }
                });
            } else {
                split_logic.push(quote! {
                    let data = postcard::to_allocvec(&self.#f_name).expect("Failed to serialize field");
                    blobs.push(#blob_name::#v_name(data));
                });
            }
            
            // 3. Reconstruct Logic - Variables
            let chunks_var = format_ident!("chunks_{}", f_name);
            let data_var = format_ident!("data_{}", f_name);
            
            if is_blobbed {
                reconstruct_vars.push(quote! { let mut #chunks_var: Vec<(u8, Vec<u8>)> = Vec::new(); });
                reconstruct_match_arms.push(quote! { #blob_name::#v_name(i, d) => #chunks_var.push((i, d)), });
                
                reconstruct_build.push(quote! {
                    #chunks_var.sort_by_key(|(i, _)| *i);
                    let bytes: Vec<u8> = #chunks_var.into_iter().flat_map(|(_, d)| d).collect();
                    let #f_name: #f_type = postcard::from_bytes(&bytes).expect("Failed to deserialize field");
                });
            } else {
                reconstruct_vars.push(quote! { let mut #data_var: Option<Vec<u8>> = None; });
                reconstruct_match_arms.push(quote! { #blob_name::#v_name(d) => #data_var = Some(d), });
                
                reconstruct_build.push(quote! {
                    let bytes = #data_var.unwrap_or_default();
                    let #f_name: #f_type = postcard::from_bytes(&bytes).expect("Failed to deserialize field");
                });
            }
            
            field_constructions.push(quote! { #f_name });
        }

        let blob_enum = quote! {
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, std::hash::Hash)]
            pub enum #blob_name {
                #(#variants),*
            }
            
            impl netabase_store::blob::NetabaseBlobItem for #blob_name {
                type Blobs = Self;
                
                fn split_into_blobs(&self) -> Vec<Self::Blobs> {
                    vec![self.clone()]
                }
                
                fn reconstruct_from_blobs(blobs: Vec<Self::Blobs>) -> Self {
                    blobs.into_iter().next().unwrap()
                }
                
                fn get_blob_index(&self) -> Option<u8> {
                    match self {
                        #(#get_index_arms)*
                    }
                }
            }
        };

        Ok(quote! {
            #blob_enum

            impl netabase_store::blob::NetabaseBlobItem for #name {
                type Blobs = #blob_name;

                fn split_into_blobs(&self) -> Vec<Self::Blobs> {
                    let mut blobs = Vec::new();
                    #(#split_logic)*
                    blobs
                }

                fn reconstruct_from_blobs(blobs: Vec<Self::Blobs>) -> Self {
                    #(#reconstruct_vars)*
                    
                    for blob in blobs {
                        match blob {
                            #(#reconstruct_match_arms)*
                        }
                    }
                    
                    #(#reconstruct_build)*
                    
                    Self {
                        #(#field_constructions),*
                    }
                }
            }
        })
    }
}
