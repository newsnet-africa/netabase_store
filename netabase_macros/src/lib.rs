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
/// // Generates: UserEmailSecondaryKey(String)         <- Note: Model-prefixed!
/// // Generates: UserDepartmentSecondaryKey(String)    <- Note: Model-prefixed!
/// // Generates: UserSecondaryKeys { Email(UserEmailSecondaryKey), Department(UserDepartmentSecondaryKey) }
/// // Generates: UserKeys { Primary(UserPrimaryKey), Secondary(UserSecondaryKeys) }
/// ```
///
/// **Important:** Secondary key types are **model-prefixed** to avoid naming conflicts
/// when multiple models have fields with the same name. For example:
///
/// ```
/// // Both models have an 'email' field
/// pub struct User {
///     #[secondary_key]
///     pub email: String,  // → UserEmailSecondaryKey
/// }
///
/// pub struct Admin {
///     #[secondary_key]
///     pub email: String,  // → AdminEmailSecondaryKey (no conflict!)
/// }
/// ```
///
/// Query syntax:
/// ```
/// // Use the model-prefixed type name
/// tree.get_by_secondary_key(
///     UserSecondaryKeys::Email(UserEmailSecondaryKey("user@example.com".to_string()))
/// )?;
/// ```
///
/// # What Gets Generated
///
/// For a model like this:
/// ```no_run
/// #[derive(NetabaseModel, Clone, bincode::Encode, bincode::Decode)]
/// #[netabase(MyDef)]
/// pub struct User {
///     #[primary_key]
///     pub id: u64,
///     pub name: String,
///     #[secondary_key]
///     pub email: String,
///     #[secondary_key]
///     pub age: u32,
/// }
/// ```
///
/// The macro generates the following code:
///
/// ## 1. Primary Key Newtype
/// ```
/// # use netabase_store::{NetabaseModel, netabase};
/// # #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode)]
/// # #[netabase(MyDef)]
/// # pub struct User {
/// #     #[primary_key]
/// #     pub id: u64,
/// #     pub email: String,
/// #     pub age: u32,
/// # }
/// // The macro generates (simplified for illustration):
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// pub struct UserPrimaryKey(pub u64);
/// ```
/// **Why:** Type safety prevents accidentally using a PostPrimaryKey with a User tree.
/// **How to use:** `tree.get(UserPrimaryKey(1))?` or `user.primary_key()`
///
/// ## 2. Secondary Key Newtypes
/// ```
/// # use netabase_store::{NetabaseModel, netabase};
/// # #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode)]
/// # #[netabase(MyDef)]
/// # pub struct User {
/// #     #[primary_key]
/// #     pub id: u64,
/// #     #[secondary_key]
/// #     pub email: String,
/// #     #[secondary_key]
/// #     pub age: u32,
/// # }
/// // The macro generates:
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// pub struct UserEmailSecondaryKey(pub String);
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// pub struct UserAgeSecondaryKey(pub u32);
/// ```
/// **Why:** Model-prefixed to avoid conflicts when multiple models have `email` fields.
/// **How to use:** Part of the SecondaryKeys enum (see below).
///
/// ## 3. Secondary Keys Enum
/// ```
/// # use netabase_store::{NetabaseModel, netabase};
/// # #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode)]
/// # #[netabase(MyDef)]
/// # pub struct User {
/// #     #[primary_key]
/// #     pub id: u64,
/// #     #[secondary_key]
/// #     pub email: String,
/// #     #[secondary_key]
/// #     pub age: u32,
/// # }
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// # pub struct UserEmailSecondaryKey(pub String);
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// # pub struct UserAgeSecondaryKey(pub u32);
/// // The macro generates:
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// pub enum UserSecondaryKeys {
///     Email(UserEmailSecondaryKey),
///     Age(UserAgeSecondaryKey),
/// }
/// ```
/// **Why:** Unified type for querying by any secondary key.
/// **How to use:** `tree.get_by_secondary_key(UserSecondaryKeys::Email(...))?`
///
/// ## 4. Combined Keys Enum
/// ```
/// # use netabase_store::{NetabaseModel, netabase};
/// # #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode)]
/// # #[netabase(MyDef)]
/// # pub struct User {
/// #     #[primary_key]
/// #     pub id: u64,
/// #     #[secondary_key]
/// #     pub email: String,
/// # }
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// # pub struct UserPrimaryKey(pub u64);
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// # pub struct UserEmailSecondaryKey(pub String);
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// # pub enum UserSecondaryKeys { Email(UserEmailSecondaryKey) }
/// // The macro generates:
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// pub enum UserKey {
///     Primary(UserPrimaryKey),
///     Secondary(UserSecondaryKeys),
/// }
/// ```
/// **Why:** Allows working with any key type in batch operations.
/// **How to use:** Usually automatic, but can use `UserKey::Primary(...)` explicitly.
///
/// ## 5. NetabaseModelTrait Implementation
/// ```
/// # use netabase_store::{NetabaseModel, netabase};
/// # use netabase_store::traits::model::NetabaseModelTrait;
/// # #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode)]
/// # #[netabase(MyDef)]
/// # pub struct User {
/// #     #[primary_key]
/// #     pub id: u64,
/// #     pub email: String,
/// #     pub age: u32,
/// # }
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// # pub struct UserPrimaryKey(pub u64);
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// # pub struct UserEmailSecondaryKey(pub String);
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// # pub struct UserAgeSecondaryKey(pub u32);
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// # pub enum UserSecondaryKeys { Email(UserEmailSecondaryKey), Age(UserAgeSecondaryKey) }
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// # pub enum UserKey { Primary(UserPrimaryKey), Secondary(UserSecondaryKeys) }
/// // The macro generates (example shown - actual implementation is more detailed):
/// impl NetabaseModelTrait<MyDef> for User {
///     type PrimaryKey = UserPrimaryKey;
///     type SecondaryKeys = UserSecondaryKeys;
///     type Keys = UserKey;
///
///     fn primary_key(&self) -> Self::PrimaryKey {
///         UserPrimaryKey(self.id)
///     }
///
///     fn secondary_keys(&self) -> Vec<Self::SecondaryKeys> {
///         vec![
///             UserSecondaryKeys::Email(UserEmailSecondaryKey(self.email.clone())),
///             UserSecondaryKeys::Age(UserAgeSecondaryKey(self.age)),
///         ]
///     }
///
///     fn discriminant_name() -> &'static str { "User" }
/// }
/// # struct MyDef;
/// ```
/// **Why:** Provides runtime access to keys from model instances.
/// **How to use:** Automatic - called internally by tree operations.
///
/// ## 6. Borrow Implementations
/// ```
/// # use std::borrow::Borrow;
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// # pub struct UserPrimaryKey(pub u64);
/// # #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
/// # pub struct UserEmailSecondaryKey(pub String);
/// // The macro generates:
/// impl Borrow<u64> for UserPrimaryKey {
///     fn borrow(&self) -> &u64 { &self.0 }
/// }
/// impl Borrow<String> for UserEmailSecondaryKey {
///     fn borrow(&self) -> &String { &self.0 }
/// }
/// // ... more Borrow impls for other key types
/// ```
/// **Why:** Enables efficient lookups without allocating new key instances.
/// **How to use:** Automatic - allows `tree.get(&1)` instead of `tree.get(UserPrimaryKey(1))`.
///
/// # Why This Architecture?
///
/// 1. **Type Safety** - Can't accidentally use wrong key type with wrong model
/// 2. **Zero Cost** - Newtypes compile to the same code as raw types
/// 3. **Ergonomics** - Single trait covers all models, consistent API
/// 4. **Flexibility** - Easy to add new models or key types
/// 5. **Performance** - Borrow traits enable zero-allocation lookups
///
/// # Common Patterns
///
/// ## Inserting a Model
/// ```rust
/// use netabase_store::{NetabaseModel, netabase_definition_module};
/// use netabase_store::databases::sled_store::SledStore;
/// use netabase_store::traits::tree::NetabaseTreeSync;
///
/// #[netabase_definition_module(MyDef, MyKeys)]
/// mod models {
///     use netabase_store::{NetabaseModel, netabase};
///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
///              bincode::Encode, bincode::Decode,
///              serde::Serialize, serde::Deserialize)]
///     #[netabase(MyDef)]
///     pub struct User {
///         #[primary_key]
///         pub id: u64,
///         pub name: String,
///         #[secondary_key]
///         pub email: String,
///         pub age: u32,
///     }
/// }
/// use models::*;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let store = SledStore::<MyDef>::temp()?;
/// let tree = store.open_tree::<User>();
///
/// let user = User { id: 1, name: "Alice".into(), email: "alice@example.com".into(), age: 30 };
/// tree.put(user)?;  // Automatically extracts and stores both primary and secondary keys
/// # Ok(())
/// # }
/// ```
///
/// ## Querying by Primary Key
/// ```rust
/// use netabase_store::{NetabaseModel, netabase_definition_module};
/// use netabase_store::databases::sled_store::SledStore;
/// use netabase_store::traits::tree::NetabaseTreeSync;
/// use netabase_store::traits::model::NetabaseModelTrait;
///
/// #[netabase_definition_module(MyDef, MyKeys)]
/// mod models {
///     use netabase_store::{NetabaseModel, netabase};
///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
///              bincode::Encode, bincode::Decode,
///              serde::Serialize, serde::Deserialize)]
///     #[netabase(MyDef)]
///     pub struct User {
///         #[primary_key]
///         pub id: u64,
///         pub name: String,
///     }
/// }
/// use models::*;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let store = SledStore::<MyDef>::temp()?;
/// let tree = store.open_tree::<User>();
/// let user = User { id: 1, name: "Alice".into() };
/// tree.put(user.clone())?;
///
/// // Query with newtype
/// let retrieved = tree.get(user.primary_key())?;
/// assert!(retrieved.is_some());
///
/// // Or with borrowing (Sled backend):
/// let retrieved2 = tree.get(&1)?;
/// assert_eq!(retrieved2.unwrap(), user);
/// # Ok(())
/// # }
/// ```
///
/// ## Querying by Secondary Key
///
/// ### Verbose API
/// ```rust
/// use netabase_store::{NetabaseModel, netabase_definition_module};
/// use netabase_store::databases::sled_store::SledStore;
/// use netabase_store::traits::tree::NetabaseTreeSync;
///
/// #[netabase_definition_module(MyDef, MyKeys)]
/// mod models {
///     use netabase_store::{NetabaseModel, netabase};
///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
///              bincode::Encode, bincode::Decode,
///              serde::Serialize, serde::Deserialize)]
///     #[netabase(MyDef)]
///     pub struct User {
///         #[primary_key]
///         pub id: u64,
///         #[secondary_key]
///         pub email: String,
///     }
/// }
/// use models::*;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let store = SledStore::<MyDef>::temp()?;
/// let tree = store.open_tree::<User>();
/// let user = User { id: 1, email: "alice@example.com".into() };
/// tree.put(user)?;
///
/// let users = tree.get_by_secondary_key(
///     UserSecondaryKeys::Email(UserEmailSecondaryKey("alice@example.com".to_string()))
/// )?;
/// // Returns Vec<User> since multiple users could share the same secondary key value
/// assert_eq!(users.len(), 1);
/// # Ok(())
/// # }
/// ```
///
/// ### Ergonomic API (Convenience Extension Traits)
///
/// The macro also generates extension traits for each secondary key:
/// ```rust
/// use netabase_store::{NetabaseModel, netabase_definition_module};
/// use netabase_store::databases::sled_store::SledStore;
/// use netabase_store::traits::tree::NetabaseTreeSync;
///
/// #[netabase_definition_module(MyDef, MyKeys)]
/// mod models {
///     use netabase_store::{NetabaseModel, netabase};
///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
///              bincode::Encode, bincode::Decode,
///              serde::Serialize, serde::Deserialize)]
///     #[netabase(MyDef)]
///     pub struct User {
///         #[primary_key]
///         pub id: u64,
///         #[secondary_key]
///         pub email: String,
///     }
/// }
/// use models::*;
/// use models::AsUserEmail;  // Generated trait
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let store = SledStore::<MyDef>::temp()?;
/// let tree = store.open_tree::<User>();
/// let user = User { id: 1, email: "alice@example.com".into() };
/// tree.put(user)?;
///
/// // Much more ergonomic!
/// let users = tree.get_by_secondary_key("alice@example.com".as_user_email_key())?;
/// assert_eq!(users.len(), 1);
/// # Ok(())
/// # }
/// ```
///
/// Generated traits follow the pattern `As{Model}{Field}` with method `as_{model}_{field}_key()`:
/// - `AsUserEmail` with `as_user_email_key()`
/// - `AsUserAge` with `as_user_age_key()`
/// - `AsPostPublished` with `as_post_published_key()`
///
/// The traits are implemented for:
/// - `String` fields: `String`, `&str`, `&String`
/// - Numeric/bool fields: The type itself and `&Type`
///
/// ## Supported Primary Key Types
///
/// - Primitives: `u8`, `u16`, `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`
/// - `String`
/// - `uuid::Uuid` (with uuid crate)
/// - Any type that implements `bincode::Encode + bincode::Decode + Clone`
///
/// # Troubleshooting
///
/// **Error: "expected type, found module"**
/// - Make sure you imported the definition: `use my_module::MyDefinition;`
///
/// **Error: "trait bounds not satisfied"**
/// - Ensure your struct derives all required traits: `Clone`, `bincode::Encode`, `bincode::Decode`
///
/// **Error: "no primary key found"**
/// - Add exactly one `#[primary_key]` attribute to a field
///
/// # See Also
///
/// - [`netabase_definition_module`] - Groups multiple models into a schema
#[proc_macro_derive(NetabaseModel, attributes(primary_key, secondary_key, link))]
pub fn netabase_model_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut visitor = ModelVisitor::default();
    visitor.visit_derive_input(&input);
    let (p, sl, s, k) = visitor.generate_keys();
    let trait_impl = visitor.generate_model_trait_impl();
    let borrow_impls = visitor.generate_borrow_impls();
    let extension_traits = visitor.generate_key_extension_traits();

    quote! {
        #p
        #(#sl)*
        #s
        #k
        #(#trait_impl)*
        #(#borrow_impls)*
        #(#extension_traits)*
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

/// Derive macro for NetabaseDiscriminant trait.
///
/// This is a marker trait implementation that confirms a type satisfies all
/// the bounds required by NetabaseDiscriminant. The type must already
/// implement all the required traits (Clone, Copy, Debug, etc.).
///
/// This macro is automatically applied by `#[netabase_definition_module]` to
/// generated discriminant enums, but can also be used manually if needed.
///
/// # Example
///
/// This derive is automatically applied by `#[netabase_definition_module]`.
/// Manual usage would look like:
///
/// ```
/// # // This example shows the generated code structure (for illustration)
/// # use netabase_store::traits::definition::NetabaseDiscriminant;
/// # use strum::{Display, AsRefStr, EnumIter, EnumString};
/// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Display, AsRefStr, EnumIter, EnumString)]
/// enum MyDiscriminant {
///     Variant1,
///     Variant2,
/// }
/// // The NetabaseDiscriminant trait is implemented via blanket impl
/// // when all the required traits are present
/// ```
/// (Note: This is for illustration only - the macro handles this automatically)
#[proc_macro_derive(NetabaseDiscriminant)]
pub fn netabase_discriminant_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Generate the implementation
    // This is a marker trait, so we just need to confirm the type satisfies the bounds
    quote! {
        // NetabaseDiscriminant is a blanket implementation, so implementing it
        // just requires that all the bounds are satisfied.
        // We use a const assertion to verify at compile time that the type implements the trait.
        const _: () = {
            fn assert_netabase_discriminant<T: ::netabase_store::traits::definition::NetabaseDiscriminant>() {}
            fn assert_impl() {
                assert_netabase_discriminant::<#name>();
            }
        };
    }
    .into()
}

/// Attribute macro for zero-copy redb optimization.
///
/// This macro modifies the struct to inject a cached borrowed reference field and
/// generates borrowed reference types (`*Ref<'a>`) that enable zero-copy reads
/// from redb by using tuple-based serialization and borrowed string slices.
///
/// # Requirements
///
/// - Must be used with `#[cfg_attr(feature = "redb-zerocopy", redb_zerocopy)]`
/// - Model fields must use only [redb-native types](../REDB_ZEROCOPY.md#supported-types)
/// - Struct must also derive `NetabaseModel`
///
/// # Supported Types
///
/// - **Primitives** (Copy): `u8`-`u128`, `i8`-`i128`, `f32`, `f64`, `bool`, `char`
/// - **Zero-copy**: `String` → `&'a str`, `Vec<u8>` → `&'a [u8]`
/// - **Compound**: `Option<T>`, `[T; N]`, tuples
///
/// See [`REDB_ZEROCOPY.md`](../REDB_ZEROCOPY.md) for complete documentation.
///
/// # Example
///
/// ```ignore
/// #[derive(NetabaseModel, Clone, bincode::Encode, bincode::Decode)]
/// #[cfg_attr(feature = "redb-zerocopy", redb_zerocopy)]
/// #[netabase(BlogDefinition)]
/// struct User {
///     #[primary_key]
///     id: u64,
///     name: String,      // Borrowed as &str
///     email: String,     // Borrowed as &str
/// }
/// ```
///
/// This modifies the struct to add:
/// ```ignore
/// #[cfg(feature = "redb-zerocopy")]
/// _borrowed_ref: std::cell::OnceCell<UserBorrowed>
/// ```
///
/// And generates:
/// - `UserRef<'a>` - Borrowed type with `name: &'a str`, `email: &'a str`
/// - `UserBorrowed` - Ouroboros self-referential wrapper
/// - `impl User { fn as_ref(&self) -> UserRef<'_> }` - Convert to borrowed
/// - `impl Borrow<UserRef<'_>> for User` - Proper Borrow trait
/// - `impl From<UserRef<'a>> for User` - Convert back to owned
/// - `impl redb::Value for User` - Tuple-based serialization with zero-copy reads
///
/// # Performance
///
/// With `redb-zerocopy` enabled:
/// - **Read operations**: ~6.6x faster (68µs → 10µs for 100 items)
/// - **No string allocations** on reads
/// - Write operations unchanged
///
/// See [`REDB_ZEROCOPY_PHASES.md`](../REDB_ZEROCOPY_PHASES.md) for implementation details.
#[cfg(feature = "redb-zerocopy")]
#[proc_macro_attribute]
pub fn redb_zerocopy(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as syn::ItemStruct);

    // Inject the cached reference field using VisitMut
    generators::zerocopy::inject_ref_field(&mut item_struct);

    // Generate all the components
    let borrowed_type = generators::zerocopy::generate_borrowed_type(&item_struct);
    let as_ref_method = generators::zerocopy::generate_as_ref_method(&item_struct);
    let from_borrowed = generators::zerocopy::generate_from_borrowed(&item_struct);
    let value_impl = generators::zerocopy::generate_value_impl(&item_struct);
    let ouroboros_wrapper = generators::zerocopy::generate_ouroboros_wrapper(&item_struct);
    let borrow_impl = generators::zerocopy::generate_borrow_impl(&item_struct);

    quote! {
        #item_struct
        #borrowed_type
        #ouroboros_wrapper
        #as_ref_method
        #from_borrowed
        #borrow_impl
        #value_impl
    }
    .into()
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
/// # What Gets Generated
///
/// For a definition module with models `User` and `Post`:
///
/// ```ignore
/// #[netabase_definition_module(BlogSchema, BlogKeys)]
/// mod blog {
///     #[derive(NetabaseModel, ...)]
///     #[netabase(BlogSchema)]
///     pub struct User { #[primary_key] pub id: u64, ... }
///
///     #[derive(NetabaseModel, ...)]
///     #[netabase(BlogSchema)]
///     pub struct Post { #[primary_key] pub id: String, ... }
/// }
/// ```
///
/// The macro generates:
///
/// ## 1. Definition Enum
/// ```ignore
/// #[derive(Clone, Debug, PartialEq, Eq, bincode::Encode, bincode::Decode, ...)]
/// #[derive(strum::EnumDiscriminants, strum::IntoStaticStr, ...)]
/// #[strum_discriminants(derive(EnumIter, Display, AsRefStr, EnumString, Hash))]
/// #[strum_discriminants(name(BlogSchemaDiscriminant))]
/// pub enum BlogSchema {
///     User(User),
///     Post(Post),
/// }
/// ```
/// **Why:** Allows storing any model from this schema in a unified type.
/// **How to use:** Usually automatic, but can do `BlogSchema::User(user)` explicitly.
///
/// ## 2. Discriminant Enum (Auto-generated by strum)
/// ```ignore
/// pub enum BlogSchemaDiscriminant {
///     User,
///     Post,
/// }
/// ```
/// **Why:** Used as table/tree names, provides efficient type identification.
/// **How to use:** Automatic - used internally to identify which model type.
///
/// ## 3. Keys Enum
/// ```ignore
/// #[derive(Clone, Debug, PartialEq, Eq, bincode::Encode, bincode::Decode, ...)]
/// #[derive(strum::EnumDiscriminants)]
/// #[strum_discriminants(name(BlogKeysDiscriminant))]
/// pub enum BlogKeys {
///     UserKey(UserKey),
///     PostKey(PostKey),
/// }
/// ```
/// **Why:** Allows working with keys from any model in batch operations.
/// **How to use:** `tree.get_by_key(BlogKeys::UserKey(UserKey::Primary(...)))?`
///
/// ## 4. Table Definitions Struct (Redb only)
/// ```ignore
/// pub struct BlogSchemaTables {
///     pub user: TableDefinition<'static, BincodeWrapper<UserPrimaryKey>, BincodeWrapper<User>>,
///     pub user_secondary: MultimapTableDefinition<'static, BincodeWrapper<CompositeKey<...>>, ()>,
///     pub post: TableDefinition<'static, BincodeWrapper<PostPrimaryKey>, BincodeWrapper<Post>>,
///     pub post_secondary: MultimapTableDefinition<'static, BincodeWrapper<CompositeKey<...>>, ()>,
/// }
/// ```
/// **Why:** Redb requires static table definitions for zero-copy operations.
/// **How to use:** Automatic - accessed via `store.tables()`.
///
/// ## 5. NetabaseDefinitionTrait Implementation
/// ```ignore
/// impl NetabaseDefinitionTrait for BlogSchema {
///     type Keys = BlogKeys;
///     type Discriminant = BlogSchemaDiscriminant;
///     type Tables = BlogSchemaTables;
/// }
/// ```
/// **Why:** Enables generic code that works with any definition.
/// **How to use:** Automatic - used by `SledStore<BlogSchema>`, etc.
///
/// ## 6. Conversion Traits
/// ```ignore
/// // Convert from specific model to definition
/// impl From<User> for BlogSchema {
///     fn from(value: User) -> Self { BlogSchema::User(value) }
/// }
/// impl From<Post> for BlogSchema {
///     fn from(value: Post) -> Self { BlogSchema::Post(value) }
/// }
///
/// // Convert from definition to specific model
/// impl TryFrom<BlogSchema> for User {
///     type Error = String;
///     fn try_from(value: BlogSchema) -> Result<Self, Self::Error> {
///         match value {
///             BlogSchema::User(u) => Ok(u),
///             _ => Err("Expected User variant".to_string()),
///         }
///     }
/// }
/// // ... similar for Post
/// ```
/// **Why:** Enables type-safe conversions between models and definition enum.
/// **How to use:** Usually automatic, but can use `.into()` and `.try_into()`.
///
/// # Why This Architecture?
///
/// 1. **Schema Cohesion** - Related models grouped together logically
/// 2. **Type Safety** - Can't mix models from different schemas
/// 3. **Performance** - Discriminants enable O(1) type identification
/// 4. **Flexibility** - Easy to add new models to existing schema
/// 5. **Backend Agnostic** - Same schema works with Sled, Redb, IndexedDB
///
/// # Common Patterns
///
/// ## Creating a Store with a Definition
/// ```ignore
/// let store = SledStore::<BlogSchema>::new("./blog.db")?;
/// // Or
/// let store = RedbStore::<BlogSchema>::new("./blog.redb")?;
/// ```
///
/// ## Opening Trees for Different Models
/// ```ignore
/// let users = store.open_tree::<User>();
/// let posts = store.open_tree::<Post>();
///
/// users.put(User { id: 1, ... })?;
/// posts.put(Post { id: "post-1".into(), ... })?;
/// ```
///
/// ## Working with the Definition Enum
/// ```ignore
/// // Store any model in the definition
/// let item: BlogSchema = user.into();
/// // Or
/// let item = BlogSchema::User(user);
///
/// // Extract specific model back
/// let user: User = item.try_into()?;
/// ```
///
/// # Troubleshooting
///
/// **Error: "expected type, found macro"**
/// - Ensure macro is imported: `use netabase_store::netabase_definition_module;`
///
/// **Error: "no models found in module"**
/// - Make sure at least one struct has `#[derive(NetabaseModel)]` and `#[netabase(DefinitionName)]`
///
/// **Error: "mismatched definition names"**
/// - All models must use the same definition name in `#[netabase(...)]`
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

    // Generate redb table definitions struct
    let tables_struct =
        generators::table_definitions::generate_tables_struct(&visitor.modules, definition);
    let tables_impl =
        generators::table_definitions::generate_tables_impl(&visitor.modules, definition);
    let tables_name = syn::Ident::new(&format!("{}Tables", definition), definition.span());

    let trait_impls =
        visitor.generate_definition_trait_impls(definition, definition_key, &tables_name);

    // Generate discriminant type names for compile-time assertions
    let discriminant_name =
        syn::Ident::new(&format!("{}Discriminant", definition), definition.span());
    let key_discriminant_name = syn::Ident::new(
        &format!("{}Discriminant", definition_key),
        definition_key.span(),
    );

    // Generate compile-time assertions for discriminant trait bounds
    let discriminant_assertions: syn::Item = syn::parse_quote! {
        const _: () = {
            fn assert_discriminant<T: ::netabase_store::traits::definition::NetabaseDiscriminant>() {}
            fn assert_key_discriminant<T: ::netabase_store::traits::definition::NetabaseKeyDiscriminant>() {}
            fn _check() {
                assert_discriminant::<#discriminant_name>();
                assert_key_discriminant::<#key_discriminant_name>();
            }
        };
    };

    if let Some((_, c)) = &mut def_module.content {
        c.push(syn::Item::Enum(defin));
        c.push(syn::Item::Enum(def_key));
        c.push(syn::Item::Struct(tables_struct));
        c.push(discriminant_assertions);
    };

    quote! {
        #def_module
        #trait_impls
        #tables_impl
    }
    .into()
}
