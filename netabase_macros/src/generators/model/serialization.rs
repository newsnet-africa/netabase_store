use crate::utils::naming::*;
use crate::visitors::model::field::ModelFieldVisitor;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Generator for serialization trait implementations (redb Value/Key)
pub struct SerializationGenerator<'a> {
    visitor: &'a ModelFieldVisitor,
    /// Flag to control whether to generate trait impls for the ID type
    generate_id_traits: bool,
}

impl<'a> SerializationGenerator<'a> {
    pub fn new(visitor: &'a ModelFieldVisitor) -> Self {
        Self {
            visitor,
            generate_id_traits: true,
        }
    }

    /// Create a new generator with explicit control over ID trait generation
    pub fn with_id_traits(visitor: &'a ModelFieldVisitor, generate_id_traits: bool) -> Self {
        Self {
            visitor,
            generate_id_traits,
        }
    }

    /// Generate redb Value and Key implementations for the model
    pub fn generate_model_value_key(&self) -> TokenStream {
        let model_name = &self.visitor.model_name;

        if let Some(_ca_config) = &self.visitor.content_addressed_config {
            // Content-addressed: Generate Envelope and impl Value for Envelope
            let envelope_name = format_ident!("{}Envelope", model_name);

            quote! {
                /// Envelope for content-addressed model, storing the hash and the data.
                #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, std::hash::Hash)]
                pub struct #envelope_name {
                    pub hash: <#model_name as ::netabase_store::traits::registry::models::content_addressed::ContentAddressedModel>::Key,
                    pub inner: #model_name,
                }

                impl redb::Value for #envelope_name {
                    type SelfType<'a> = #envelope_name;
                    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

                    #[inline]
                    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
                    where
                        Self: 'a,
                    {
                        postcard::from_bytes(data).unwrap()
                    }

                    #[inline]
                    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                    where
                        Self: 'a,
                        Self: 'b,
                    {
                        std::borrow::Cow::Owned(
                            postcard::to_allocvec(value).unwrap()
                        )
                    }

                    #[inline]
                    fn fixed_width() -> Option<usize> {
                        None
                    }

                    #[inline]
                    fn type_name() -> redb::TypeName {
                        redb::TypeName::new(&format!("{}::{}", module_path!(), stringify!(#envelope_name)))
                    }
                }

                impl redb::Key for #envelope_name {
                    #[inline]
                    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                        let val1: #envelope_name = postcard::from_bytes(data1).unwrap();
                        let val2: #envelope_name = postcard::from_bytes(data2).unwrap();
                        val1.cmp(&val2)
                    }
                }

                impl<'a> From<&'a #model_name> for #envelope_name {
                    fn from(model: &'a #model_name) -> Self {
                         use ::netabase_store::traits::registry::models::content_addressed::ContentAddressedModel;
                         let hash = model.compute_hash();
                         Self {
                             hash,
                             inner: model.clone(),
                         }
                    }
                }

                impl From<#model_name> for #envelope_name {
                    fn from(model: #model_name) -> Self {
                         use ::netabase_store::traits::registry::models::content_addressed::ContentAddressedModel;
                         let hash = model.compute_hash();
                         Self {
                             hash,
                             inner: model,
                         }
                    }
                }

                impl std::ops::Deref for #envelope_name {
                    type Target = #model_name;

                    fn deref(&self) -> &Self::Target {
                        &self.inner
                    }
                }

                // The model itself still implements redb::Value (direct serialization)
                // This is useful if used nested or for computation
                impl redb::Value for #model_name {
                    type SelfType<'a> = #model_name;
                    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

                    #[inline]
                    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
                    where
                        Self: 'a,
                    {
                        // Content-addressed models can also be versioned
                        postcard::from_bytes(data).unwrap()
                    }

                    #[inline]
                    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                    where
                        Self: 'a,
                        Self: 'b,
                    {
                        std::borrow::Cow::Owned(
                            postcard::to_allocvec(value).unwrap()
                        )
                    }

                    #[inline]
                    fn fixed_width() -> Option<usize> {
                        None
                    }

                    #[inline]
                    fn type_name() -> redb::TypeName {
                        redb::TypeName::new(&format!("{}::{}", module_path!(), stringify!(#model_name)))
                    }
                }
            }
        } else {
            // Standard model  
            let from_bytes_impl = if let Some(version_info) = &self.visitor.version_info {
                if version_info.is_current.unwrap_or(false) {
                    // Current version of a versioned model - use family enum for automatic migration
                    let family = &version_info.family;
                    let family_enum = format_ident!("{}Family", family);
                    
                    quote! {
                        // Try using family enum for automatic version detection
                        // This allows reading old versions and auto-migrating
                        match #family_enum::try_from_bytes(data) {
                            Ok(family) => family.to_current(),
                            Err(_) => {
                                // Fallback to direct deserialization (shouldn't happen)
                                postcard::from_bytes(data).unwrap()
                            }
                        }
                    }
                } else {
                    // Old version - direct deserialization only
                    quote! {
                        postcard::from_bytes(data).unwrap()
                    }
                }
            } else {
                // Non-versioned model - direct deserialization
                quote! {
                    postcard::from_bytes(data).unwrap()
                }
            };

            quote! {
                impl redb::Value for #model_name {
                    type SelfType<'a> = #model_name;
                    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

                    #[inline]
                    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
                    where
                        Self: 'a,
                    {
                        #from_bytes_impl
                    }

                    #[inline]
                    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                    where
                        Self: 'a,
                        Self: 'b,
                    {
                        std::borrow::Cow::Owned(
                            postcard::to_allocvec(value).unwrap()
                        )
                    }

                    #[inline]
                    fn fixed_width() -> Option<usize> {
                        None
                    }

                    #[inline]
                    fn type_name() -> redb::TypeName {
                        redb::TypeName::new(&format!("{}::{}", module_path!(), stringify!(#model_name)))
                    }
                }

                impl redb::Key for #model_name {
                    #[inline]
                    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                        let val1: #model_name = postcard::from_bytes(data1).unwrap();
                        let val2: #model_name = postcard::from_bytes(data2).unwrap();
                        val1.cmp(&val2)
                    }
                }
            }
        }
    }

    /// Generate redb Value and Key implementations for key enums
    pub fn generate_key_enum_value_key(&self) -> TokenStream {
        let mut output = TokenStream::new();

        let model_name = &self.visitor.model_name;

        // ID type - only generate if flagged (to avoid duplicates for versioned models)
        if self.generate_id_traits {
            let id_type = primary_key_type_name_for_model(self.visitor);
            let inner_ty = self.visitor.primary_key.as_ref().map(|f| &f.ty);
            output.extend(self.generate_value_key_for_type(&id_type, inner_ty));
        }

        // Secondary keys enum
        let secondary_enum = secondary_keys_enum_name(model_name);
        output.extend(self.generate_value_key_for_type(&secondary_enum, None));

        // Relational keys enum
        let relational_enum = relational_keys_enum_name(model_name);
        output.extend(self.generate_value_key_for_type(&relational_enum, None));

        // Subscriptions enum - handled by definition/traits.rs to properly support
        // both empty and non-empty enums with correct trait implementations

        // Blob keys enum
        let blob_keys = blob_keys_enum_name(model_name);
        output.extend(self.generate_value_key_for_type(&blob_keys, None));

        let blob_item = blob_item_enum_name(model_name);
        output.extend(self.generate_value_key_for_type(&blob_item, None));

        let libp2p_provider_key = libp2p_provider_key_enum_name(model_name);
        output.extend(self.generate_value_key_for_type(&libp2p_provider_key, None));

        output
    }

    fn generate_value_key_for_type(&self, type_name: &syn::Ident, inner_type: Option<&syn::Type>) -> TokenStream {
        // If inner type is a supported primitive, delegate to it (Transparent Key Optimization)
        if let Some(ty) = inner_type
            && is_primitive_type(ty) {
                return quote! {
                    impl redb::Value for #type_name {
                        type SelfType<'a> = #type_name;
                        type AsBytes<'a> = <#ty as redb::Value>::AsBytes<'a>;

                        #[inline]
                        fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
                        where
                            Self: 'a,
                        {
                            #type_name(<#ty as redb::Value>::from_bytes(data))
                        }

                        #[inline]
                        fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                        where
                            Self: 'a,
                            Self: 'b,
                        {
                            <#ty as redb::Value>::as_bytes(&value.0)
                        }

                        #[inline]
                        fn fixed_width() -> Option<usize> {
                            <#ty as redb::Value>::fixed_width()
                        }

                        #[inline]
                        fn type_name() -> redb::TypeName {
                            redb::TypeName::new(&format!("{}::{}", module_path!(), stringify!(#type_name)))
                        }
                    }

                    impl redb::Key for #type_name {
                        #[inline]
                        fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                            <#ty as redb::Key>::compare(data1, data2)
                        }
                    }
                };
            }

        // Fallback to postcard serialization (Varint / Custom format)
        quote! {
            impl redb::Value for #type_name {
                type SelfType<'a> = #type_name;
                type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

                #[inline]
                fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
                where
                    Self: 'a,
                {
                    postcard::from_bytes(data).unwrap()
                }

                #[inline]
                fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                where
                    Self: 'a,
                    Self: 'b,
                {
                    std::borrow::Cow::Owned(
                        postcard::to_allocvec(value).unwrap()
                    )
                }

                #[inline]
                fn fixed_width() -> Option<usize> {
                    None
                }

                #[inline]
                fn type_name() -> redb::TypeName {
                    redb::TypeName::new(&format!("{}::{}", module_path!(), stringify!(#type_name)))
                }
            }

            impl redb::Key for #type_name {
                #[inline]
                fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                    let val1: #type_name = postcard::from_bytes(data1).unwrap();
                    let val2: #type_name = postcard::from_bytes(data2).unwrap();
                    val1.cmp(&val2)
                }
            }
        }
    }

    /// Generate blob trait implementations
    /// Generates NetabaseBlobItem implementations for:
    /// 1. The BlobItem enum itself (as a passthrough wrapper for chunks)
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
                    
                    fn get_blob_index(&self) -> Option<u8> {
                        None
                    }
                }
            };
        }

        // Generate match arms for get_blob_index
        let get_index_arms: Vec<_> = self
            .visitor
            .blob_fields
            .iter()
            .map(|field| {
                let variant_name = to_pascal_case(&field.name.to_string());
                let variant_ident = syn::Ident::new(&variant_name, field.name.span());

                quote! {
                    #blob_item_enum::#variant_ident(inner) => inner.get_blob_index()
                }
            })
            .collect();

        // Generate impl for the BlobItem enum itself
        // The enum itself IS the chunk, so it splits into itself
        quote! {
            impl netabase_store::blob::NetabaseBlobItem for #blob_item_enum {
                type Blobs = Self;

                fn split_into_blobs(&self) -> Vec<Self::Blobs> {
                    vec![self.clone()]
                }

                fn reconstruct_from_blobs(mut blobs: Vec<Self::Blobs>) -> Self {
                    blobs.into_iter().next().expect("Cannot reconstruct from empty blob list")
                }
                
                fn get_blob_index(&self) -> Option<u8> {
                    match self {
                        #(#get_index_arms),*
                    }
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

fn is_primitive_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(p) = ty
        && let Some(ident) = p.path.get_ident() {
            let s = ident.to_string();
            return matches!(s.as_str(), 
                "u8" | "u16" | "u32" | "u64" | "u128" | 
                "i8" | "i16" | "i32" | "i64" | "i128" |
                "String");
        }
    false
}
