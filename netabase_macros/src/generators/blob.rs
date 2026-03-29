use proc_macro_flow::match_enum;
use crate::visitors::{
    blob::{BlobItemKind, BlobVisitor},
    BLOB_ATTR, BLOB_FIELD_ARG, CHUNK_CHECKSUM_ARG, CHUNK_DERIVES_ARG,
    CHUNK_DESERIALIZE_ARG, CHUNK_OWNER_ID_ARG, CHUNK_SERIALIZE_ARG, CHUNK_SIZE_ARG, STRATEGY_ARG,
};
use heck::ToUpperCamelCase;
use quote::{format_ident, quote};
use syn::{Attribute, Generics, Ident, Item, LitInt, Meta, Path, Type};

pub struct FieldPlanningContext<'a> {
    pub parent_ident: &'a Ident,
    pub prefix: Option<&'a Ident>,
    pub idx: usize,
    pub type_params: &'a [Ident],
    pub assert_tys: &'a mut Vec<Type>,
    pub inherited_size: Option<usize>,
}

pub struct VariantPlanningContext<'a> {
    pub parent_ident: &'a Ident,
    pub type_params: &'a [Ident],
    pub assert_tys: &'a mut Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct ChunkStructPlan {
    pub ident: Ident,
}

impl ChunkStructPlan {
    pub fn new(ident: Ident) -> Self {
        Self { ident }
    }

    pub fn generate(&self, config: &BlobConfig, parent_ident: &Ident) -> Vec<Item> {
        config.generate_chunk_struct(&self.ident, parent_ident)
    }
}

#[derive(Debug, Clone)]
pub struct FieldPlan {
    pub field_ident: Option<Ident>,
    pub variant_ident: Ident,
    pub ty: Type,
    pub size_override: Option<usize>,
    pub nested: bool,
    pub chunk_plan: ChunkStructPlan,
}

impl FieldPlan {
    pub fn from_visitor(
        field: &crate::visitors::blob::BlobField,
        ctx: &mut FieldPlanningContext,
    ) -> Result<(Self, bool), syn::Error> {
        let (nested, field_chunk_size) = BlobConfig::parse_blob_field(&field.attrs)?;
        let local_size = BlobConfig::parse_chunk_size(&field.attrs)?.or(field_chunk_size);
        let size_override = local_size.or(ctx.inherited_size);
        let is_partial = nested || local_size.is_some();

        if nested && BlobPlan::type_uses_generics(&field.ty, ctx.type_params) {
            ctx.assert_tys.push(field.ty.clone());
        }

        let variant_ident = if let Some(id) = &field.ident {
            format_ident!("{}", id.to_string().to_upper_camel_case())
        } else {
            format_ident!("Field{}", ctx.idx)
        };

        let chunk_ident = if let Some(prefix) = ctx.prefix {
            format_ident!("{}{}{}Chunk", ctx.parent_ident, prefix, variant_ident)
        } else {
            format_ident!("{}{}Chunk", ctx.parent_ident, variant_ident)
        };

        let plan = FieldPlan {
            field_ident: field.ident.clone(),
            variant_ident,
            ty: field.ty.clone(),
            size_override,
            nested,
            chunk_plan: ChunkStructPlan::new(chunk_ident),
        };

        Ok((plan, is_partial))
    }

    pub fn chunk_ident(&self) -> &Ident {
        &self.chunk_plan.ident
    }

    pub fn generate(&self, config: &BlobConfig, parent_ident: &Ident) -> Vec<Item> {
        self.chunk_plan.generate(config, parent_ident)
    }
}

#[derive(Debug, Clone)]
pub struct PartialStructPlan {
    pub root_chunk_ident: Ident,
    pub default_size: Option<usize>,
    pub fields: Vec<FieldPlan>,
}

impl PartialStructPlan {
    pub fn from_visitor(
        visitor: &BlobVisitor,
        type_params: &[Ident],
        assert_tys: &mut Vec<Type>,
    ) -> Result<(Self, bool), syn::Error> {
        let fields = match &visitor.kind {
            BlobItemKind::Struct { fields } => fields,
            _ => return Err(syn::Error::new(visitor.ident.span(), "Expected struct")),
        };

        let default_size = BlobConfig::parse_chunk_size(&visitor.attrs)?;
        let mut planned_fields = Vec::new();
        let mut is_partial = false;

        for (idx, field) in fields.iter().enumerate() {
            let (plan, field_partial) = FieldPlan::from_visitor(
                field,
                &mut FieldPlanningContext {
                    parent_ident: &visitor.ident,
                    prefix: None,
                    idx,
                    type_params,
                    assert_tys,
                    inherited_size: default_size,
                },
            )?;
            is_partial |= field_partial;
            planned_fields.push(plan);
        }

        let plan = PartialStructPlan {
            root_chunk_ident: format_ident!("{}Chunk", visitor.ident),
            default_size,
            fields: planned_fields,
        };

        Ok((plan, is_partial))
    }

    pub fn generate(&self, config: &BlobConfig, parent_ident: &Ident) -> Vec<Item> {
        let mut items = Vec::new();

        for field in &self.fields {
            items.extend(field.generate(config, parent_ident));
        }

        let variants: Vec<_> = self
            .fields
            .iter()
            .map(|f| {
                let variant_ident = &f.variant_ident;
                let chunk_ident = f.chunk_ident();
                quote! { #variant_ident(#chunk_ident) }
            })
            .collect();

        let derives = config.build_derive_list();
        let root_ident = &self.root_chunk_ident;
        items.push(syn::parse_quote! {
            #[derive(#(#derives),*)]
            pub enum #root_ident {
                #(#variants),*,
                Missing
            }
        });

        items
    }
}

#[derive(Debug, Clone)]
pub struct VariantPlan {
    pub ident: Ident,
    pub chunk_ident: Ident,
    pub kind: VariantPlanKind,
}

#[derive(Debug, Clone)]
pub enum VariantPlanKind {
    Full,
    Partial { fields: Vec<FieldPlan> },
}

impl VariantPlan {
    pub fn from_visitor(
        variant: &crate::visitors::blob::BlobVariant,
        ctx: &mut VariantPlanningContext,
    ) -> Result<(Self, bool), syn::Error> {
        let (_nested, variant_chunk_size) = BlobConfig::parse_blob_field(&variant.attrs)?;
        let size_override = BlobConfig::parse_chunk_size(&variant.attrs)?.or(variant_chunk_size);

        let variant_chunk_ident = format_ident!("{}{}Chunk", ctx.parent_ident, variant.ident);
        let mut planned_fields = Vec::new();
        let mut is_variant_partial = false;

        for (idx, field) in variant.fields.iter().enumerate() {
            let (plan, field_partial) = FieldPlan::from_visitor(
                field,
                &mut FieldPlanningContext {
                    parent_ident: ctx.parent_ident,
                    prefix: Some(&variant.ident),
                    idx,
                    type_params: ctx.type_params,
                    assert_tys: ctx.assert_tys,
                    inherited_size: size_override,
                },
            )?;
            is_variant_partial |= field_partial;
            planned_fields.push(plan);
        }

        let plan = VariantPlan {
            ident: variant.ident.clone(),
            chunk_ident: variant_chunk_ident,
            kind: if is_variant_partial {
                VariantPlanKind::Partial {
                    fields: planned_fields,
                }
            } else {
                VariantPlanKind::Full
            },
        };

        Ok((plan, is_variant_partial))
    }

    pub fn chunk_ident_type(&self) -> Type {
        let ident = &self.chunk_ident;
        syn::parse_quote! { #ident }
    }

    pub fn generate(&self, config: &BlobConfig, parent_ident: &Ident) -> Vec<Item> {
        match &self.kind {
            VariantPlanKind::Full => config.generate_chunk_struct(&self.chunk_ident, parent_ident),
            VariantPlanKind::Partial { fields } => {
                let mut items = Vec::with_capacity(fields.len() + 1);

                for field in fields {
                    items.extend(field.generate(config, parent_ident));
                }

                let variants: Vec<_> = fields
                    .iter()
                    .map(|f| {
                        let variant_ident = &f.variant_ident;
                        let chunk_ident = f.chunk_ident();
                        quote! { #variant_ident(#chunk_ident) }
                    })
                    .collect();

                let derives = config.build_derive_list();
                let chunk_ident = &self.chunk_ident;
                items.push(syn::parse_quote! {
                    #[derive(#(#derives),*)]
                    pub enum #chunk_ident {
                        #(#variants),*,
                        Missing
                    }
                });

                let field_variant_idents: Vec<_> =
                    fields.iter().map(|f| &f.variant_ident).collect();
                items.push(syn::parse_quote! {
                    impl rewrite::traits::structural::blob::BlobItemChunk for #chunk_ident {
                        type Index = usize;
                        fn get_index(&self) -> &Self::Index {
                            match self {
                                #( Self::#field_variant_idents(inner) => inner.get_index(), )*
                                Self::Missing => panic!("Called get_index() on Missing chunk"),
                            }
                        }
                    }
                });

                items
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PartialEnumPlan {
    pub root_chunk_ident: Ident,
    pub default_size: Option<usize>,
    pub variants: Vec<VariantPlan>,
}

impl PartialEnumPlan {
    pub fn from_visitor(
        visitor: &BlobVisitor,
        type_params: &[Ident],
        assert_tys: &mut Vec<Type>,
    ) -> Result<(Self, bool), syn::Error> {
        let variants = match &visitor.kind {
            BlobItemKind::Enum { variants } => variants,
            _ => return Err(syn::Error::new(visitor.ident.span(), "Expected enum")),
        };

        let default_size = BlobConfig::parse_chunk_size(&visitor.attrs)?;
        let mut planned_variants = Vec::new();
        let mut is_any_variant_partial = false;

        for variant in variants {
            let (plan, is_variant_partial) = VariantPlan::from_visitor(
                variant,
                &mut VariantPlanningContext {
                    parent_ident: &visitor.ident,
                    type_params,
                    assert_tys,
                },
            )?;
            is_any_variant_partial |= is_variant_partial;
            planned_variants.push(plan);
        }

        let plan = PartialEnumPlan {
            root_chunk_ident: format_ident!("{}Chunk", visitor.ident),
            default_size,
            variants: planned_variants,
        };

        Ok((plan, is_any_variant_partial))
    }

    pub fn generate(&self, config: &BlobConfig, parent_ident: &Ident) -> Vec<Item> {
        let mut items = Vec::new();

        for variant in &self.variants {
            items.extend(variant.generate(config, parent_ident));
        }

        let root_variants: Vec<_> = self
            .variants
            .iter()
            .map(|v| {
                let variant_ident = &v.ident;
                let chunk_ident = &v.chunk_ident;
                quote! { #variant_ident(#chunk_ident) }
            })
            .collect();

        let derives = config.build_derive_list();
        let root_ident = &self.root_chunk_ident;
        items.push(syn::parse_quote! {
            #[derive(#(#derives),*)]
            pub enum #root_ident {
                #(#root_variants),*,
                Missing
            }
        });

        items
    }
}

#[derive(Debug, Clone)]
pub struct FullBlobPlan {
    pub default_size: Option<usize>,
    pub chunk_plan: ChunkStructPlan,
}

impl FullBlobPlan {
    pub fn from_visitor(visitor: &BlobVisitor) -> Result<Self, syn::Error> {
        let default_size = BlobConfig::parse_chunk_size(&visitor.attrs)?;
        let root_chunk_ident = format_ident!("{}Chunk", visitor.ident);
        Ok(FullBlobPlan {
            default_size,
            chunk_plan: ChunkStructPlan::new(root_chunk_ident),
        })
    }

    pub fn generate(&self, config: &BlobConfig, parent_ident: &Ident) -> Vec<Item> {
        self.chunk_plan.generate(config, parent_ident)
    }
}

#[derive(Debug, Clone)]
pub enum BlobPlanKind {
    Full(FullBlobPlan),
    PartialStruct(PartialStructPlan),
    PartialEnum(PartialEnumPlan),
}

impl BlobPlanKind {
    pub fn from_visitor(
        visitor: &BlobVisitor,
        config: &BlobConfig,
        assert_tys: &mut Vec<Type>,
    ) -> Result<Self, syn::Error> {
        let type_params: Vec<_> = visitor
            .generics
            .type_params()
            .map(|tp| tp.ident.clone())
            .collect();

        if config.strategy == BlobStrategy::Full {
            return Ok(BlobPlanKind::Full(FullBlobPlan::from_visitor(visitor)?));
        }

        match &visitor.kind {
            BlobItemKind::Struct { .. } => {
                let (plan, is_partial) =
                    PartialStructPlan::from_visitor(visitor, &type_params, assert_tys)?;
                if is_partial || config.strategy == BlobStrategy::Partial {
                    Ok(BlobPlanKind::PartialStruct(plan))
                } else {
                    Ok(BlobPlanKind::Full(FullBlobPlan::from_visitor(visitor)?))
                }
            }
            BlobItemKind::Enum { .. } => {
                let (plan, is_partial) =
                    PartialEnumPlan::from_visitor(visitor, &type_params, assert_tys)?;
                if is_partial || config.strategy == BlobStrategy::Partial {
                    Ok(BlobPlanKind::PartialEnum(plan))
                } else {
                    Ok(BlobPlanKind::Full(FullBlobPlan::from_visitor(visitor)?))
                }
            }
            BlobItemKind::Unknown => Err(syn::Error::new(
                visitor.ident.span(),
                "Unknown blob item kind",
            )),
        }
    }

    pub fn from_visitor_accumulated(
        visitor: &BlobVisitor,
        config: &BlobConfig,
    ) -> Result<(Self, Vec<Type>), syn::Error> {
        let mut assert_tys = Vec::new();
        let kind = Self::from_visitor(visitor, config, &mut assert_tys)?;
        Ok((kind, assert_tys))
    }

    pub fn generate(&self, config: &BlobConfig, parent_ident: &Ident) -> Vec<Item> {
        match self {
            BlobPlanKind::Full(plan) => plan.generate(config, parent_ident),
            BlobPlanKind::PartialStruct(plan) => plan.generate(config, parent_ident),
            BlobPlanKind::PartialEnum(plan) => plan.generate(config, parent_ident),
        }
    }
}

#[derive(Debug, proc_macro_flow::FlowPlan)]
#[plan(visitor = BlobVisitor)]
pub struct BlobPlan {
    #[plan(copy)]
    pub ident: Ident,
    #[plan(copy)]
    pub generics: Generics,
    #[plan(try_from = "BlobConfig::parse(&v.attrs)")]
    pub config: BlobConfig,
    #[plan(try_from = "BlobPlanKind::from_visitor_accumulated(v, &config)")]
    pub kind_info: (BlobPlanKind, Vec<Type>),
    #[plan(from = "kind_info.1.clone()")]
    pub assert_tys: Vec<Type>,
    #[plan(from = "kind_info.0.clone()")]
    pub kind: BlobPlanKind,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BlobStrategy {
    #[default]
    Auto,
    Full,
    Partial,
}

#[derive(Debug, Default, Clone)]
pub struct BlobConfig {
    pub custom_derives: Vec<Path>,
    pub serialize_fn: Option<Path>,
    pub deserialize_fn: Option<Path>,
    pub include_owner_id: bool,
    pub include_checksum: bool,
    pub strategy: BlobStrategy,
}

impl BlobConfig {
    pub fn parse(attrs: &[Attribute]) -> Result<Self, syn::Error> {
        use proc_macro_flow::AttributeExt;
        let mut config = BlobConfig::default();

        // 1. Check for #[blob(...)] consolidated attributes
        for meta in attrs.get_all_meta(BLOB_ATTR)? {
            if let Some(items) = meta.as_list() {
                for item in items {
                    let name = item.name_str();
                    match name.as_str() {
                        CHUNK_DERIVES_ARG => {
                            if let Some(nested) = item.nested() {
                                for n in nested {
                                    if let Some(lit) = n.as_value() {
                                        // This is a bit tricky, nested might be paths.
                                        // MetaItem::nested() returns Vec<MetaItem>.
                                        // For #[blob(chunk_derives(Serialize))], n is Flag(Serialize).
                                        if let Some(id) = n.name() {
                                            config.custom_derives.push(syn::Path::from(id.clone()));
                                        }
                                    } else if let Some(id) = n.name() {
                                        config.custom_derives.push(syn::Path::from(id.clone()));
                                    }
                                }
                            }
                        }
                        CHUNK_SERIALIZE_ARG => {
                            if let Some(val) = item.as_str() {
                                config.serialize_fn = Some(val.parse()?);
                            } else if let Some(nested) = item.nested() {
                                if let Some(first) = nested.first() {
                                    if let Some(id) = first.name() {
                                        config.serialize_fn = Some(syn::Path::from(id.clone()));
                                    }
                                }
                            }
                        }
                        CHUNK_DESERIALIZE_ARG => {
                            if let Some(val) = item.as_str() {
                                config.deserialize_fn = Some(val.parse()?);
                            } else if let Some(nested) = item.nested() {
                                if let Some(first) = nested.first() {
                                    if let Some(id) = first.name() {
                                        config.deserialize_fn = Some(syn::Path::from(id.clone()));
                                    }
                                }
                            }
                        }
                        CHUNK_OWNER_ID_ARG => config.include_owner_id = true,
                        CHUNK_CHECKSUM_ARG => config.include_checksum = true,
                        STRATEGY_ARG => {
                            let strategy_str = if let Some(val) = item.as_str() {
                                val.value()
                            } else if let Some(nested) = item.nested() {
                                nested.first().map(|n| n.name_str()).unwrap_or_default()
                            } else if let Some(val) = item.as_value() {
                                // handle case like strategy = partial (no quotes)
                                if let syn::Lit::Str(s) = val { s.value() } else { String::new() }
                            } else {
                                String::new()
                            };

                            config.strategy = match strategy_str.as_str() {
                                "full" => BlobStrategy::Full,
                                "partial" => BlobStrategy::Partial,
                                _ => config.strategy,
                            };
                        }
                        _ => {}
                    }
                }
            }
        }

        // 2. Check for standalone attributes (e.g., #[strategy("partial")], #[chunk_size(1024)])
        for attr in attrs {
            let path = attr.path();
            if path.is_ident(STRATEGY_ARG) {
                let meta = syn::parse2::<proc_macro_flow::meta::Meta>(attr.meta.require_list()?.tokens.clone())?;
                let strategy_str = if let Some(s) = meta.as_str() {
                    s.value()
                } else if let Some(items) = meta.as_list() {
                    items.first().map(|i| i.name_str()).unwrap_or_default()
                } else {
                    String::new()
                };

                match strategy_str.as_str() {
                    "full" => config.strategy = BlobStrategy::Full,
                    "partial" => config.strategy = BlobStrategy::Partial,
                    _ => {}
                }
            } else if path.is_ident(CHUNK_OWNER_ID_ARG) {
                config.include_owner_id = true;
            } else if path.is_ident(CHUNK_CHECKSUM_ARG) {
                config.include_checksum = true;
            } else if path.is_ident(CHUNK_SERIALIZE_ARG) {
                if let Meta::List(list) = &attr.meta {
                    config.serialize_fn = Some(list.parse_args()?);
                }
            } else if path.is_ident(CHUNK_DESERIALIZE_ARG) {
                if let Meta::List(list) = &attr.meta {
                    config.deserialize_fn = Some(list.parse_args()?);
                }
            } else if path.is_ident(CHUNK_DERIVES_ARG) {
                if let Meta::List(list) = &attr.meta {
                    let paths: syn::punctuated::Punctuated<Path, syn::Token![,]> =
                        list.parse_args_with(syn::punctuated::Punctuated::parse_terminated)?;
                    for p in paths {
                        config.custom_derives.push(p);
                    }
                }
            }
        }

        Ok(config)
    }

    pub fn parse_chunk_size(attrs: &[Attribute]) -> Result<Option<usize>, syn::Error> {
        use proc_macro_flow::AttributeExt;
        
        // 1. Standalone #[chunk_size(1024)]
        for meta in attrs.get_all_meta(CHUNK_SIZE_ARG)? {
            if let Some(i) = meta.as_int() {
                return Ok(Some(i.base10_parse()?));
            } else if let Some(items) = meta.as_list() {
                if let Some(first) = items.first() {
                    if let Some(i) = first.as_int() {
                        return Ok(Some(i.base10_parse()?));
                    }
                }
            }
        }

        // 2. Consolidated #[blob(chunk_size(1024))]
        for meta in attrs.get_all_meta(BLOB_ATTR)? {
            if let Some(items) = meta.as_list() {
                for item in items {
                    if item.is_path_named(CHUNK_SIZE_ARG) {
                        if let Some(i) = item.as_int() {
                            return Ok(Some(i.base10_parse()?));
                        } else if let Some(nested) = item.nested() {
                            if let Some(first) = nested.first() {
                                if let Some(i) = first.as_int() {
                                    return Ok(Some(i.base10_parse()?));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    pub fn parse_blob_field(attrs: &[Attribute]) -> Result<(bool, Option<usize>), syn::Error> {
        use proc_macro_flow::AttributeExt;
        let mut found = false;
        let mut size = None;

        for meta in attrs.get_all_meta(BLOB_FIELD_ARG)? {
            found = true;
            if let Some(items) = meta.as_list() {
                for item in items {
                    if item.is_path_named(CHUNK_SIZE_ARG) {
                        if let Some(i) = item.as_int() {
                            size = Some(i.base10_parse()?);
                        } else if let Some(nested) = item.nested() {
                            if let Some(first) = nested.first() {
                                if let Some(i) = first.as_int() {
                                    size = Some(i.base10_parse()?);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Also check #[blob(blob_field(...))]
        for meta in attrs.get_all_meta(BLOB_ATTR)? {
            if let Some(items) = meta.as_list() {
                for item in items {
                    if item.is_path_named(BLOB_FIELD_ARG) {
                        found = true;
                        if let Some(nested) = item.nested() {
                            for n in nested {
                                if n.is_path_named(CHUNK_SIZE_ARG) {
                                    if let Some(i) = n.as_int() {
                                        size = Some(i.base10_parse()?);
                                    } else if let Some(nn) = n.nested() {
                                        if let Some(first) = nn.first() {
                                            if let Some(i) = first.as_int() {
                                                size = Some(i.base10_parse()?);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((found, size))
    }

    pub fn build_derive_list(&self) -> Vec<Path> {
        let mut derives = vec![
            syn::parse_quote!(Debug),
            syn::parse_quote!(Clone),
            syn::parse_quote!(PartialEq),
            syn::parse_quote!(Eq),
            syn::parse_quote!(PartialOrd),
            syn::parse_quote!(Ord),
            syn::parse_quote!(::rkyv::Archive),
            syn::parse_quote!(::rkyv::Serialize),
            syn::parse_quote!(::rkyv::Deserialize),
        ];

        derives.retain(|d| {
            !self.custom_derives.iter().any(|cd| {
                let d_str = quote!(#d).to_string().replace(" ", "");
                let cd_str = quote!(#cd).to_string().replace(" ", "");
                cd_str.ends_with(&d_str) || d_str.ends_with(&cd_str)
            })
        });

        derives.extend(self.custom_derives.iter().cloned());
        derives
    }

    pub fn generate_chunk_struct(&self, ident: &Ident, _parent_ident: &Ident) -> Vec<Item> {
        let derives = self.build_derive_list();
        let mut fields = vec![quote! { pub index: usize }, quote! { pub data: Vec<u8> }];

        if self.include_owner_id {
            fields.insert(0, quote! { pub owner_id: u64 });
        }

        if self.include_checksum {
            fields.push(quote! { pub checksum: u64 });
        }

        let struct_item: Item = syn::parse_quote! {
            #[derive(#(#derives),*)]
            pub struct #ident {
                #(#fields),*
            }
        };

        let chunk_impl: Item = syn::parse_quote! {
            impl ::rewrite::traits::structural::blob::BlobItemChunk for #ident {
                type Index = usize;

                fn get_index(&self) -> &Self::Index {
                    &self.index
                }
            }
        };

        vec![struct_item, chunk_impl]
    }
}

#[derive(proc_macro_flow::FlowGenerator)]
#[generator(output = Vec<Item>, plan = BlobPlan, multi)]
pub struct BlobGenerator {
    #[gen(from = "plan")]
    pub plan: BlobPlan,
}

impl BlobGenerator {
    fn generate_impl(self) -> Result<Vec<Item>, syn::Error> {
        let mut items = Vec::new();

        // Chunk Types
        items.extend(self.plan.kind.generate(&self.plan.config, &self.plan.ident));

        // ChunkFill Logic
        let ident = &self.plan.ident;
        let chunk_fill_ident = format_ident!("{}ChunkFill", ident);
        let enum_def: Item = syn::parse_quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ::rkyv::Archive, ::rkyv::Serialize, ::rkyv::Deserialize)]
            pub enum #chunk_fill_ident {
                Full(usize),
                Partial(usize),
                Corrupted(usize),
            }
        };
        let enum_impl: Item = syn::parse_quote! {
            impl #chunk_fill_ident {
                pub fn from_size(actual: usize, expected: usize) -> Self {
                    if actual == expected {
                        Self::Full(actual)
                    } else if actual < expected {
                        Self::Partial(actual)
                    } else {
                        Self::Corrupted(actual)
                    }
                }
            }
        };
        items.push(enum_def);
        items.push(enum_impl);

        // Trait Implementations
        let trait_impls = self.plan.generate_trait_impls(&chunk_fill_ident);
        items.extend(trait_impls);

        // Type Assertions
        if !self.plan.assert_tys.is_empty() {
            let (impl_generics, ty_generics, where_clause) = self.plan.generics.split_for_impl();
            let assert_tys = &self.plan.assert_tys;
            let assertion: Item = syn::parse_quote! {
                const _: () = {
                    impl #impl_generics #ident #ty_generics #where_clause {
                        #[doc(hidden)]
                        const fn _netabase_blob_assert_bounds() {
                            fn assert_impl<__T: rewrite::traits::structural::blob::NetabaseBlobItem>() {}
                            #(
                                assert_impl::<#assert_tys>();
                            )*
                        }
                    }
                };
            };
            items.push(assertion);
        }

        Ok(items)
    }
}

impl BlobPlan {
    pub fn type_uses_generics(ty: &Type, params: &[Ident]) -> bool {
        struct GenericFinder<'a> {
            params: &'a [Ident],
            found: bool,
        }

        impl<'a, 'ast> syn::visit::Visit<'ast> for GenericFinder<'a> {
            fn visit_path(&mut self, i: &'ast syn::Path) {
                if i.leading_colon.is_none()
                    && i.segments.len() == 1
                    && self.params.iter().any(|p| p == &i.segments[0].ident)
                {
                    self.found = true;
                }
                syn::visit::visit_path(self, i);
            }
        }

        let mut finder = GenericFinder {
            params,
            found: false,
        };
        syn::visit::visit_type(&mut finder, ty);
        finder.found
    }

    fn generate_trait_impls(&self, chunk_fill_ident: &Ident) -> Vec<Item> {
        let mut items = Vec::new();

        let root_chunk_ident = match &self.kind {
            BlobPlanKind::Full(plan) => &plan.chunk_plan.ident,
            BlobPlanKind::PartialStruct(plan) => &plan.root_chunk_ident,
            BlobPlanKind::PartialEnum(plan) => &plan.root_chunk_ident,
        };

        if matches!(
            self.kind,
            BlobPlanKind::PartialStruct(_) | BlobPlanKind::PartialEnum(_)
        ) {
            items.push(self.generate_blob_item_chunk_impl(root_chunk_ident));
        }

        items.extend(self.generate_netabase_blob_item_impl(root_chunk_ident, chunk_fill_ident));

        items
    }

    fn generate_blob_item_chunk_impl(&self, chunk_ident: &Ident) -> Item {
        let body = match &self.kind {
            BlobPlanKind::Full(_) => quote! { &self.index },
            BlobPlanKind::PartialStruct(plan) => {
                let variants = plan.fields.iter().map(|f| &f.variant_ident);
                quote! {
                    match self {
                        #( Self::#variants(c) => c.get_index(), )*
                        _ => panic!("get_index called on missing chunk"),
                    }
                }
            }
            BlobPlanKind::PartialEnum(plan) => {
                let variants = plan.variants.iter().map(|v| &v.ident);
                quote! {
                    match self {
                        #( Self::#variants(c) => c.get_index(), )*
                        _ => panic!("get_index called on missing chunk"),
                    }
                }
            }
        };

        syn::parse_quote! {
            impl rewrite::traits::structural::blob::BlobItemChunk for #chunk_ident {
                type Index = usize;

                fn get_index(&self) -> &Self::Index {
                    #body
                }
            }
        }
    }

    fn generate_netabase_blob_item_impl(
        &self,
        chunk_ident: &Ident,
        chunk_fill_ident: &Ident,
    ) -> Vec<Item> {
        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let mut extended_where_clause = where_clause.cloned();
        for param in &self.generics.params {
            if let syn::GenericParam::Type(type_param) = param {
                let type_ident = &type_param.ident;
                let clause = extended_where_clause.get_or_insert_with(|| syn::WhereClause {
                    where_token: Default::default(),
                    predicates: Default::default(),
                });
                clause.predicates.push(syn::parse_quote! {
                    #type_ident: rkyv::Archive
                });
                clause.predicates.push(syn::parse_quote! {
                    #type_ident: for<'__a> rkyv::Serialize<rkyv::rancor::Strategy<rkyv::ser::Serializer<rkyv::util::AlignedVec, rkyv::ser::allocator::ArenaHandle<'__a>, rkyv::ser::sharing::Share>, rkyv::rancor::Error>>
                });
                clause.predicates.push(syn::parse_quote! {
                    <#type_ident as rkyv::Archive>::Archived: for<'__a> rkyv::bytecheck::CheckBytes<rkyv::rancor::Strategy<rkyv::validation::Validator<rkyv::validation::archive::ArchiveValidator<'__a>, rkyv::validation::shared::SharedValidator>, rkyv::rancor::Error>>
                });
                clause.predicates.push(syn::parse_quote! {
                    <#type_ident as rkyv::Archive>::Archived: rkyv::Deserialize<#type_ident, rkyv::rancor::Strategy<rkyv::de::Pool, rkyv::rancor::Error>>
                });
            }
        }

        let owner_id_field = if self.config.include_owner_id {
            quote! { owner_id: 0, }
        } else {
            quote! {}
        };

        let checksum_field = if self.config.include_checksum {
            quote! { checksum: 0, }
        } else {
            quote! {}
        };

        match &self.kind {
            BlobPlanKind::Full(plan) => {
                let default_chunk_size = plan.default_size.unwrap_or(0);
                let serialize_expr = if let Some(ser_fn) = &self.config.serialize_fn {
                    quote! { #ser_fn(&self)? }
                } else {
                    quote! { rkyv::to_bytes::<rkyv::rancor::Error>(&self).map_err(|e|
                        rewrite::results::NetabaseError::Serialization(format!("rkyv serialization failed: {:?}", e))
                    )?.to_vec() }
                };

                let deserialize_expr = if let Some(de_fn) = &self.config.deserialize_fn {
                    quote! { #de_fn(&serialized_data)? }
                } else {
                    quote! {
                        rkyv::from_bytes::<Self, rkyv::rancor::Error>(&serialized_data).map_err(|e|
                            rewrite::results::NetabaseError::Serialization(format!("rkyv deserialization failed: {:?}", e))
                        )?
                    }
                };

                vec![
                    syn::parse_quote! {
                        impl #impl_generics rewrite::traits::structural::blob::NetabaseBlobItem for #ident #ty_generics #extended_where_clause {
                            type Chunk = #chunk_ident;
                            type BlobIter = std::vec::IntoIter<rewrite::results::NetabaseResult<Self::Chunk>>;
                            const DEFAULT_CHUNK_SIZE: usize = #default_chunk_size;

                            fn into_chunks(self, size: rewrite::traits::structural::blob::ChunkSize) -> Box<dyn Iterator<Item = Self::Chunk>> {
                                Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
                            }

                            fn into_chunks_iter(self, size: rewrite::traits::structural::blob::ChunkSize) -> Self::BlobIter {
                                let serialized_data: rewrite::results::NetabaseResult<Vec<u8>> = (|| -> rewrite::results::NetabaseResult<Vec<u8>> {
                                    Ok(#serialize_expr)
                                })();

                                let chunk_size = match size {
                                    rewrite::traits::structural::blob::ChunkSize::Default => {
                                        if Self::DEFAULT_CHUNK_SIZE > 0 {
                                            Self::DEFAULT_CHUNK_SIZE
                                        } else {
                                            1024
                                        }
                                    }
                                    rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                                };

                                match serialized_data {
                                    Ok(data) => {
                                        data.chunks(chunk_size)
                                            .enumerate()
                                            .map(|(index, chunk_data)| {
                                                Ok(Self::Chunk {
                                                    #owner_id_field
                                                    index,
                                                    data: chunk_data.to_vec(),
                                                    #checksum_field
                                                })
                                            })
                                            .collect::<Vec<_>>()
                                            .into_iter()
                                    }
                                    Err(e) => vec![Err(e)].into_iter(),
                                }
                            }

                            fn try_from_chunks(
                                chunks: impl Iterator<Item = Self::Chunk>,
                                size: rewrite::traits::structural::blob::ChunkSize,
                            ) -> rewrite::results::NetabaseResult<Self> {
                                let mut sorted_chunks: Vec<_> = chunks.collect();
                                sorted_chunks.sort_by_key(|c| c.index);

                                if sorted_chunks.is_empty() {
                                    return Err(rewrite::results::NetabaseError::BlobReconstruction(
                                        rewrite::results::BlobReconstructionError::MissingChunks
                                    ));
                                }

                                let chunk_size = match size {
                                    rewrite::traits::structural::blob::ChunkSize::Default => {
                                        if Self::DEFAULT_CHUNK_SIZE > 0 {
                                            Self::DEFAULT_CHUNK_SIZE
                                        } else {
                                            1024
                                        }
                                    }
                                    rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                                };
                                let mut missing_details = Vec::new();
                                let mut next_expected = 0;
                                let max_idx = sorted_chunks.last().map(|c| c.index).unwrap_or(0);

                                for chunk in &sorted_chunks {
                                    while chunk.index > next_expected {
                                        missing_details.push(format!("{:?}({{ Index: {}, Size: {} }})", #chunk_fill_ident::Full(chunk_size), next_expected, chunk_size));
                                        next_expected += 1;
                                    }

                                    let fill = #chunk_fill_ident::from_size(chunk.data.len(), chunk_size);
                                    match fill {
                                        #chunk_fill_ident::Corrupted(size) => {
                                            return Err(rewrite::results::NetabaseError::BlobReconstruction(
                                                rewrite::results::BlobReconstructionError::InvalidChunkData(
                                                    format!("Corrupted chunk detected: {:?}({{ Index: {}, Size: {} }}). Max allowed size is {}.", fill, chunk.index, size, chunk_size)
                                                )
                                            ));
                                        }
                                        #chunk_fill_ident::Partial(size) if chunk.index < max_idx => {
                                            return Err(rewrite::results::NetabaseError::BlobReconstruction(
                                                rewrite::results::BlobReconstructionError::InvalidChunkData(
                                                    format!("Unexpected partial chunk in middle of stream: {:?}({{ Index: {}, Size: {} }}). Expected {} bytes.", fill, chunk.index, size, chunk_size)
                                                )
                                            ));
                                        }
                                        _ => {}
                                    }

                                    if chunk.index == next_expected {
                                        next_expected += 1;
                                    }
                                }

                                if !missing_details.is_empty() {
                                    if let Some(last) = sorted_chunks.last() {
                                        let fill = #chunk_fill_ident::from_size(last.data.len(), chunk_size);
                                        if matches!(fill, #chunk_fill_ident::Full(_)) {
                                            missing_details.push(format!("... (Stream truncated: last chunk was Full, expected more data after Index {})", last.index));
                                        }
                                    }
                                }

                                if !missing_details.is_empty() {
                                    return Err(rewrite::results::NetabaseError::BlobReconstruction(
                                        rewrite::results::BlobReconstructionError::InvalidChunkData(
                                            format!(
                                                "Missing chunks: [{}]. Total chunks present: {}",
                                                missing_details.join(", "),
                                                sorted_chunks.len()
                                            )
                                        )
                                    ));
                                }

                                let serialized_data: Vec<u8> = sorted_chunks
                                    .into_iter()
                                    .flat_map(|c| c.data)
                                    .collect();

                                Ok(#deserialize_expr)
                            }

                            fn get_blob(&self) -> &Self::Chunk {
                                unimplemented!("get_blob() requires storing a chunk reference")
                            }
                        }
                    },
                    syn::parse_quote! {
                        impl #impl_generics IntoIterator for #ident #ty_generics #extended_where_clause {
                            type Item = rewrite::results::NetabaseResult<#chunk_ident>;
                            type IntoIter = std::vec::IntoIter<Self::Item>;

                            fn into_iter(self) -> Self::IntoIter {
                                rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(self, rewrite::traits::structural::blob::ChunkSize::Default)
                            }
                        }
                    },
                ]
            }
            BlobPlanKind::PartialStruct(plan) => {
                let field_idents: Vec<_> = plan
                    .fields
                    .iter()
                    .map(|f| {
                        if let Some(id) = &f.field_ident {
                            quote! { #id }
                        } else {
                            let idx = syn::Index::from(
                                plan.fields
                                    .iter()
                                    .position(|x| x as *const _ == f as *const _)
                                    .unwrap(),
                            );
                            quote! { #idx }
                        }
                    })
                    .collect();
                let field_names: Vec<_> = plan
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        f.field_ident
                            .as_ref()
                            .map(|id| id.to_string())
                            .unwrap_or_else(|| format!("field_{}", i))
                    })
                    .collect();
                let variant_idents: Vec<_> = plan.fields.iter().map(|f| &f.variant_ident).collect();
                let chunk_idents: Vec<_> = plan.fields.iter().map(|f| f.chunk_ident()).collect();
                let field_tys: Vec<_> = plan.fields.iter().map(|f| &f.ty).collect();
                let field_chunk_vec_idents: Vec<_> = variant_idents
                    .iter()
                    .map(|id| format_ident!("chunks_{}", id.to_string().to_lowercase()))
                    .collect();
                let field_chunk_sizes: Vec<_> = plan
                    .fields
                    .iter()
                    .map(|f| f.size_override.unwrap_or(0))
                    .collect();
                let default_chunk_size = plan.default_size.unwrap_or(0);
                vec![
                    syn::parse_quote! {
                        impl #impl_generics rewrite::traits::structural::blob::NetabaseBlobItem for #ident #ty_generics #extended_where_clause {
                            type Chunk = #chunk_ident;
                            type BlobIter = std::vec::IntoIter<rewrite::results::NetabaseResult<Self::Chunk>>;
                            const DEFAULT_CHUNK_SIZE: usize = #default_chunk_size;

                            fn into_chunks(self, size: rewrite::traits::structural::blob::ChunkSize) -> Box<dyn Iterator<Item = Self::Chunk>> {
                                Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
                            }

                            fn into_chunks_iter(self, size: rewrite::traits::structural::blob::ChunkSize) -> Self::BlobIter {
                                let mut all_chunks = Vec::new();

                                #(
                                    {
                                        let serialized_field: rewrite::results::NetabaseResult<Vec<u8>> = rkyv::to_bytes::<rkyv::rancor::Error>(&self.#field_idents).map_err(|e|
                                            rewrite::results::NetabaseError::Serialization(format!("rkyv serialization failed for field {}: {:?}", #field_names, e))
                                        ).map(|d| d.to_vec());

                                        match serialized_field {
                                            Ok(data) => {
                                                let chunk_size = match size {
                                                    rewrite::traits::structural::blob::ChunkSize::Default => {
                                                        let default = #field_chunk_sizes;
                                                        if default > 0 { default } else { 1024 }
                                                    }
                                                    rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                                                };

                                                all_chunks.extend(data
                                                    .chunks(chunk_size)
                                                    .enumerate()
                                                    .map(|(index, chunk_data)| {
                                                        Ok(Self::Chunk::#variant_idents(#chunk_idents {
                                                            #owner_id_field
                                                            index,
                                                            data: chunk_data.to_vec(),
                                                            #checksum_field
                                                        }))
                                                    })
                                                );
                                            }
                                            Err(e) => {
                                                all_chunks.push(Err(e));
                                            }
                                        }
                                    }
                                )*

                                all_chunks.into_iter()
                            }

                            fn try_from_chunks(
                                chunks: impl Iterator<Item = Self::Chunk>,
                                size: rewrite::traits::structural::blob::ChunkSize,
                            ) -> rewrite::results::NetabaseResult<Self> {
                                #( let mut #field_chunk_vec_idents = Vec::new(); )*

                                for chunk in chunks {
                                    match chunk {
                                        #(
                                            Self::Chunk::#variant_idents(c) => #field_chunk_vec_idents.push(c),
                                        )*
                                        _ => {}
                                    }
                                }

                                #(
                                    let #field_idents = {
                                        if #field_chunk_vec_idents.is_empty() {
                                            return Err(rewrite::results::NetabaseError::BlobReconstruction(
                                                rewrite::results::BlobReconstructionError::MissingChunks
                                            ));
                                        }
                                        let mut sorted = #field_chunk_vec_idents;
                                        sorted.sort_by_key(|c| c.index);

                                        let chunk_size = match size {
                                            rewrite::traits::structural::blob::ChunkSize::Default => {
                                                let default = #field_chunk_sizes;
                                                if default > 0 { default } else { 1024 }
                                            }
                                            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                                        };
                                        let mut missing_details = Vec::new();
                                        let mut next_expected = 0;
                                        let max_idx = sorted.last().map(|c| c.index).unwrap_or(0);

                                        for chunk in &sorted {
                                            while chunk.index > next_expected {
                                                missing_details.push(format!("{:?}({{ Index: {}, Size: {} }})", #chunk_fill_ident::Full(chunk_size), next_expected, chunk_size));
                                                next_expected += 1;
                                            }

                                            let fill = #chunk_fill_ident::from_size(chunk.data.len(), chunk_size);
                                            match fill {
                                                #chunk_fill_ident::Corrupted(size) => {
                                                    return Err(rewrite::results::NetabaseError::BlobReconstruction(
                                                        rewrite::results::BlobReconstructionError::InvalidChunkData(
                                                            format!("Corrupted chunk detected for field {}: {:?}({{ Index: {}, Size: {} }}). Max allowed size is {}.", #field_names, fill, chunk.index, size, chunk_size)
                                                        )
                                                    ));
                                                }
                                                #chunk_fill_ident::Partial(size) if chunk.index < max_idx => {
                                                    return Err(rewrite::results::NetabaseError::BlobReconstruction(
                                                        rewrite::results::BlobReconstructionError::InvalidChunkData(
                                                            format!("Unexpected partial chunk in middle of stream for field {}: {:?}({{ Index: {}, Size: {} }}). Expected {} bytes.", #field_names, fill, chunk.index, size, chunk_size)
                                                        )
                                                    ));
                                                }
                                                _ => {}
                                            }

                                            if chunk.index == next_expected {
                                                next_expected += 1;
                                            }
                                        }

                                        if !missing_details.is_empty() {
                                            if let Some(last) = sorted.last() {
                                                let fill = #chunk_fill_ident::from_size(last.data.len(), chunk_size);
                                                if matches!(fill, #chunk_fill_ident::Full(_)) {
                                                    missing_details.push(format!("... (Stream truncated for field {}: last chunk was Full, expected more data after Index {})", #field_names, last.index));
                                                }
                                            }
                                        }

                                        if !missing_details.is_empty() {
                                            return Err(rewrite::results::NetabaseError::BlobReconstruction(
                                                rewrite::results::BlobReconstructionError::InvalidChunkData(
                                                    format!(
                                                        "Missing chunks for field {}: [{}]. Total chunks present: {}",
                                                        #field_names,
                                                        missing_details.join(", "),
                                                        sorted.len()
                                                    )
                                                )
                                            ));
                                        }
                                        let data: Vec<u8> = sorted.into_iter().flat_map(|c| c.data).collect();
                                        rkyv::from_bytes::<#field_tys, rkyv::rancor::Error>(&data).map_err(|e|
                                            rewrite::results::NetabaseError::Serialization(format!("rkyv deserialization failed for field {}: {:?}", #field_names, e))
                                        )?
                                    };
                                )*

                                Ok(Self { #(#field_idents),* })
                            }

                            fn get_blob(&self) -> &Self::Chunk {
                                unimplemented!("get_blob() requires storing a chunk reference")
                            }
                        }
                    },
                    syn::parse_quote! {
                        impl #impl_generics IntoIterator for #ident #ty_generics #extended_where_clause {
                            type Item = rewrite::results::NetabaseResult<#chunk_ident>;
                            type IntoIter = std::vec::IntoIter<Self::Item>;

                            fn into_iter(self) -> Self::IntoIter {
                                rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(self, rewrite::traits::structural::blob::ChunkSize::Default)
                            }
                        }
                    },
                ]
            }
            BlobPlanKind::PartialEnum(plan) => {
                let variants = &plan.variants;
                let variant_idents: Vec<_> = variants.iter().map(|v| &v.ident).collect();
                let default_chunk_size = plan.default_size.unwrap_or(0);

                let into_chunks_body = match_enum(
                    quote!(self),
                    variants,
                    |v| &v.ident,
                    |v| match &v.kind {
                        VariantPlanKind::Full => vec![(None, v.chunk_ident_type())],
                        VariantPlanKind::Partial { fields } => fields
                            .iter()
                            .map(|f| (f.field_ident.clone(), f.ty.clone()))
                            .collect(),
                    },
                    |v, fields| {
                        let variant_ident = &v.ident;
                        let chunk_ident = &v.chunk_ident;

                        match &v.kind {
                            VariantPlanKind::Full => {
                                quote! {
                                    let serialized: rewrite::results::NetabaseResult<Vec<u8>> = rkyv::to_bytes::<rkyv::rancor::Error>(&self).map_err(|e|
                                        rewrite::results::NetabaseError::Serialization(format!("rkyv serialization failed for enum variant {}: {:?}", stringify!(#variant_ident), e))
                                    ).map(|d| d.to_vec());

                                    match serialized {
                                        Ok(data) => {
                                            let chunk_size = match size {
                                                rewrite::traits::structural::blob::ChunkSize::Default => {
                                                    if #default_chunk_size > 0 {
                                                        #default_chunk_size
                                                    } else {
                                                        1024
                                                    }
                                                }
                                                rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                                            };
                                            all_chunks.extend(data.chunks(chunk_size).enumerate().map(|(index, chunk_data)| {
                                                Ok(Self::Chunk::#variant_ident(#chunk_ident {
                                                    #owner_id_field
                                                    index,
                                                    data: chunk_data.to_vec(),
                                                    #checksum_field
                                                }))
                                            }));
                                        }
                                        Err(e) => all_chunks.push(Err(e)),
                                    }
                                }
                            }
                            VariantPlanKind::Partial {
                                fields: planned_fields,
                            } => {
                                let mut field_logic = Vec::new();
                                for (idx, f) in planned_fields.iter().enumerate() {
                                    let field_binding = &fields[idx].binding;
                                    let field_variant_ident = &f.variant_ident;
                                    let field_chunk_ident = f.chunk_ident();
                                    let field_name = f
                                        .field_ident
                                        .as_ref()
                                        .map(|id| id.to_string())
                                        .unwrap_or_else(|| format!("field_{}", idx));
                                    let default_field_size = f.size_override.unwrap_or(1024);

                                    field_logic.push(quote! {
                                        {
                                            let serialized_field: rewrite::results::NetabaseResult<Vec<u8>> = rkyv::to_bytes::<rkyv::rancor::Error>(#field_binding).map_err(|e|
                                                rewrite::results::NetabaseError::Serialization(format!("rkyv serialization failed for variant {} field {}: {:?}", stringify!(#variant_ident), #field_name, e))
                                            ).map(|d| d.to_vec());

                                            match serialized_field {
                                                Ok(data) => {
                                                    let chunk_size = match size {
                                                        rewrite::traits::structural::blob::ChunkSize::Default => #default_field_size,
                                                        rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                                                    };

                                                    all_chunks.extend(data.chunks(chunk_size).enumerate().map(|(index, chunk_data)| {
                                                        Ok(Self::Chunk::#variant_ident(#chunk_ident::#field_variant_ident(#field_chunk_ident {
                                                            #owner_id_field
                                                            index,
                                                            data: chunk_data.to_vec(),
                                                            #checksum_field
                                                        })))
                                                    }));
                                                }
                                                Err(e) => all_chunks.push(Err(e)),
                                            }
                                        }
                                    });
                                }
                                quote! { #(#field_logic)* }
                            }
                        }
                    },
                );

                let mut variant_reconstructions = Vec::new();
                for v in variants {
                    let variant_ident = &v.ident;
                    let chunk_ident = &v.chunk_ident;

                    let logic = match &v.kind {
                        VariantPlanKind::Full => {
                            quote! {
                                let mut sorted = chunks;
                                sorted.sort_by_key(|c| match c {
                                    Self::Chunk::#variant_ident(inner) => inner.index,
                                    _ => 0,
                                });

                                let data: Vec<u8> = sorted.into_iter().flat_map(|c| match c {
                                    Self::Chunk::#variant_ident(inner) => inner.data,
                                    _ => Vec::new(),
                                }).collect();

                                let val: Self = rkyv::from_bytes::<Self, rkyv::rancor::Error>(&data).map_err(|e|
                                    rewrite::results::NetabaseError::Serialization(format!("rkyv deserialization failed for variant {}: {:?}", stringify!(#variant_ident), e))
                                )?;
                                return Ok(val);
                            }
                        }
                        VariantPlanKind::Partial { fields } => {
                            let mut field_collectors = Vec::new();
                            let mut field_matches = Vec::new();
                            let mut field_reconstructors = Vec::new();
                            let mut field_names = Vec::new();

                            for (idx, f) in fields.iter().enumerate() {
                                let f_variant = &f.variant_ident;
                                let f_name = format_ident!("field_{}", idx);
                                let f_ty = &f.ty;
                                let default_field_size = f.size_override.unwrap_or(1024);
                                let display_name = f
                                    .field_ident
                                    .as_ref()
                                    .map(|id| id.to_string())
                                    .unwrap_or_else(|| format!("field_{}", idx));

                                field_collectors.push(quote! {
                                    let mut #f_name = Vec::new();
                                });

                                field_matches.push(quote! {
                                    #chunk_ident::#f_variant(field_chunk) => {
                                        #f_name.push(field_chunk);
                                    }
                                });

                                field_reconstructors.push(quote! {
                                    let #f_name = {
                                        if #f_name.is_empty() {
                                            return Err(rewrite::results::NetabaseError::BlobReconstruction(
                                                rewrite::results::BlobReconstructionError::MissingChunks
                                            ));
                                        }
                                        let mut sorted = #f_name;
                                        sorted.sort_by_key(|c| c.index);

                                        let chunk_size = match size {
                                            rewrite::traits::structural::blob::ChunkSize::Default => #default_field_size,
                                            rewrite::traits::structural::blob::ChunkSize::Size(n) => n,
                                        };

                                        let mut next_expected = 0;
                                        let max_idx = sorted.last().map(|c| c.index).unwrap_or(0);

                                        for chunk in &sorted {
                                            if chunk.index > next_expected {
                                                return Err(rewrite::results::NetabaseError::BlobReconstruction(
                                                    rewrite::results::BlobReconstructionError::InvalidChunkData(format!("Gap at index {}", next_expected))
                                                ));
                                            }
                                            if chunk.data.len() > chunk_size {
                                                return Err(rewrite::results::NetabaseError::BlobReconstruction(
                                                    rewrite::results::BlobReconstructionError::InvalidChunkData(format!("Chunk overflow at index {}", chunk.index))
                                                ));
                                            }
                                            if chunk.data.len() < chunk_size && chunk.index < max_idx {
                                                return Err(rewrite::results::NetabaseError::BlobReconstruction(
                                                    rewrite::results::BlobReconstructionError::InvalidChunkData(format!("Partial chunk in middle at index {}", chunk.index))
                                                ));
                                            }
                                            next_expected += 1;
                                        }

                                        let data: Vec<u8> = sorted.into_iter().flat_map(|c| c.data).collect();
                                        rkyv::from_bytes::<#f_ty, rkyv::rancor::Error>(&data).map_err(|e|
                                            rewrite::results::NetabaseError::Serialization(format!("rkyv deserialization failed for field {}: {:?}", #display_name, e))
                                        )?
                                    };
                                });

                                field_names.push(if let Some(id) = &f.field_ident {
                                    quote! { #id: #f_name }
                                } else {
                                    quote! { #f_name }
                                });
                            }

                            let constructor = if fields.len() > 0 && fields[0].field_ident.is_some()
                            {
                                quote! { Self::#variant_ident { #(#field_names),* } }
                            } else {
                                let f_names: Vec<_> = (0..fields.len())
                                    .map(|i| format_ident!("field_{}", i))
                                    .collect();
                                quote! { Self::#variant_ident(#(#f_names),*) }
                            };

                            quote! {
                                #(#field_collectors)*
                                for c in chunks {
                                    if let Self::Chunk::#variant_ident(inner) = c {
                                        match inner {
                                            #(#field_matches)*
                                            _ => {}
                                        }
                                    }
                                }
                                #(#field_reconstructors)*
                                return Ok(#constructor);
                            }
                        }
                    };
                    variant_reconstructions.push(logic);
                }

                vec![
                    syn::parse_quote! {
                        impl #impl_generics rewrite::traits::structural::blob::NetabaseBlobItem for #ident #ty_generics #extended_where_clause {
                            type Chunk = #chunk_ident;
                            type BlobIter = std::vec::IntoIter<rewrite::results::NetabaseResult<Self::Chunk>>;
                            const DEFAULT_CHUNK_SIZE: usize = #default_chunk_size;

                            fn into_chunks(self, size: rewrite::traits::structural::blob::ChunkSize) -> Box<dyn Iterator<Item = Self::Chunk>> {
                                Box::new(self.into_chunks_iter(size).filter_map(|r| r.ok()))
                            }

                            fn into_chunks_iter(self, size: rewrite::traits::structural::blob::ChunkSize) -> Self::BlobIter {
                                let mut all_chunks = Vec::new();
                                #into_chunks_body
                                all_chunks.into_iter()
                            }

                            fn try_from_chunks(
                                chunks: impl Iterator<Item = Self::Chunk>,
                                size: rewrite::traits::structural::blob::ChunkSize,
                            ) -> rewrite::results::NetabaseResult<Self> {
                                let mut all_variant_chunks: std::collections::HashMap<String, Vec<Self::Chunk>> = std::collections::HashMap::new();
                                for chunk in chunks {
                                    let key = match &chunk {
                                        #( Self::Chunk::#variant_idents(_) => stringify!(#variant_idents).to_string(), )*
                                        _ => "Unknown".to_string(),
                                    };
                                    all_variant_chunks.entry(key).or_default().push(chunk);
                                }

                                for (variant_name, chunks) in all_variant_chunks {
                                    let res: rewrite::results::NetabaseResult<Self> = match variant_name.as_str() {
                                        #(
                                            stringify!(#variant_idents) => {
                                                (|| {
                                                    let chunks = chunks;
                                                    #variant_reconstructions
                                                })()
                                            }
                                        )*
                                        _ => Err(rewrite::results::NetabaseError::BlobReconstruction(
                                            rewrite::results::BlobReconstructionError::InvalidChunkData(format!("Unknown variant {}", variant_name))
                                        )),
                                    };
                                    if res.is_ok() { return res; }
                                }

                                Err(rewrite::results::NetabaseError::BlobReconstruction(
                                    rewrite::results::BlobReconstructionError::InvalidChunkData("Could not reconstruct any variant".to_string())
                                ))
                            }

                            fn get_blob(&self) -> &Self::Chunk { unimplemented!() }
                        }
                    },
                    syn::parse_quote! {
                        impl #impl_generics IntoIterator for #ident #ty_generics #extended_where_clause {
                            type Item = rewrite::results::NetabaseResult<#chunk_ident>;
                            type IntoIter = std::vec::IntoIter<Self::Item>;

                            fn into_iter(self) -> Self::IntoIter {
                                rewrite::traits::structural::blob::NetabaseBlobItem::into_chunks_iter(self, rewrite::traits::structural::blob::ChunkSize::Default)
                            }
                        }
                    },
                ]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, DeriveInput};

    fn get_root_chunk_ident(kind: &BlobPlanKind) -> &Ident {
        match kind {
            BlobPlanKind::Full(plan) => &plan.chunk_plan.ident,
            BlobPlanKind::PartialStruct(plan) => &plan.root_chunk_ident,
            BlobPlanKind::PartialEnum(plan) => &plan.root_chunk_ident,
        }
    }

    #[test]
    fn test_blob_planner_full_struct() {
        let input: DeriveInput = parse_quote! {
            #[derive(NetabaseBlob)]
            #[chunk_size(2048)]
            struct MyBlob {
                field1: String,
                field2: u64,
            }
        };

        let visited = proc_macro_flow::Visited::<BlobVisitor>::from(&input);
        let plan = BlobPlan::try_from(&visited).unwrap();
        assert_eq!(plan.ident, "MyBlob");
        assert_eq!(get_root_chunk_ident(&plan.kind), "MyBlobChunk");

        match &plan.kind {
            BlobPlanKind::Full(full_plan) => {
                assert_eq!(full_plan.default_size, Some(2048));
            }
            _ => panic!("Expected Full kind"),
        }
    }

    #[test]
    fn test_blob_planner_partial_struct() {
        let input: DeriveInput = parse_quote! {
            #[derive(NetabaseBlob)]
            struct MyBlob {
                #[chunk_size(1024)]
                field1: String,
                #[blob_field(chunk_size(512))]
                field2: OtherBlob,
            }
        };

        let visited = proc_macro_flow::Visited::<BlobVisitor>::from(&input);
        let plan = BlobPlan::try_from(&visited).unwrap();

        match &plan.kind {
            BlobPlanKind::PartialStruct(partial_plan) => {
                let fields = &partial_plan.fields;
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].variant_ident, "Field1");
                assert_eq!(fields[0].size_override, Some(1024));
                assert_eq!(fields[1].variant_ident, "Field2");
                assert_eq!(fields[1].size_override, Some(512));
            }
            _ => panic!("Expected PartialStruct kind"),
        }
    }

    #[test]
    fn test_blob_planner_field_level_config() {
        let input: DeriveInput = parse_quote! {
            #[derive(NetabaseBlob)]
            struct MyBlob {
                #[blob(chunk_size(512))]
                field1: String,
            }
        };

        let visited = proc_macro_flow::Visited::<BlobVisitor>::from(&input);
        let plan = BlobPlan::try_from(&visited).unwrap();
        match &plan.kind {
            BlobPlanKind::PartialStruct(p) => {
                assert_eq!(p.fields[0].size_override, Some(512));
            }
            _ => panic!("Expected PartialStruct"),
        }
    }

    #[test]
    fn test_blob_generator_full_struct() {
        let input: DeriveInput = parse_quote! {
            #[derive(NetabaseBlob)]
            struct MyBlob {
                field1: String,
            }
        };

        let visited = proc_macro_flow::Visited::<BlobVisitor>::from(&input);
        let plan = BlobPlan::try_from(&visited).unwrap();
        let generator = BlobGenerator::from(plan);
        let items = proc_macro_flow::MultiGeneratable::generate(generator).unwrap();
        let mut output_ts = proc_macro2::TokenStream::new();
        for item in items {
            output_ts.extend(quote::quote!(#item));
        }
        let output = output_ts.to_string();
        assert!(output.contains("pub struct MyBlobChunk"));
        assert!(output.contains("pub index : usize"));
        assert!(output.contains("pub data : Vec < u8 >"));
    }

    #[test]
    fn test_blob_config_custom_derives() {
        let input: DeriveInput = parse_quote! {
            #[derive(NetabaseBlob)]
            #[chunk_derives(Serialize, Deserialize)]
            struct MyBlob {
                field1: String,
            }
        };

        let visited = proc_macro_flow::Visited::<BlobVisitor>::from(&input);
        let plan = BlobPlan::try_from(&visited).unwrap();
        assert_eq!(plan.config.custom_derives.len(), 2);

        let generator = BlobGenerator::from(plan);
        let items = proc_macro_flow::MultiGeneratable::generate(generator).unwrap();
        let mut output_ts = proc_macro2::TokenStream::new();
        for item in items {
            output_ts.extend(quote::quote!(#item));
        }
        let output = output_ts.to_string();
        assert!(output.contains("Serialize"));
        assert!(output.contains("Deserialize"));
    }

    #[test]
    fn test_blob_config_owner_id_and_checksum() {
        let input: DeriveInput = parse_quote! {
            #[derive(NetabaseBlob)]
            #[chunk_owner_id]
            #[chunk_checksum]
            struct MyBlob {
                field1: String,
            }
        };

        let visited = proc_macro_flow::Visited::<BlobVisitor>::from(&input);
        let plan = BlobPlan::try_from(&visited).unwrap();
        assert!(plan.config.include_owner_id);
        assert!(plan.config.include_checksum);

        let generator = BlobGenerator::from(plan);
        let items = proc_macro_flow::MultiGeneratable::generate(generator).unwrap();
        let mut output_ts = proc_macro2::TokenStream::new();
        for item in items {
            output_ts.extend(quote::quote!(#item));
        }
        let output = output_ts.to_string();
        assert!(output.contains("pub owner_id : u64"));
        assert!(output.contains("pub checksum : u64"));
        assert!(output.contains("pub index : usize"));
        assert!(output.contains("pub data : Vec < u8 >"));
    }

    #[test]
    fn test_blob_config_serialization_functions() {
        let input: DeriveInput = parse_quote! {
            #[derive(NetabaseBlob)]
            #[chunk_serialize(my_crate::serialize_chunk)]
            #[chunk_deserialize(my_crate::deserialize_chunk)]
            struct MyBlob {
                field1: String,
            }
        };

        let visited = proc_macro_flow::Visited::<BlobVisitor>::from(&input);
        let plan = BlobPlan::try_from(&visited).unwrap();
        assert!(plan.config.serialize_fn.is_some());
        assert!(plan.config.deserialize_fn.is_some());
    }

    #[test]
    fn test_blob_config_single_attribute() {
        let input: DeriveInput = parse_quote! {
            #[derive(NetabaseBlob)]
            #[blob(chunk_size(1024), chunk_derives(Serialize, Deserialize), chunk_owner_id, chunk_checksum, strategy(partial))]
            struct MyBlob {
                field1: String,
            }
        };

        let visited = proc_macro_flow::Visited::<BlobVisitor>::from(&input);
        let plan = BlobPlan::try_from(&visited).unwrap();
        assert_eq!(plan.config.strategy, BlobStrategy::Partial);
        assert!(plan.config.include_owner_id);
        assert!(plan.config.include_checksum);
        assert_eq!(plan.config.custom_derives.len(), 2);

        match &plan.kind {
            BlobPlanKind::PartialStruct(p) => {
                assert_eq!(p.fields[0].size_override, Some(1024));
            }
            _ => panic!("Expected PartialStruct due to strategy(partial)"),
        }
    }
}
