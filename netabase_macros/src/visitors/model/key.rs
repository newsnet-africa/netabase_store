pub struct ModelKeyVisitor {
    primary_key: syn::Type,
    secondary_keys: Vec<syn::Type>,
}
