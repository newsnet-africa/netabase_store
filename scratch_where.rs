use syn::WhereClause;
fn main() {
    let _: WhereClause = syn::parse_quote! { where };
}
