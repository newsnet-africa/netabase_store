use proc_macro2::Ident;
use syn::Path;

/// Utilities for generating consistent names for wrapper types, enums, and other generated items

/// Generate the primary key type name (e.g., User -> UserID)
pub fn primary_key_type_name(model_name: &Ident) -> Ident {
    quote::format_ident!("{}ID", model_name)
}

/// Generate wrapper type name for a field (e.g., User.name -> UserName)
pub fn field_wrapper_name(model_name: &Ident, field_name: &Ident) -> Ident {
    let field_pascal = to_pascal_case(&field_name.to_string());
    quote::format_ident!("{}{}", model_name, field_pascal)
}

/// Generate secondary keys enum name (e.g., User -> UserSecondaryKeys)
pub fn secondary_keys_enum_name(model_name: &Ident) -> Ident {
    quote::format_ident!("{}SecondaryKeys", model_name)
}

/// Generate relational keys enum name (e.g., User -> UserRelationalKeys)
pub fn relational_keys_enum_name(model_name: &Ident) -> Ident {
    quote::format_ident!("{}RelationalKeys", model_name)
}

/// Generate subscriptions enum name (e.g., User -> UserSubscriptions)
pub fn subscriptions_enum_name(model_name: &Ident) -> Ident {
    quote::format_ident!("{}Subscriptions", model_name)
}

/// Generate blob keys enum name (e.g., User -> UserBlobKeys)
pub fn blob_keys_enum_name(model_name: &Ident) -> Ident {
    quote::format_ident!("{}BlobKeys", model_name)
}

/// Generate blob item enum name (e.g., User -> UserBlobItem)
pub fn blob_item_enum_name(model_name: &Ident) -> Ident {
    quote::format_ident!("{}BlobItem", model_name)
}

/// Generate unified keys enum name (e.g., User -> UserKeys)
pub fn unified_keys_enum_name(model_name: &Ident) -> Ident {
    quote::format_ident!("{}Keys", model_name)
}

/// Generate TreeName discriminant name (e.g., UserSecondaryKeys -> UserSecondaryKeysTreeName)
pub fn tree_name_type(base_name: &Ident) -> Ident {
    quote::format_ident!("{}TreeName", base_name)
}

/// Generate definition subscriptions enum name (e.g., Definition -> DefinitionSubscriptions)
pub fn definition_subscriptions_enum_name(definition_name: &Ident) -> Ident {
    quote::format_ident!("{}Subscriptions", definition_name)
}

/// Generate definition tree name type (e.g., Definition -> DefinitionTreeName)
pub fn definition_tree_name_type(definition_name: &Ident) -> Ident {
    quote::format_ident!("{}TreeName", definition_name)
}

/// Generate definition complex tree names enum name (e.g., Definition -> DefinitionTreeNames)
pub fn definition_tree_names_enum_name(definition_name: &Ident) -> Ident {
    quote::format_ident!("{}TreeNames", definition_name)
}

/// Generate definition keys enum name (e.g., Definition -> DefinitionKeys)
pub fn definition_keys_enum_name(definition_name: &Ident) -> Ident {
    quote::format_ident!("{}Keys", definition_name)
}

/// Generate table name string for database tables
pub fn table_name(
    definition_name: &str,
    model_name: &str,
    key_type: &str,
    key_name: &str,
) -> String {
    format!("{}:{}:{}:{}", definition_name, model_name, key_type, key_name)
}

/// Generate subscription table name
pub fn subscription_table_name(definition_name: &str, subscription_name: &str) -> String {
    format!("{}:Subscription:{}", definition_name, subscription_name)
}

/// Convert snake_case to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Extract the last segment from a path (e.g., foo::bar::Baz -> Baz)
pub fn path_last_segment(path: &Path) -> Option<&Ident> {
    path.segments.last().map(|seg| &seg.ident)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("user_name"), "UserName");
        assert_eq!(to_pascal_case("id"), "Id");
        assert_eq!(to_pascal_case("first_name_last_name"), "FirstNameLastName");
    }

    #[test]
    fn test_table_name() {
        assert_eq!(
            table_name("Definition", "User", "Secondary", "Name"),
            "Definition:User:Secondary:Name"
        );
    }
}
