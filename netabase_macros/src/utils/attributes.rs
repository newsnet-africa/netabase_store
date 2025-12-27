use syn::{Attribute, Meta, Path, Result, Error};

/// Utilities for parsing field and item attributes

/// Check if an attribute matches a given path (e.g., "primary_key")
pub fn is_attribute(attr: &Attribute, name: &str) -> bool {
    attr.path().is_ident(name)
}

/// Find attribute by name
pub fn find_attribute<'a>(attrs: &'a [Attribute], name: &str) -> Option<&'a Attribute> {
    attrs.iter().find(|attr| is_attribute(attr, name))
}

/// Check if attributes contain a specific attribute
pub fn has_attribute(attrs: &[Attribute], name: &str) -> bool {
    find_attribute(attrs, name).is_some()
}

/// Parse #[link(Definition, Model)] attribute and extract (definition, model) paths
pub fn parse_link_attribute(attr: &Attribute) -> Result<(Path, Path)> {
    use syn::parse::Parse;

    struct LinkArgs {
        definition: Path,
        _comma: syn::Token![,],
        model: Path,
    }

    impl Parse for LinkArgs {
        fn parse(input: syn::parse::ParseStream) -> Result<Self> {
            Ok(LinkArgs {
                definition: input.parse()?,
                _comma: input.parse()?,
                model: input.parse()?,
            })
        }
    }

    let meta = &attr.meta;

    if let Meta::List(meta_list) = meta {
        let args: LinkArgs = syn::parse2(meta_list.tokens.clone())?;
        Ok((args.definition, args.model))
    } else {
        Err(Error::new_spanned(
            attr,
            "link attribute must be in the form #[link(Definition, Model)]"
        ))
    }
}

/// Parse #[subscribe(Topic1, Topic2, ...)] attribute and extract topic paths
pub fn parse_subscribe_attribute(attr: &Attribute) -> Result<Vec<Path>> {
    use syn::parse::Parse;

    struct SubscribeArgs {
        topics: syn::punctuated::Punctuated<Path, syn::Token![,]>,
    }

    impl Parse for SubscribeArgs {
        fn parse(input: syn::parse::ParseStream) -> Result<Self> {
            Ok(SubscribeArgs {
                topics: input.parse_terminated(Parse::parse, syn::Token![,])?,
            })
        }
    }

    let meta = &attr.meta;

    if let Meta::List(meta_list) = meta {
        let args: SubscribeArgs = syn::parse2(meta_list.tokens.clone())?;
        Ok(args.topics.into_iter().collect())
    } else {
        Err(Error::new_spanned(
            attr,
            "subscribe attribute must be in the form #[subscribe(Topic1, Topic2, ...)]"
        ))
    }
}

/// Parse subscriptions from netabase_definition attribute
/// #[netabase_definition(DefinitionName, subscriptions(Topic1, Topic2), from_file = "path/to/schema.toml")]
pub fn parse_definition_attribute_from_tokens(tokens: proc_macro2::TokenStream) -> Result<(Path, Vec<Path>, Option<String>)> {
    use syn::parse::Parse;
    use syn::Token;

    struct DefinitionAttr {
        definition: Path,
        subscriptions: Vec<Path>,
        from_file: Option<String>,
    }

    impl Parse for DefinitionAttr {
        fn parse(input: syn::parse::ParseStream) -> Result<Self> {
            let definition: Path = input.parse()?;
            
            let mut subscriptions = Vec::new();
            let mut from_file = None;

            while !input.is_empty() {
                let _comma: Token![,] = input.parse()?;
                if input.is_empty() {
                    break;
                }

                let ident: syn::Ident = input.parse()?;
                if ident == "subscriptions" {
                    let content;
                    syn::parenthesized!(content in input);
                    let topics: syn::punctuated::Punctuated<Path, Token![,]> =
                        content.parse_terminated(Parse::parse, Token![,])?;
                    subscriptions = topics.into_iter().collect();
                } else if ident == "from_file" {
                    let _eq: Token![=] = input.parse()?;
                    let lit: syn::LitStr = input.parse()?;
                    from_file = Some(lit.value());
                } else {
                    return Err(Error::new(ident.span(), "expected 'subscriptions' or 'from_file'"));
                }
            }

            Ok(DefinitionAttr {
                definition,
                subscriptions,
                from_file,
            })
        }
    }

    let attr: DefinitionAttr = syn::parse2(tokens)?;
    Ok((attr.definition, attr.subscriptions, attr.from_file))
}

pub fn parse_definition_attribute(attr: &Attribute) -> Result<(Path, Vec<Path>, Option<String>)> {
    let meta = &attr.meta;

    if let Meta::List(meta_list) = meta {
        parse_definition_attribute_from_tokens(meta_list.tokens.clone())
    } else {
        Err(Error::new_spanned(
            attr,
            "netabase_definition must be in the form #[netabase_definition(DefinitionName, subscriptions(...), from_file = \"...\")]"
        ))
    }
}

/// Parse netabase global attribute
/// #[netabase(GlobalName)]
pub fn parse_global_attribute(attr: &Attribute) -> Result<Path> {
    let meta = &attr.meta;

    if let Meta::List(meta_list) = meta {
        let tokens = &meta_list.tokens;
        syn::parse2(tokens.clone())
            .map_err(|e| Error::new(e.span(), format!("Failed to parse netabase attribute: {}", e)))
    } else {
        Err(Error::new_spanned(
            attr,
            "netabase must be in the form #[netabase(GlobalName)]"
        ))
    }
}

/// Remove an attribute from a list
pub fn remove_attribute(attrs: &mut Vec<Attribute>, name: &str) {
    attrs.retain(|attr| !is_attribute(attr, name));
}
