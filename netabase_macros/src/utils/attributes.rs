use syn::{Attribute, Error, Meta, Path, Result};

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
            "link attribute must be in the form #[link(Definition, Model)]",
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
            "subscribe attribute must be in the form #[subscribe(Topic1, Topic2, ...)]",
        ))
    }
}

/// Parsed definition attribute containing all configuration
#[derive(Debug, Clone)]
pub struct DefinitionAttributeConfig {
    /// The definition name path
    pub definition: Path,
    /// Subscription topics for this definition
    pub subscriptions: Vec<Path>,
    /// Optional file path to import schema from
    pub from_file: Option<String>,
    /// Repository identifiers this definition belongs to
    pub repositories: Vec<syn::Ident>,
}

impl DefinitionAttributeConfig {
    /// Check if this definition belongs to a specific repository
    pub fn belongs_to_repository(&self, repo_name: &syn::Ident) -> bool {
        self.repositories.iter().any(|r| r == repo_name)
    }

    /// Check if this definition has any repository memberships
    pub fn has_repositories(&self) -> bool {
        !self.repositories.is_empty()
    }
}

/// Parse subscriptions from netabase_definition attribute
/// #[netabase_definition(DefinitionName, subscriptions(Topic1, Topic2), repos(Repo1, Repo2), from_file = "path/to/schema.toml")]
pub fn parse_definition_attribute_from_tokens(
    tokens: proc_macro2::TokenStream,
) -> Result<DefinitionAttributeConfig> {
    use syn::Token;
    use syn::parse::Parse;

    struct DefinitionAttr {
        definition: Path,
        subscriptions: Vec<Path>,
        from_file: Option<String>,
        repositories: Vec<syn::Ident>,
    }

    impl Parse for DefinitionAttr {
        fn parse(input: syn::parse::ParseStream) -> Result<Self> {
            let definition: Path = input.parse()?;

            let mut subscriptions = Vec::new();
            let mut from_file = None;
            let mut repositories = Vec::new();

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
                } else if ident == "repos" {
                    let content;
                    syn::parenthesized!(content in input);
                    let repo_list: syn::punctuated::Punctuated<syn::Ident, Token![,]> =
                        content.parse_terminated(Parse::parse, Token![,])?;
                    repositories = repo_list.into_iter().collect();
                } else {
                    return Err(Error::new(
                        ident.span(),
                        "expected 'subscriptions', 'from_file', or 'repos'",
                    ));
                }
            }

            Ok(DefinitionAttr {
                definition,
                subscriptions,
                from_file,
                repositories,
            })
        }
    }

    let attr: DefinitionAttr = syn::parse2(tokens)?;
    Ok(DefinitionAttributeConfig {
        definition: attr.definition,
        subscriptions: attr.subscriptions,
        from_file: attr.from_file,
        repositories: attr.repositories,
    })
}

pub fn parse_definition_attribute(attr: &Attribute) -> Result<DefinitionAttributeConfig> {
    let meta = &attr.meta;

    if let Meta::List(meta_list) = meta {
        parse_definition_attribute_from_tokens(meta_list.tokens.clone())
    } else {
        Err(Error::new_spanned(
            attr,
            "netabase_definition must be in the form #[netabase_definition(DefinitionName, subscriptions(...), repos(...), from_file = \"...\")]",
        ))
    }
}

/// Parsed repository attribute containing configuration
#[derive(Debug, Clone)]
pub struct RepositoryAttributeConfig {
    /// The repository name identifier
    pub name: syn::Ident,
}

/// Parse netabase_repository attribute
/// #[netabase_repository(RepoName)]
pub fn parse_repository_attribute_from_tokens(
    tokens: proc_macro2::TokenStream,
) -> Result<RepositoryAttributeConfig> {
    use syn::parse::Parse;

    struct RepositoryAttr {
        name: syn::Ident,
    }

    impl Parse for RepositoryAttr {
        fn parse(input: syn::parse::ParseStream) -> Result<Self> {
            let name: syn::Ident = input.parse()?;
            Ok(RepositoryAttr { name })
        }
    }

    let attr: RepositoryAttr = syn::parse2(tokens)?;
    Ok(RepositoryAttributeConfig { name: attr.name })
}

pub fn parse_repository_attribute(attr: &Attribute) -> Result<RepositoryAttributeConfig> {
    let meta = &attr.meta;

    if let Meta::List(meta_list) = meta {
        parse_repository_attribute_from_tokens(meta_list.tokens.clone())
    } else {
        Err(Error::new_spanned(
            attr,
            "netabase_repository must be in the form #[netabase_repository(RepoName)]",
        ))
    }
}

/// Parse netabase global attribute
/// #[netabase(GlobalName)]
pub fn parse_global_attribute(attr: &Attribute) -> Result<Path> {
    let meta = &attr.meta;

    if let Meta::List(meta_list) = meta {
        let tokens = &meta_list.tokens;
        syn::parse2(tokens.clone()).map_err(|e| {
            Error::new(
                e.span(),
                format!("Failed to parse netabase attribute: {}", e),
            )
        })
    } else {
        Err(Error::new_spanned(
            attr,
            "netabase must be in the form #[netabase(GlobalName)]",
        ))
    }
}

/// Remove an attribute from a list
pub fn remove_attribute(attrs: &mut Vec<Attribute>, name: &str) {
    attrs.retain(|attr| !is_attribute(attr, name));
}
