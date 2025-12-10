//! Module parsing for netabase definition modules
//!
//! This module provides visitors for parsing entire netabase definition modules,
//! including all models and nested modules.

use syn::{visit::Visit, ItemMod, ItemStruct, Attribute};

use super::attributes::ModuleAttributes;
use super::metadata::{ModuleMetadata, ErrorCollector, MetadataValidator};
use super::model::ModelVisitor;

/// Visitor for parsing a netabase definition module
pub struct ModuleVisitor {
    /// Collected module metadata
    pub metadata: Option<ModuleMetadata>,

    /// Accumulated errors during parsing
    pub errors: ErrorCollector,
}

impl ModuleVisitor {
    /// Create a new module visitor
    pub fn new() -> Self {
        Self {
            metadata: None,
            errors: ErrorCollector::new(),
        }
    }

    /// Parse a module from an ItemMod with its attribute
    ///
    /// This is the main entry point for parsing a #[netabase_definition_module(...)] module.
    pub fn parse_module(attr: &Attribute, module: &ItemMod) -> Result<ModuleMetadata, syn::Error> {
        let mut visitor = Self::new();

        // Parse module attributes
        let module_attrs = match ModuleAttributes::parse(attr) {
            Ok(attrs) => attrs,
            Err(e) => return Err(e),
        };

        // Create module metadata
        let mut module_meta = ModuleMetadata::new(
            module.ident.clone(),
            module_attrs.definition_name,
            module_attrs.keys_name
        );

        // Add available subscriptions
        for sub in module_attrs.subscriptions {
            module_meta.add_subscription(sub);
        }

        // Visit the module contents
        if let Some((_, items)) = &module.content {
            for item in items {
                visitor.visit_item(item);
            }
        } else {
            return Err(syn::Error::new_spanned(
                module,
                "Module must have inline content (not a path)"
            ));
        }

        // Merge collected models into module metadata
        if let Some(collected) = visitor.metadata {
            // Transfer models to our module_meta
            for model in collected.models {
                module_meta.add_model(model);
            }

            // Transfer nested modules
            for nested in collected.nested_modules {
                module_meta.add_nested_module(nested);
            }
        }

        // Validate the complete module
        MetadataValidator::validate_module(&module_meta)?;

        // Check for collected errors
        if visitor.errors.has_errors() {
            return Err(visitor.errors.into_result().unwrap_err());
        }

        Ok(module_meta)
    }

    /// Initialize metadata if needed
    fn ensure_metadata(&mut self) {
        if self.metadata.is_none() {
            // Create temporary metadata (will be replaced with real one)
            self.metadata = Some(ModuleMetadata::new(
                syn::Ident::new("TempMod", proc_macro2::Span::call_site()),
                syn::Ident::new("TempDef", proc_macro2::Span::call_site()),
                syn::Ident::new("TempDefKeys", proc_macro2::Span::call_site())
            ));
        }
    }

    /// Check if an attribute is derive(NetabaseModel)
    fn is_netabase_model_derive(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            if attr.path().is_ident("derive") {
                // Check token stream for "NetabaseModel"
                if let syn::Meta::List(list) = &attr.meta {
                    let tokens_str = list.tokens.to_string();
                    return tokens_str.contains("NetabaseModel");
                }
            }
            false
        })
    }

    /// Check if an attribute is netabase_definition_module
    fn is_netabase_module_attr(attr: &Attribute) -> bool {
        attr.path().is_ident("netabase_definition_module")
    }
}

impl<'ast> Visit<'ast> for ModuleVisitor {
    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        // Check if this struct has #[derive(NetabaseModel)]
        if Self::is_netabase_model_derive(&item.attrs) {
            self.ensure_metadata();

            // Convert to DeriveInput for parsing
            let attrs = &item.attrs;
            let vis = &item.vis;
            let ident = &item.ident;
            let generics = &item.generics;
            let fields = &item.fields;

            let derive_input: syn::DeriveInput = syn::parse_quote! {
                #(#attrs)*
                #vis struct #ident #generics #fields
            };

            match ModelVisitor::parse_model(&derive_input) {
                Ok(model) => {
                    if let Some(ref mut meta) = self.metadata {
                        meta.add_model(model);
                    }
                }
                Err(e) => {
                    self.errors.add(e);
                }
            }
        }

        // Continue visiting
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        // Check if this is a nested netabase definition module
        if let Some(attr) = module.attrs.iter().find(|a| Self::is_netabase_module_attr(a)) {
            self.ensure_metadata();

            // Recursively parse the nested module
            match Self::parse_module(attr, module) {
                Ok(nested_meta) => {
                    if let Some(ref mut meta) = self.metadata {
                        meta.add_nested_module(nested_meta);
                    }
                }
                Err(e) => {
                    self.errors.add(e);
                }
            }
        } else {
            // Regular module, continue visiting its contents
            syn::visit::visit_item_mod(self, module);
        }
    }
}

impl Default for ModuleVisitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parse_simple_module() {
        let attr: Attribute = parse_quote! {
            #[netabase_definition_module(TestDef, TestDefKeys)]
        };

        let module: ItemMod = parse_quote! {
            pub mod test_def {
                #[derive(NetabaseModel)]
                pub struct User {
                    #[primary_key]
                    pub id: u64,
                }
            }
        };

        let meta = ModuleVisitor::parse_module(&attr, &module).unwrap();
        assert_eq!(meta.module_name.to_string(), "test_def");
        assert_eq!(meta.definition_name.to_string(), "TestDef");
        assert_eq!(meta.keys_name.to_string(), "TestDefKeys");
        assert_eq!(meta.models.len(), 1);
        assert_eq!(meta.models[0].name.to_string(), "User");
    }

    #[test]
    fn test_parse_module_with_subscriptions() {
        let attr: Attribute = parse_quote! {
            #[netabase_definition_module(TestDef, TestDefKeys, subscriptions(Updates, Premium))]
        };

        let module: ItemMod = parse_quote! {
            pub mod test_def {
                #[derive(NetabaseModel)]
                #[subscribe(Updates)]
                pub struct User {
                    #[primary_key]
                    id: u64,
                }
            }
        };

        let meta = ModuleVisitor::parse_module(&attr, &module).unwrap();
        assert_eq!(meta.available_subscriptions.len(), 2);
        assert_eq!(meta.available_subscriptions[0].to_string(), "Updates");
        assert_eq!(meta.available_subscriptions[1].to_string(), "Premium");
    }

    #[test]
    fn test_parse_module_with_multiple_models() {
        let attr: Attribute = parse_quote! {
            #[netabase_definition_module(TestDef, TestDefKeys)]
        };

        let module: ItemMod = parse_quote! {
            pub mod test_def {
                #[derive(NetabaseModel)]
                pub struct User {
                    #[primary_key]
                    id: u64,
                }

                #[derive(NetabaseModel)]
                pub struct Post {
                    #[primary_key]
                    id: u64,
                }
            }
        };

        let meta = ModuleVisitor::parse_module(&attr, &module).unwrap();
        assert_eq!(meta.models.len(), 2);
    }

    #[test]
    fn test_error_invalid_subscription() {
        let attr: Attribute = parse_quote! {
            #[netabase_definition_module(TestDef, TestDefKeys, subscriptions(Updates))]
        };

        let module: ItemMod = parse_quote! {
            pub mod test_def {
                #[derive(NetabaseModel)]
                #[subscribe(InvalidTopic)]
                pub struct User {
                    #[primary_key]
                    id: u64,
                }
            }
        };

        let result = ModuleVisitor::parse_module(&attr, &module);
        assert!(result.is_err());
    }
}
