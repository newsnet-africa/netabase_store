#[cfg(test)]
mod tests {
    use syn::{parse_quote, DeriveInput};
    use crate::generators::blob::{BlobPlan, BlobGenerator};
    use crate::visitors::blob::BlobVisitor;

    #[test]
    fn test_netabase_blob_macro_visitation() {
        let input: DeriveInput = parse_quote! {
            #[derive(NetabaseBlob)]
            struct TestBlob {
                #[chunk_size(100)]
                a: String,
            }
        };

        let output = proc_macro_flow::run_derive_pipeline::<BlobVisitor, BlobPlan, BlobGenerator, syn::DeriveInput>(input).unwrap();
        assert!(!output.is_empty());
    }
}

