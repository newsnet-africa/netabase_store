use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, Ident, ItemMod, Token, parse::Parser, parse_macro_input, punctuated::Punctuated,
    visit::Visit,
};

use crate::visitors::{definitions_visitor::DefinitionsVisitor, model_visitor::ModelVisitor};

mod errors;
mod generators;
mod item_info;
mod util;
mod visitors;

/// Derives the `NetabaseModelTrait` for a struct, enabling it to be stored in netabase.
///
/// This macro automatically generates:
/// - Primary key newtype: `{ModelName}PrimaryKey(T)`
/// - Secondary keys enum: `{ModelName}SecondaryKeys` with variants for each `#[secondary_key]`
/// - Combined keys enum: `{ModelName}Keys`
/// - Implementation of `NetabaseModelTrait<D>`
///
/// # Attributes
///
/// - `#[primary_key]` - **Required**. Marks exactly one field as the primary key
/// - `#[secondary_key]` - **Optional**. Marks fields that can be queried efficiently
/// - `#[link]` - **Future**. Reserved for foreign key relationships
///
/// # Required Derives
///
/// Your struct must also derive:
/// - `Clone` - For internal operations
/// - `bincode::Encode` - For serialization
/// - `bincode::Decode` - For deserialization
/// - `serde::Serialize` - For JSON serialization (optional but recommended)
/// - `serde::Deserialize` - For JSON deserialization (optional but recommended)
///
/// # Examples
///
/// ## Basic Model with Primary Key Only
///
/// ```
/// use netabase_store::NetabaseModel;
///
/// #[derive(NetabaseModel, Clone, Debug,
///          bincode::Encode, bincode::Decode,
///          serde::Serialize, serde::Deserialize)]
/// #[netabase(MyDefinition)]
/// pub struct SimpleModel {
///     #[primary_key]
///     pub id: u64,
///     pub data: String,
/// }
/// // Generates: SimpleModelPrimaryKey(u64)
/// // Generates: SimpleModelSecondaryKeys (empty enum)
/// // Generates: SimpleModelKeys
/// ```
///
/// ## Model with Secondary Keys
///
/// ```
/// #[derive(NetabaseModel, Clone, Debug,
///          bincode::Encode, bincode::Decode,
///          serde::Serialize, serde::Deserialize)]
/// #[netabase(MyDefinition)]
/// pub struct User {
///     #[primary_key]
///     pub id: u64,
///     pub name: String,
///     #[secondary_key]
///     pub email: String,
///     #[secondary_key]
///     pub department: String,
/// }
/// // Generates: UserPrimaryKey(u64)
/// // Generates: UserSecondaryKeys { EmailKey(String), DepartmentKey(String) }
/// // Generates: UserKeys { Primary(UserPrimaryKey), Secondary(UserSecondaryKeys) }
/// ```
///
/// ## Supported Primary Key Types
///
/// - Primitives: `u8`, `u16`, `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`
/// - `String`
/// - `uuid::Uuid` (with uuid crate)
/// - Any type that implements `bincode::Encode + bincode::Decode + Clone`
///
/// # See Also
///
/// - [`netabase_definition_module`] - Groups multiple models into a schema
/// - [`NetabaseModelTrait`](crate::traits::model::NetabaseModelTrait) - The trait this macro implements
#[proc_macro_derive(NetabaseModel, attributes(primary_key, secondary_key, link))]
pub fn netabase_model_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut visitor = ModelVisitor::default();
    visitor.visit_derive_input(&input);
    let (p, sl, s, k) = visitor.generate_keys();
    let trait_impl = visitor.generate_model_trait_impl();

    quote! {
        #p
        #(#sl)*
        #s
        #k
        #(#trait_impl)*
    }
    .into()
}

/// Attribute macro that marks a struct as part of a netabase definition.
///
/// This is used in conjunction with `#[netabase_definition_module]` to specify
/// which definition a model belongs to. This is a pass-through attribute that
/// doesn't modify the struct itself.
///
/// # Usage
///
/// ```
/// #[derive(NetabaseModel, ...)]
/// #[netabase(MyDefinition)]  // Links this model to MyDefinition
/// pub struct MyModel {
///     #[primary_key]
///     pub id: u64,
/// }
/// ```
///
/// # See Also
///
/// - [`netabase_definition_module`] - The macro that processes this attribute
/// - [`NetabaseModel`] - Must be derived on the same struct
#[proc_macro_attribute]
pub fn netabase(_defs: TokenStream, input: TokenStream) -> TokenStream {
    input
}

/// Groups multiple models into a unified database schema (definition).
///
/// This macro processes a module containing models and generates:
/// - A definition enum that wraps all models
/// - A keys enum that wraps all model keys
/// - Trait implementations for working with the definition
/// - Conversion traits between models and the definition enum
///
/// # Syntax
///
/// ```
/// #[netabase_definition_module(DefinitionName, KeysEnumName)]
/// mod my_schema {
///     use super::*;  // Import NetabaseModel, netabase
///
///     #[derive(NetabaseModel, ...)]
///     #[netabase(DefinitionName)]
///     pub struct Model1 { ... }
///
///     #[derive(NetabaseModel, ...)]
///     #[netabase(DefinitionName)]
///     pub struct Model2 { ... }
/// }
/// ```
///
/// # Generated Code
///
/// For a module with models `User` and `Post`, this generates:
///
/// ```
/// pub enum DefinitionName {
///     User(User),
///     Post(Post),
/// }
///
/// pub enum KeysEnumName {
///     UserPrimary(UserPrimaryKey),
///     UserSecondary(UserSecondaryKeys),
///     PostPrimary(PostPrimaryKey),
///     PostSecondary(PostSecondaryKeys),
/// }
///
/// // Plus trait implementations for NetabaseDefinitionTrait
/// // Plus From/TryFrom conversions
/// ```
///
/// # Complete Example
///
/// ```
/// use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
///
/// #[netabase_definition_module(BlogSchema, BlogKeys)]
/// mod blog {
///     use super::*;
///
///     #[derive(NetabaseModel, Clone, Debug,
///              bincode::Encode, bincode::Decode,
///              serde::Serialize, serde::Deserialize)]
///     #[netabase(BlogSchema)]
///     pub struct User {
///         #[primary_key]
///         pub id: u64,
///         pub name: String,
///         #[secondary_key]
///         pub email: String,
///     }
///
///     #[derive(NetabaseModel, Clone, Debug,
///              bincode::Encode, bincode::Decode,
///              serde::Serialize, serde::Deserialize)]
///     #[netabase(BlogSchema)]
///     pub struct Post {
///         #[primary_key]
///         pub id: u64,
///         pub title: String,
///         pub author_id: u64,
///         #[secondary_key]
///         pub published: bool,
///     }
/// }
///
/// use blog::*;
///
/// // Use with a store
/// let store = SledStore::<BlogSchema>::new("./blog_db")?;
/// let users = store.open_tree::<User>();
/// let posts = store.open_tree::<Post>();
/// ```
///
/// # Type Safety
///
/// The definition provides compile-time type safety by ensuring:
/// - Only models belonging to this definition can be stored together
/// - Keys are correctly typed for their models
/// - Conversions between models and the definition are type-checked
///
/// # See Also
///
/// - [`NetabaseModel`] - Derive macro for individual models
/// - [`netabase`] - Attribute linking models to definitions
/// - [`NetabaseDefinitionTrait`](crate::traits::definition::NetabaseDefinitionTrait) - The trait this generates
#[proc_macro_attribute]
pub fn netabase_definition_module(name: TokenStream, input: TokenStream) -> TokenStream {
    let mut def_module = parse_macro_input!(input as ItemMod);
    let mut visitor = DefinitionsVisitor::default();
    visitor.visit_item_mod(&def_module);
    let list = match Punctuated::<Ident, Token![,]>::parse_terminated.parse(name) {
        Ok(l) => l,
        Err(e) => panic!("Error parsing Definitions module: {e}"),
    };
    let definition = list.first().unwrap();
    let definition_key = list.last().unwrap();
    let (defin, def_key) = visitor.generate_definitions(definition, definition_key);
    let trait_impls = visitor.generate_definition_trait_impls(definition, definition_key);

    if let Some((_, c)) = &mut def_module.content {
        c.push(syn::Item::Enum(defin));
        c.push(syn::Item::Enum(def_key));
    };

    quote! {
        #def_module
        #trait_impls
    }
    .into()
}
