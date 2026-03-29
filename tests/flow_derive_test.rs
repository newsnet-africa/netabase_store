//! Integration tests for the FlowVisitor, FlowPlan, and FlowGenerator derive macros.
//!
//! These tests demonstrate the Visitor -> Plan -> Generator pattern using the new
//! derive macros to reduce boilerplate.

use proc_macro_flow::{FlowVisitor, FlowPlan, FlowGenerator};
use proc_macro_flow::{Generatable, MultiGeneratable, Visited, items_to_token_stream};

// ============================================================================
// SECTION 1: FlowVisitor Tests
// ============================================================================

mod visitor_tests {
    use super::*;

    /// Test basic visitor with explicit #[visit(field)] attributes.
    #[derive(FlowVisitor)]
    struct ExplicitVisitor<'a> {
        #[visit(ident)]
        name: &'a syn::Ident,
        
        #[visit(generics)]
        generic_params: &'a syn::Generics,
        
        #[visit(attrs)]
        attributes: &'a [syn::Attribute],
    }

    #[test]
    fn test_explicit_field_extraction() {
        let input: syn::DeriveInput = syn::parse_quote! {
            #[derive(Debug)]
            #[custom_attr]
            struct MyStruct<T: Clone, U> {
                field: T,
            }
        };
        
        let visitor = ExplicitVisitor::from(&input);
        
        assert_eq!(visitor.name.to_string(), "MyStruct");
        assert_eq!(visitor.generic_params.params.len(), 2);
        assert_eq!(visitor.attributes.len(), 2);
    }

    /// Test visitor with default field extraction (field names match DeriveInput fields).
    #[derive(FlowVisitor)]
    struct DefaultFieldVisitor<'a> {
        ident: &'a syn::Ident,
        generics: &'a syn::Generics,
        attrs: &'a [syn::Attribute],
        vis: &'a syn::Visibility,
        data: &'a syn::Data,
    }

    #[test]
    fn test_default_field_extraction() {
        let input: syn::DeriveInput = syn::parse_quote! {
            pub struct PublicStruct {
                field1: i32,
                field2: String,
            }
        };
        
        let visitor = DefaultFieldVisitor::from(&input);
        
        assert_eq!(visitor.ident.to_string(), "PublicStruct");
        assert!(matches!(visitor.vis, syn::Visibility::Public(_)));
        assert!(matches!(visitor.data, syn::Data::Struct(_)));
    }

    /// Test visitor with enum input.
    #[derive(FlowVisitor)]
    struct EnumVisitor<'a> {
        #[visit(ident)]
        name: &'a syn::Ident,
        
        #[visit(data)]
        data: &'a syn::Data,
    }

    #[test]
    fn test_visitor_with_enum() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum MyEnum {
                VariantA,
                VariantB(i32),
                VariantC { name: String },
            }
        };
        
        let visitor = EnumVisitor::from(&input);
        
        assert_eq!(visitor.name.to_string(), "MyEnum");
        match visitor.data {
            syn::Data::Enum(e) => {
                assert_eq!(e.variants.len(), 3);
            }
            _ => panic!("Expected enum data"),
        }
    }

    /// Test visitor with complex generics.
    #[derive(FlowVisitor)]
    struct GenericVisitor<'a> {
        #[visit(ident)]
        ident: &'a syn::Ident,
        
        #[visit(generics)]
        generics: &'a syn::Generics,
    }

    #[test]
    fn test_visitor_with_complex_generics() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct Complex<'a, T: Clone + Send, U: Default>
            where
                T: 'a,
                U: std::fmt::Debug,
            {
                data: &'a T,
                value: U,
            }
        };
        
        let visitor = GenericVisitor::from(&input);
        
        assert_eq!(visitor.ident.to_string(), "Complex");
        // 3 params: 'a, T, U
        assert_eq!(visitor.generics.params.len(), 3);
        // Has a where clause
        assert!(visitor.generics.where_clause.is_some());
    }

    /// Test visitor extracts multiple attributes correctly.
    #[derive(FlowVisitor)]
    struct MultiAttrVisitor<'a> {
        #[visit(ident)]
        ident: &'a syn::Ident,
        
        #[visit(attrs)]
        attrs: &'a [syn::Attribute],
    }

    #[test]
    fn test_visitor_multiple_attributes() {
        let input: syn::DeriveInput = syn::parse_quote! {
            #[derive(Debug, Clone)]
            #[serde(rename_all = "camelCase")]
            #[custom(option1, option2)]
            struct Attributed;
        };
        
        let visitor = MultiAttrVisitor::from(&input);
        
        assert_eq!(visitor.ident.to_string(), "Attributed");
        assert_eq!(visitor.attrs.len(), 3);
    }

    /// Test visitor with unit struct.
    #[test]
    fn test_visitor_unit_struct() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct UnitStruct;
        };
        
        let visitor = GenericVisitor::from(&input);
        assert_eq!(visitor.ident.to_string(), "UnitStruct");
        assert!(visitor.generics.params.is_empty());
    }

    /// Test visitor with tuple struct.
    #[derive(FlowVisitor)]
    struct TupleStructVisitor<'a> {
        #[visit(ident)]
        ident: &'a syn::Ident,
        
        #[visit(data)]
        data: &'a syn::Data,
    }

    #[test]
    fn test_visitor_tuple_struct() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct TupleStruct(i32, String, Vec<u8>);
        };
        
        let visitor = TupleStructVisitor::from(&input);
        
        assert_eq!(visitor.ident.to_string(), "TupleStruct");
        match visitor.data {
            syn::Data::Struct(s) => {
                assert!(matches!(s.fields, syn::Fields::Unnamed(_)));
                assert_eq!(s.fields.len(), 3);
            }
            _ => panic!("Expected struct data"),
        }
    }

    // --- Owning Visitor Tests (no lifetime parameter) ---

    /// Test owning visitor that clones data instead of borrowing.
    #[derive(FlowVisitor)]
    struct OwningVisitor {
        ident: syn::Ident,
        generics: syn::Generics,
        attrs: Vec<syn::Attribute>,
    }

    #[test]
    fn test_owning_visitor() {
        let input: syn::DeriveInput = syn::parse_quote! {
            #[derive(Debug)]
            struct OwnedStruct<T: Clone> {
                field: T,
            }
        };
        
        let visitor = OwningVisitor::from(&input);
        
        assert_eq!(visitor.ident.to_string(), "OwnedStruct");
        assert_eq!(visitor.generics.params.len(), 1);
        assert_eq!(visitor.attrs.len(), 1);
        
        // The visitor owns the data, so we can use it after input is dropped
        drop(input);
        assert_eq!(visitor.ident.to_string(), "OwnedStruct");
    }

    /// Test owning visitor with explicit field mappings.
    #[derive(FlowVisitor)]
    struct ExplicitOwningVisitor {
        #[visit(ident)]
        name: syn::Ident,
        
        #[visit(generics)]
        generic_params: syn::Generics,
    }

    #[test]
    fn test_explicit_owning_visitor() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct ExplicitOwned<'a, T> {
                data: &'a T,
            }
        };
        
        let visitor = ExplicitOwningVisitor::from(&input);
        
        assert_eq!(visitor.name.to_string(), "ExplicitOwned");
        assert_eq!(visitor.generic_params.params.len(), 2); // 'a and T
    }
}

// ============================================================================
// SECTION 2: FlowPlan Tests
// ============================================================================

mod plan_tests {
    use super::*;

    /// Simple visitor for plan tests.
    #[derive(FlowVisitor)]
    struct SimplePlanVisitor<'a> {
        #[visit(ident)]
        ident: &'a syn::Ident,
        
        #[visit(generics)]
        generics: &'a syn::Generics,
    }

    /// Test basic plan with field transformation.
    #[derive(FlowPlan)]
    #[plan(visitor = SimplePlanVisitor<'_>)]
    struct BasicTransformPlan {
        #[plan(from = "v.ident.clone()")]
        name: syn::Ident,
        
        #[plan(from = "v.generics.clone()")]
        generics: syn::Generics,
    }

    #[test]
    fn test_basic_plan_transformation() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct TestStruct<T> {
                value: T,
            }
        };
        
        let visited = Visited::<SimplePlanVisitor>::from(&input);
        let plan = BasicTransformPlan::try_from(&visited).expect("Plan should succeed");
        
        assert_eq!(plan.name.to_string(), "TestStruct");
        assert_eq!(plan.generics.params.len(), 1);
    }

    /// Test plan with computed values.
    #[derive(FlowVisitor)]
    struct CountingVisitor<'a> {
        #[visit(ident)]
        ident: &'a syn::Ident,
        
        #[visit(generics)]
        generics: &'a syn::Generics,
    }

    #[derive(FlowPlan)]
    #[plan(visitor = CountingVisitor<'_>)]
    struct ComputedPlan {
        #[plan(from = "v.ident.clone()")]
        name: syn::Ident,
        
        #[plan(from = "v.generics.params.len()")]
        generic_count: usize,
        
        #[plan(from = "v.generics.params.is_empty()")]
        has_no_generics: bool,
    }

    #[test]
    fn test_plan_with_computed_values() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct WithGenerics<A, B, C> {
                a: A,
                b: B,
                c: C,
            }
        };
        
        let visited = Visited::<CountingVisitor>::from(&input);
        let plan = ComputedPlan::try_from(&visited).expect("Plan should succeed");
        
        assert_eq!(plan.name.to_string(), "WithGenerics");
        assert_eq!(plan.generic_count, 3);
        assert!(!plan.has_no_generics);
    }

    #[test]
    fn test_plan_no_generics() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct NoGenerics {
                field: i32,
            }
        };
        
        let visited = Visited::<CountingVisitor>::from(&input);
        let plan = ComputedPlan::try_from(&visited).expect("Plan should succeed");
        
        assert_eq!(plan.generic_count, 0);
        assert!(plan.has_no_generics);
    }

    /// Test plan with string manipulation.
    #[derive(FlowPlan)]
    #[plan(visitor = SimplePlanVisitor<'_>)]
    struct NameManipulationPlan {
        #[plan(from = "v.ident.clone()")]
        original_name: syn::Ident,
        
        #[plan(from = "quote::format_ident!(\"{}Impl\", v.ident)")]
        impl_name: syn::Ident,
        
        #[plan(from = "quote::format_ident!(\"{}Builder\", v.ident)")]
        builder_name: syn::Ident,
    }

    #[test]
    fn test_plan_name_manipulation() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct Widget;
        };
        
        let visited = Visited::<SimplePlanVisitor>::from(&input);
        let plan = NameManipulationPlan::try_from(&visited).expect("Plan should succeed");
        
        assert_eq!(plan.original_name.to_string(), "Widget");
        assert_eq!(plan.impl_name.to_string(), "WidgetImpl");
        assert_eq!(plan.builder_name.to_string(), "WidgetBuilder");
    }
}

// ============================================================================
// SECTION 3: FlowGenerator Tests
// ============================================================================

mod generator_tests {
    use super::*;

    /// Simple visitor and plan for generator tests.
    #[derive(FlowVisitor)]
    struct GenTestVisitor<'a> {
        #[visit(ident)]
        ident: &'a syn::Ident,
        
        #[visit(generics)]
        generics: &'a syn::Generics,
    }

    #[derive(FlowPlan)]
    #[plan(visitor = GenTestVisitor<'_>)]
    struct GenTestPlan {
        #[plan(from = "v.ident.clone()")]
        name: syn::Ident,
        
        #[plan(from = "v.generics.clone()")]
        generics: syn::Generics,
    }

    /// Test basic generator with impl output.
    #[derive(FlowGenerator)]
    #[generator(output = syn::ItemImpl, plan = GenTestPlan)]
    struct ImplGenerator {
        name: syn::Ident,
        generics: syn::Generics,
    }

    impl ImplGenerator {
        fn generate_impl(&self) -> Result<syn::ItemImpl, syn::Error> {
            let name = &self.name;
            let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
            
            Ok(syn::parse_quote! {
                impl #impl_generics Default for #name #ty_generics #where_clause {
                    fn default() -> Self {
                        Self::new()
                    }
                }
            })
        }
    }

    #[test]
    fn test_impl_generator() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct TestType;
        };
        
        let visited = Visited::<GenTestVisitor>::from(&input);
        let plan = GenTestPlan::try_from(&visited).unwrap();
        let generator = ImplGenerator::from(plan);
        
        let output = Generatable::generate(generator).unwrap();
        let code = quote::quote! { #output }.to_string();
        
        assert!(code.contains("impl"));
        assert!(code.contains("Default"));
        assert!(code.contains("TestType"));
        assert!(code.contains("fn default"));
    }

    /// Test generator producing struct output.
    #[derive(FlowGenerator)]
    #[generator(output = syn::ItemStruct, plan = GenTestPlan)]
    struct StructGenerator {
        name: syn::Ident,
    }

    impl StructGenerator {
        fn generate_impl(&self) -> Result<syn::ItemStruct, syn::Error> {
            let name = quote::format_ident!("{}Builder", self.name);
            
            Ok(syn::parse_quote! {
                #[derive(Default)]
                pub struct #name {
                    _marker: std::marker::PhantomData<()>,
                }
            })
        }
    }

    #[test]
    fn test_struct_generator() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct Widget;
        };
        
        let visited = Visited::<GenTestVisitor>::from(&input);
        let plan = GenTestPlan::try_from(&visited).unwrap();
        let generator = StructGenerator { name: plan.name };
        
        let output = Generatable::generate(generator).unwrap();
        let code = quote::quote! { #output }.to_string();
        
        assert!(code.contains("WidgetBuilder"));
        assert!(code.contains("pub struct"));
        assert!(code.contains("PhantomData"));
    }

    /// Test generator with custom method name.
    #[derive(FlowGenerator)]
    #[generator(output = syn::ItemImpl, method = create_output)]
    struct CustomMethodGenerator {
        name: syn::Ident,
    }

    impl CustomMethodGenerator {
        fn create_output(&self) -> Result<syn::ItemImpl, syn::Error> {
            let name = &self.name;
            Ok(syn::parse_quote! {
                impl #name {
                    pub fn custom_method(&self) -> bool {
                        true
                    }
                }
            })
        }
    }

    #[test]
    fn test_generator_custom_method() {
        let generator = CustomMethodGenerator {
            name: syn::Ident::new("MyType", proc_macro2::Span::call_site()),
        };
        
        let output = Generatable::generate(generator).unwrap();
        let code = quote::quote! { #output }.to_string();
        
        assert!(code.contains("impl MyType"));
        assert!(code.contains("custom_method"));
    }

    /// Test generator with generics in output.
    #[test]
    fn test_generator_with_generics() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct Container<T: Clone + Send, U: Default> {
                data: T,
                value: U,
            }
        };
        
        let visited = Visited::<GenTestVisitor>::from(&input);
        let plan = GenTestPlan::try_from(&visited).unwrap();
        let generator = ImplGenerator::from(plan);
        
        let output = Generatable::generate(generator).unwrap();
        let code = quote::quote! { #output }.to_string();
        
        assert!(code.contains("impl < T : Clone + Send , U : Default >"));
        assert!(code.contains("Container < T , U >"));
    }

    /// Test generator with skip field.
    #[derive(FlowGenerator)]
    #[generator(output = syn::ItemImpl, plan = GenTestPlan)]
    struct SkipFieldGenerator {
        name: syn::Ident,
        
        #[gen(skip)]
        _cached: Option<String>,
    }

    impl SkipFieldGenerator {
        fn generate_impl(&self) -> Result<syn::ItemImpl, syn::Error> {
            let name = &self.name;
            Ok(syn::parse_quote! {
                impl #name {}
            })
        }
    }

    #[test]
    fn test_generator_skip_field() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct Skipped;
        };
        
        let visited = Visited::<GenTestVisitor>::from(&input);
        let plan = GenTestPlan::try_from(&visited).unwrap();
        let generator = SkipFieldGenerator::from(plan);
        
        // The _cached field should be None (default)
        assert!(generator._cached.is_none());
        
        let output = Generatable::generate(generator).unwrap();
        let code = quote::quote! { #output }.to_string();
        assert!(code.contains("impl Skipped"));
    }
}

// ============================================================================
// SECTION 4: Full Pipeline Integration Tests
// ============================================================================

mod integration_tests {
    use super::*;

    /// Complete visitor for integration tests.
    #[derive(FlowVisitor)]
    struct IntegrationVisitor<'a> {
        #[visit(ident)]
        ident: &'a syn::Ident,
        
        #[visit(generics)]
        generics: &'a syn::Generics,
        
        #[visit(attrs)]
        attrs: &'a [syn::Attribute],
        
        #[visit(data)]
        data: &'a syn::Data,
    }

    impl<'a> IntegrationVisitor<'a> {
        fn field_count(&self) -> usize {
            match self.data {
                syn::Data::Struct(s) => s.fields.len(),
                syn::Data::Enum(e) => e.variants.len(),
                syn::Data::Union(u) => u.fields.named.len(),
            }
        }
        
        fn is_struct(&self) -> bool {
            matches!(self.data, syn::Data::Struct(_))
        }
    }

    #[derive(FlowPlan)]
    #[plan(visitor = IntegrationVisitor<'_>)]
    struct IntegrationPlan {
        #[plan(from = "v.ident.clone()")]
        name: syn::Ident,
        
        #[plan(from = "v.generics.clone()")]
        generics: syn::Generics,
        
        #[plan(from = "v.field_count()")]
        field_count: usize,
        
        #[plan(from = "v.is_struct()")]
        is_struct: bool,
    }

    #[derive(FlowGenerator)]
    #[generator(output = syn::ItemImpl, plan = IntegrationPlan)]
    struct IntegrationGenerator {
        name: syn::Ident,
        generics: syn::Generics,
        field_count: usize,
        is_struct: bool,
    }

    impl IntegrationGenerator {
        fn generate_impl(&self) -> Result<syn::ItemImpl, syn::Error> {
            let name = &self.name;
            let field_count = self.field_count;
            let is_struct = self.is_struct;
            let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
            
            Ok(syn::parse_quote! {
                impl #impl_generics #name #ty_generics #where_clause {
                    pub const FIELD_COUNT: usize = #field_count;
                    pub const IS_STRUCT: bool = #is_struct;
                    
                    pub fn info() -> &'static str {
                        concat!("Type has ", stringify!(#field_count), " fields")
                    }
                }
            })
        }
    }

    #[test]
    fn test_full_pipeline_struct() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct Person {
                name: String,
                age: u32,
                email: Option<String>,
            }
        };
        
        // Phase 1: Visit
        let visited = Visited::<IntegrationVisitor>::from(&input);
        assert_eq!(visited.inner().ident.to_string(), "Person");
        assert_eq!(visited.inner().field_count(), 3);
        assert!(visited.inner().is_struct());
        
        // Phase 2: Plan
        let plan = IntegrationPlan::try_from(&visited).unwrap();
        assert_eq!(plan.name.to_string(), "Person");
        assert_eq!(plan.field_count, 3);
        assert!(plan.is_struct);
        
        // Phase 3: Generate
        let generator = IntegrationGenerator::from(plan);
        let items: Vec<_> = MultiGeneratable::generate(generator)
            .unwrap()
            .into_iter()
            .collect();
        
        let code = items_to_token_stream(items).to_string();
        
        assert!(code.contains("impl Person"));
        assert!(code.contains("FIELD_COUNT"));
        assert!(code.contains("IS_STRUCT"));
        assert!(code.contains("true"));
    }

    #[test]
    fn test_full_pipeline_enum() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum Status {
                Active,
                Inactive,
                Pending,
                Error(String),
            }
        };
        
        let visited = Visited::<IntegrationVisitor>::from(&input);
        let plan = IntegrationPlan::try_from(&visited).unwrap();
        
        // Assertions on plan before converting to generator
        assert_eq!(plan.name.to_string(), "Status");
        assert_eq!(plan.field_count, 4); // 4 variants
        assert!(!plan.is_struct);
        
        let generator = IntegrationGenerator::from(plan);
        
        let output = Generatable::generate(generator).unwrap();
        let code = quote::quote! { #output }.to_string();
        
        assert!(code.contains("impl Status"));
        assert!(code.contains("false")); // IS_STRUCT = false
    }

    #[test]
    fn test_full_pipeline_generic_struct() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct Container<T: Clone + Send, U>
            where
                U: Default + std::fmt::Debug,
            {
                data: T,
                metadata: U,
            }
        };
        
        let visited = Visited::<IntegrationVisitor>::from(&input);
        let plan = IntegrationPlan::try_from(&visited).unwrap();
        let generator = IntegrationGenerator::from(plan);
        
        let output = Generatable::generate(generator).unwrap();
        let code = quote::quote! { #output }.to_string();
        
        // Check generics are preserved
        assert!(code.contains("impl < T : Clone + Send , U >"));
        assert!(code.contains("Container < T , U >"));
        assert!(code.contains("where"));
        assert!(code.contains("U : Default + std :: fmt :: Debug"));
    }

    #[test]
    fn test_full_pipeline_unit_struct() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct Marker;
        };
        
        let visited = Visited::<IntegrationVisitor>::from(&input);
        let plan = IntegrationPlan::try_from(&visited).unwrap();
        
        assert_eq!(plan.field_count, 0);
        assert!(plan.is_struct);
    }

    #[test]
    fn test_multiple_attributes() {
        let input: syn::DeriveInput = syn::parse_quote! {
            #[derive(Debug, Clone, PartialEq)]
            #[serde(rename_all = "camelCase")]
            #[custom_attr(option = "value")]
            struct Attributed {
                field: i32,
            }
        };
        
        let visited = Visited::<IntegrationVisitor>::from(&input);
        
        assert_eq!(visited.inner().attrs.len(), 3);
    }
}

// ============================================================================
// SECTION 5: Edge Cases and Error Handling
// ============================================================================

mod edge_cases {
    use super::*;

    /// Visitor for testing edge cases.
    #[derive(FlowVisitor)]
    struct EdgeCaseVisitor<'a> {
        #[visit(ident)]
        ident: &'a syn::Ident,
        
        #[visit(generics)]
        generics: &'a syn::Generics,
    }

    #[test]
    fn test_empty_struct() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct Empty {}
        };
        
        let visitor = EdgeCaseVisitor::from(&input);
        assert_eq!(visitor.ident.to_string(), "Empty");
    }

    #[test]
    fn test_struct_with_lifetime_only() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct WithLifetime<'a> {
                data: &'a str,
            }
        };
        
        let visitor = EdgeCaseVisitor::from(&input);
        assert_eq!(visitor.generics.params.len(), 1);
    }

    #[test]
    fn test_struct_with_const_generic() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct Array<const N: usize> {
                data: [u8; N],
            }
        };
        
        let visitor = EdgeCaseVisitor::from(&input);
        assert_eq!(visitor.ident.to_string(), "Array");
        assert_eq!(visitor.generics.params.len(), 1);
    }

    #[test]
    fn test_complex_where_clause() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct Complex<T, U, V>
            where
                T: Clone + Send + Sync + 'static,
                U: Default + std::fmt::Debug,
                V: for<'a> From<&'a str>,
            {
                t: T,
                u: U,
                v: V,
            }
        };
        
        let visitor = EdgeCaseVisitor::from(&input);
        
        assert_eq!(visitor.generics.params.len(), 3);
        assert!(visitor.generics.where_clause.is_some());
        
        let where_clause = visitor.generics.where_clause.as_ref().unwrap();
        assert_eq!(where_clause.predicates.len(), 3);
    }

    #[test]
    fn test_tuple_struct_single_field() {
        let input: syn::DeriveInput = syn::parse_quote! {
            struct Wrapper(String);
        };
        
        let visitor = EdgeCaseVisitor::from(&input);
        assert_eq!(visitor.ident.to_string(), "Wrapper");
    }

    #[test]
    fn test_enum_with_all_variant_types() {
        let input: syn::DeriveInput = syn::parse_quote! {
            enum AllVariants {
                Unit,
                Tuple(i32, String),
                Struct { x: f64, y: f64 },
            }
        };
        
        let visitor = EdgeCaseVisitor::from(&input);
        assert_eq!(visitor.ident.to_string(), "AllVariants");
    }
}
