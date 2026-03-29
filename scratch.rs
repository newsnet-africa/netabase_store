use syn::{parse_quote, DeriveInput, Attribute, Meta, LitInt};

fn parse_chunk_size(attrs: &[Attribute]) -> Option<usize> {
    for attr in attrs {
        if attr.path().is_ident("chunk_size") {
            if let Meta::List(list) = &attr.meta {
                let lit: LitInt = list.parse_args().unwrap();
                return Some(lit.base10_parse().unwrap());
            }
        }
    }
    None
}

fn main() {
    let input: DeriveInput = parse_quote! {
        #[derive(NetabaseBlob)]
        #[chunk_size(2048)]
        struct MyBlob {
            field1: String,
        }
    };
    println!("{:?}", parse_chunk_size(&input.attrs));
}
