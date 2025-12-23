use syn::{Attribute, Meta, Path, Result, Error};
use proc_macro2::Span;

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
    let meta = &attr.meta;

    if let Meta::List(meta_list) = meta {
        let tokens = &meta_list.tokens;

        let parser = |input: syn::parse::ParseStream| {
            let paths: syn::punctuated::Punctuated<Path, syn::Token![,]> =
                input.parse_terminated(Path::parse, syn::Token![,])?;
            Ok(paths)
        };

        let parsed = syn::parse::Parser::parse2(parser, tokens.clone())?;
        let paths: Vec<&Path> = parsed.iter().collect();

        if paths.len() != 2 {
            return Err(Error::new_spanned(
                attr,
                "link attribute must have exactly 2 arguments: #[link(Definition, Model)]"
            ));
        }

        Ok((paths[0].clone(), paths[1].clone()))
    } else {
        Err(Error::new_spanned(
            attr,
            "link attribute must be in the form #[link(Definition, Model)]"
        ))
    }
}

/// Parse #[subscribe(Topic1, Topic2, ...)] attribute and extract topic paths
pub fn parse_subscribe_attribute(attr: &Attribute) -> Result<Vec<Path>> {
    let meta = &attr.meta;

    if let Meta::List(meta_list) = meta {
        let tokens = &meta_list.tokens;

        let parser = |input: syn::parse::ParseStream| {
            let paths: syn::punctuated::Punctuated<Path, syn::Token![,]> =
                input.parse_terminated(Path::parse, syn::Token![,])?;
            Ok(paths)
        };

        let parsed = syn::parse::Parser::parse2(parser, tokens.clone())?;
        Ok(parsed.into_iter().collect())
    } else {
        Err(Error::new_spanned(
            attr,
            "subscribe attribute must be in the form #[subscribe(Topic1, Topic2, ...)]"
        ))
    }
}

/// Parse subscriptions from netabase_definition attribute
/// #[netabase_definition(DefinitionName, subscriptions(Topic1, Topic2))]
pub fn parse_definition_attribute_from_tokens(tokens: proc_macro2::TokenStream) -> Result<(Path, Vec<Path>)> {
    use syn::parse::Parse;

    struct DefinitionAttr {
        definition: Path,
        subscriptions: Vec<Path>,
    }

    impl Parse for DefinitionAttr {
        fn parse(input: syn::parse::ParseStream) -> Result<Self> {
            let definition: Path = input.parse()?;
            input.parse::<syn::Token![,]>()?;

            // Parse subscriptions(...)
            let subscriptions_ident: syn::Ident = input.parse()?;
            if subscriptions_ident != "subscriptions" {
                return Err(Error::new(
                    subscriptions_ident.span(),
                    "expected 'subscriptions'"
                ));
            }

            let content;
            syn::parenthesized!(content in input);
            let topics: syn::punctuated::Punctuated<Path, syn::Token![,]> =
                content.parse_terminated(Path::parse, syn::Token![,])?;

            Ok(DefinitionAttr {
                definition,
                subscriptions: topics.into_iter().collect(),
            })
        }
    }

    let attr: DefinitionAttr = syn::parse2(tokens)?;
    Ok((attr.definition, attr.subscriptions))
}

pub fn parse_definition_attribute(attr: &Attribute) -> Result<(Path, Vec<Path>)> {
    let meta = &attr.meta;

    if let Meta::List(meta_list) = meta {
        let tokens = &meta_list.tokens;

        // Parse as: DefinitionName, subscriptions(Topic1, Topic2, ...)
        let parser = |input: syn::parse::ParseStream| {
            let definition_name: Path = input.parse()?;
            input.parse::<syn::Token![,]>()?;

            // Parse subscriptions(...)
            let subscriptions_ident: syn::Ident = input.parse()?;
            if subscriptions_ident != "subscriptions" {
                return Err(Error::new(
                    subscriptions_ident.span(),
                    "expected 'subscriptions'"
                ));
            }

            let content;
            syn::parenthesized!(content in input);
            let topics: syn::punctuated::Punctuated<Path, syn::Token![,]> =
                content.parse_terminated(Path::parse, syn::Token![,])?;

            Ok((definition_name, topics.into_iter().collect()))
        };

        syn::parse2(tokens.clone())
            .map_err(|e| Error::new(e.span(), format!("Failed to parse netabase_definition attribute: {}", e)))
    } else {
        Err(Error::new_spanned(
            attr,
            "netabase_definition must be in the form #[netabase_definition(DefinitionName, subscriptions(Topic1, Topic2))]"
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
