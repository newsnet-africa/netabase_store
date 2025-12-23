use proc_macro2::TokenStream;
use quote::quote;
use crate::visitors::model::field::ModelFieldVisitor;
use crate::utils::naming::*;

/// Generator for serialization trait implementations (redb Value/Key)
pub struct SerializationGenerator<'a> {
    visitor: &'a ModelFieldVisitor,
}

impl<'a> SerializationGenerator<'a> {
    pub fn new(visitor: &'a ModelFieldVisitor) -> Self {
        Self { visitor }
    }

    /// Generate redb Value and Key implementations for the model
    pub fn generate_model_value_key(&self) -> TokenStream {
        let model_name = &self.visitor.model_name;

        quote! {
            impl redb::Value for #model_name {
                type SelfType<'a> = #model_name;
                type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

                fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
                where
                    Self: 'a,
                {
                    bincode::decode_from_slice(data, bincode::config::standard())
                        .unwrap()
                        .0
                }

                fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                where
                    Self: 'a,
                    Self: 'b,
                {
                    std::borrow::Cow::Owned(
                        bincode::encode_to_vec(value, bincode::config::standard()).unwrap()
                    )
                }

                fn fixed_width() -> Option<usize> {
                    None
                }

                fn type_name() -> redb::TypeName {
                    redb::TypeName::new(&format!("{}::{}", module_path!(), stringify!(#model_name)))
                }
            }

            impl redb::Key for #model_name {
                fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                    let val1: #model_name = bincode::decode_from_slice(data1, bincode::config::standard())
                        .unwrap()
                        .0;
                    let val2: #model_name = bincode::decode_from_slice(data2, bincode::config::standard())
                        .unwrap()
                        .0;
                    val1.cmp(&val2)
                }
            }
        }
    }

    /// Generate redb Value and Key implementations for key enums
    pub fn generate_key_enum_value_key(&self) -> TokenStream {
        let mut output = TokenStream::new();

        let model_name = &self.visitor.model_name;

        // ID type
        let id_type = primary_key_type_name(model_name);
        output.extend(self.generate_value_key_for_type(&id_type));

        // Secondary keys enum
        let secondary_enum = secondary_keys_enum_name(model_name);
        output.extend(self.generate_value_key_for_type(&secondary_enum));

        // Relational keys enum
        let relational_enum = relational_keys_enum_name(model_name);
        output.extend(self.generate_value_key_for_type(&relational_enum));

        // Subscriptions enum - always generate even if no subscriptions
        let enum_name = subscriptions_enum_name(model_name);
        output.extend(self.generate_value_key_for_type(&enum_name));

        // Blob keys enum
        let blob_keys = blob_keys_enum_name(model_name);
        output.extend(self.generate_value_key_for_type(&blob_keys));

        let blob_item = blob_item_enum_name(model_name);
        output.extend(self.generate_value_key_for_type(&blob_item));

        output
    }

    fn generate_value_key_for_type(&self, type_name: &syn::Ident) -> TokenStream {
        quote! {
            impl redb::Value for #type_name {
                type SelfType<'a> = #type_name;
                type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

                fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
                where
                    Self: 'a,
                {
                    bincode::decode_from_slice(data, bincode::config::standard())
                        .unwrap()
                        .0
                }

                fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                where
                    Self: 'a,
                    Self: 'b,
                {
                    std::borrow::Cow::Owned(
                        bincode::encode_to_vec(value, bincode::config::standard()).unwrap()
                    )
                }

                fn fixed_width() -> Option<usize> {
                    None
                }

                fn type_name() -> redb::TypeName {
                    redb::TypeName::new(&format!("{}::{}", module_path!(), stringify!(#type_name)))
                }
            }

            impl redb::Key for #type_name {
                fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                    let val1: #type_name = bincode::decode_from_slice(data1, bincode::config::standard())
                        .unwrap()
                        .0;
                    let val2: #type_name = bincode::decode_from_slice(data2, bincode::config::standard())
                        .unwrap()
                        .0;
                    val1.cmp(&val2)
                }
            }
        }
    }

    /// Generate blob trait implementations
    /// NOTE: Users must manually implement NetabaseBlobItem for their blob types.
    /// We only generate the implementation for the BlobItem enum itself.
    pub fn generate_blob_traits(&self) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let blob_item_enum = blob_item_enum_name(model_name);

        // If no blob fields, generate empty impl for the struct
        if self.visitor.blob_fields.is_empty() {
            return quote! {
                impl netabase_store::blob::NetabaseBlobItem for #blob_item_enum {
                    type Blobs = Self;

                    fn split_into_blobs(&self) -> Vec<Self::Blobs> {
                        vec![]
                    }

                    fn reconstruct_from_blobs(_blobs: Vec<Self::Blobs>) -> Self {
                        #blob_item_enum
                    }

                    fn wrap_blob(_index: u8, _data: Vec<u8>) -> Self::Blobs {
                        #blob_item_enum
                    }

                    fn unwrap_blob(_blob: &Self::Blobs) -> Option<(u8, Vec<u8>)> {
                        None
                    }
                }
            };
        }

        // Generate impl for the BlobItem enum only
        let reconstruct_arms: Vec<_> = self.visitor.blob_fields
            .iter()
            .map(|field| {
                let variant_name = to_pascal_case(&field.name.to_string());
                let variant_ident = syn::Ident::new(&variant_name, field.name.span());

                quote! {
                    #blob_item_enum::#variant_ident { .. } => blobs.into_iter().next().unwrap()
                }
            })
            .collect();

        quote! {
            impl netabase_store::blob::NetabaseBlobItem for #blob_item_enum {
                type Blobs = Self;

                fn split_into_blobs(&self) -> Vec<Self::Blobs> {
                    vec![self.clone()]
                }

                fn reconstruct_from_blobs(mut blobs: Vec<Self::Blobs>) -> Self {
                    match blobs.first() {
                        Some(first) => match first {
                            #(#reconstruct_arms),*
                        }
                        None => panic!("Cannot reconstruct from empty blob list"),
                    }
                }

                fn wrap_blob(_index: u8, _data: Vec<u8>) -> Self::Blobs {
                    panic!("Cannot wrap blob directly on enum")
                }

                fn unwrap_blob(_blob: &Self::Blobs) -> Option<(u8, Vec<u8>)> {
                    panic!("Cannot unwrap blob directly on enum")
                }
            }
        }
    }
}

/// Helper function to convert snake_case to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}
