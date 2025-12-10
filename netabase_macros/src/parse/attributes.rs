//! Attribute parsing using darling
//!
//! This module uses the darling crate to parse attributes on models and fields,
//! providing a clean and type-safe interface for extracting attribute data.

use syn::{Attribute, Ident, Path};

/// Parsed attributes from a model struct
#[derive(Debug)]
pub struct ModelAttributes {
    /// Model identifier
    pub ident: Ident,

    /// Visibility
    pub vis: syn::Visibility,

    /// Subscription topics this model subscribes to
    pub subscribe: Vec<Ident>,
}

impl ModelAttributes {
    /// Parse model attributes from a DeriveInput
    pub fn from_derive_input(input: &syn::DeriveInput) -> syn::Result<Self> {
        let mut subscribe = Vec::new();

        // Parse attributes
        for attr in &input.attrs {
            if attr.path().is_ident("subscribe") {
                // Parse the subscription list: #[subscribe(Topic1, Topic2)]
                attr.parse_nested_meta(|meta| {
                    // Each item in the list is an identifier
                    if let Some(ident) = meta.path.get_ident() {
                        subscribe.push(ident.clone());
                        Ok(())
                    } else {
                        Err(meta.error("Expected identifier"))
                    }
                })?;
            }
        }

        Ok(Self {
            ident: input.ident.clone(),
            vis: input.vis.clone(),
            subscribe,
        })
    }
}

/// Parsed attributes from a field
#[derive(Debug)]
pub struct FieldAttributes {
    /// Field identifier
    pub ident: Option<Ident>,

    /// Field type
    pub ty: syn::Type,

    /// Field visibility
    pub vis: syn::Visibility,

    /// Whether this is a primary key
    pub primary_key: bool,

    /// Whether this is a secondary key
    pub secondary_key: bool,

    /// Whether this is a relation
    pub relation: bool,

    /// Path to cross-definition model
    pub cross_definition_link: Option<Path>,
}

impl FieldAttributes {
    /// Count how many key/relation attributes are present
    pub fn attribute_count(&self) -> usize {
        [self.primary_key, self.secondary_key, self.relation]
            .iter()
            .filter(|&&x| x)
            .count()
    }

    /// Check if this field has any special attributes
    pub fn has_special_attribute(&self) -> bool {
        self.primary_key || self.secondary_key || self.relation
    }

    /// Parse field attributes from a syn::Field
    pub fn from_field(field: &syn::Field) -> syn::Result<Self> {
        let mut primary_key = false;
        let mut secondary_key = false;
        let mut relation = false;
        let mut cross_definition_link = None;

        // Parse each attribute
        for attr in &field.attrs {
            if attr.path().is_ident("primary_key") {
                primary_key = true;
            } else if attr.path().is_ident("secondary_key") {
                secondary_key = true;
            } else if attr.path().is_ident("relation") {
                relation = true;
            } else if attr.path().is_ident("cross_definition_link") {
                // Parse the path argument: #[cross_definition_link(path::to::Model)]
                cross_definition_link = Some(attr.parse_args::<Path>()?);
            }
        }

        Ok(Self {
            ident: field.ident.clone(),
            ty: field.ty.clone(),
            vis: field.vis.clone(),
            primary_key,
            secondary_key,
            relation,
            cross_definition_link,
        })
    }
}

/// Module-level attributes for netabase_definition_module
#[derive(Debug)]
pub struct ModuleAttributes {
    /// Name of the definition enum
    pub definition_name: Ident,

    /// Name of the keys enum
    pub keys_name: Ident,

    /// Available subscription topics
    pub subscriptions: Vec<Ident>,
}

impl ModuleAttributes {
    /// Parse module attributes from attribute tokens
    ///
    /// Expected format:
    /// `#[netabase_definition_module(DefinitionName, DefinitionKeys, subscriptions(Topic1, Topic2))]`
    pub fn parse(attr: &Attribute) -> syn::Result<Self> {
        let meta = attr.meta.clone();

        // Parse the attribute arguments
        if let syn::Meta::List(list) = meta {
            let mut tokens = list.tokens.clone().into_iter();

            // Parse definition name
            let definition_name = Self::parse_ident(&mut tokens, "definition name")?;
            Self::expect_comma(&mut tokens)?;

            // Parse keys name
            let keys_name = Self::parse_ident(&mut tokens, "keys name")?;

            // Parse optional subscriptions
            let subscriptions = if Self::peek_comma(&mut tokens) {
                Self::expect_comma(&mut tokens)?;
                Self::parse_subscriptions(&mut tokens)?
            } else {
                Vec::new()
            };

            Ok(Self {
                definition_name,
                keys_name,
                subscriptions,
            })
        } else {
            Err(syn::Error::new_spanned(
                attr,
                "Expected attribute format: #[netabase_definition_module(DefinitionName, DefinitionKeys)]"
            ))
        }
    }

    fn parse_ident(
        tokens: &mut proc_macro2::token_stream::IntoIter,
        context: &str,
    ) -> syn::Result<Ident> {
        match tokens.next() {
            Some(proc_macro2::TokenTree::Ident(ident)) => Ok(ident),
            other => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Expected {} identifier, got: {:?}", context, other)
            ))
        }
    }

    fn expect_comma(tokens: &mut proc_macro2::token_stream::IntoIter) -> syn::Result<()> {
        match tokens.next() {
            Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == ',' => Ok(()),
            other => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Expected comma, got: {:?}", other)
            ))
        }
    }

    fn peek_comma(tokens: &mut proc_macro2::token_stream::IntoIter) -> bool {
        tokens.clone().next().map(|t| {
            matches!(t, proc_macro2::TokenTree::Punct(p) if p.as_char() == ',')
        }).unwrap_or(false)
    }

    fn parse_subscriptions(
        tokens: &mut proc_macro2::token_stream::IntoIter,
    ) -> syn::Result<Vec<Ident>> {
        // Expect: subscriptions(Topic1, Topic2, ...)
        let ident = Self::parse_ident(tokens, "subscriptions keyword")?;
        if ident != "subscriptions" {
            return Err(syn::Error::new(
                ident.span(),
                format!("Expected 'subscriptions', got '{}'", ident)
            ));
        }

        // Parse the group (Topic1, Topic2)
        match tokens.next() {
            Some(proc_macro2::TokenTree::Group(group)) => {
                let mut subs = Vec::new();
                let mut sub_tokens = group.stream().into_iter();

                loop {
                    match sub_tokens.next() {
                        Some(proc_macro2::TokenTree::Ident(ident)) => {
                            subs.push(ident);

                            // Check for comma or end
                            match sub_tokens.next() {
                                Some(proc_macro2::TokenTree::Punct(p)) if p.as_char() == ',' => {
                                    continue; // More items
                                }
                                None => break, // End of list
                                other => {
                                    return Err(syn::Error::new(
                                        proc_macro2::Span::call_site(),
                                        format!("Expected comma or end of subscriptions, got: {:?}", other)
                                    ));
                                }
                            }
                        }
                        None => break,
                        other => {
                            return Err(syn::Error::new(
                                proc_macro2::Span::call_site(),
                                format!("Expected subscription topic identifier, got: {:?}", other)
                            ));
                        }
                    }
                }

                Ok(subs)
            }
            other => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Expected subscription list in parentheses, got: {:?}", other)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_field_attributes_parsing() {
        let field: syn::Field = parse_quote! {
            #[primary_key]
            pub id: u64
        };

        let attrs = FieldAttributes::from_field(&field).unwrap();
        assert!(attrs.primary_key);
        assert!(!attrs.secondary_key);
        assert!(!attrs.relation);
        assert_eq!(attrs.attribute_count(), 1);
    }

    #[test]
    fn test_field_attributes_multiple() {
        let field: syn::Field = parse_quote! {
            #[secondary_key]
            pub email: String
        };

        let attrs = FieldAttributes::from_field(&field).unwrap();
        assert!(!attrs.primary_key);
        assert!(attrs.secondary_key);
        assert!(!attrs.relation);
    }

    #[test]
    fn test_model_attributes_parsing() {
        let input: syn::DeriveInput = parse_quote! {
            #[subscribe(Updates, Premium)]
            pub struct User {
                id: u64,
            }
        };

        let attrs = ModelAttributes::from_derive_input(&input).unwrap();
        assert_eq!(attrs.ident.to_string(), "User");
        assert_eq!(attrs.subscribe.len(), 2);
        assert_eq!(attrs.subscribe[0].to_string(), "Updates");
        assert_eq!(attrs.subscribe[1].to_string(), "Premium");
    }

    #[test]
    fn test_field_no_attributes() {
        let field: syn::Field = parse_quote! {
            pub name: String
        };

        let attrs = FieldAttributes::from_field(&field).unwrap();
        assert!(!attrs.has_special_attribute());
        assert_eq!(attrs.attribute_count(), 0);
    }
}
