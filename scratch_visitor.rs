use proc_macro_flow::Visited;
struct LifetimeVisitor<'a> {
    ident: &'a syn::Ident,
}
impl<'a> From<&'a syn::DeriveInput> for LifetimeVisitor<'a> {
    fn from(input: &'a syn::DeriveInput) -> Self {
        Self { ident: &input.ident }
    }
}
struct Plan;
impl<'v, 'a> TryFrom<&'v Visited<'a, LifetimeVisitor<'a>>> for Plan {
    type Error = syn::Error;
    fn try_from(_: &'v Visited<'a, LifetimeVisitor<'a>>) -> Result<Self, syn::Error> {
        Ok(Plan)
    }
}
fn compile_check() {
    // let result = proc_macro_flow::run_derive_pipeline::<LifetimeVisitor<'_>, Plan, ()>(syn::parse_quote!(struct A;));
}
