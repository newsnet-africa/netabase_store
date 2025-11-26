use syn::{Fields, Ident, visit::Visit};

#[derive(Default)]
pub struct UniffiVisitor<'ast> {
    pub name: Option<&'ast Ident>,
    pub fields: Option<&'ast Fields>,
}

impl<'a> Visit<'a> for UniffiVisitor<'a> {
    fn visit_derive_input(&mut self, i: &'a syn::DeriveInput) {
        self.name = Some(&i.ident);
        match &i.data {
            syn::Data::Struct(data_struct) => self.fields = Some(&data_struct.fields),
            _ => unreachable!("Only structs should be visited"),
        }
    }
}
