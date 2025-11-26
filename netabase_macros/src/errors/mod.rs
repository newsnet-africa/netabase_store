use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum MacroError {
    #[error("There was an error parsing a derive Macro: {0:?}")]
    Derive(#[from] DeriveError),
}

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum DeriveError {
    #[error("Failed to parse Netabase Model")]
    Model(#[from] NetabaseModelDeriveError),
}

#[derive(Error, Debug)]
pub enum NetabaseModelDeriveError {
    #[error(
        "Could not find a primary key.\n\
         \n\
         Help: Add exactly one `#[primary_key]` attribute to a field in your struct.\n\
         \n\
         Example:\n\
         #[derive(NetabaseModel, Clone, bincode::Encode, bincode::Decode)]\n\
         #[netabase(YourDefinition)]\n\
         pub struct YourModel {{\n\
             #[primary_key]  // <- Add this\n\
             pub id: u64,\n\
             pub other_field: String,\n\
         }}\n\
         \n\
         Note: Every model must have exactly ONE primary key field."
    )]
    PrimaryKeyNotFound,

    #[error(
        "Incorrect model type: expected a struct with named fields.\n\
         \n\
         Help: NetabaseModel can only be derived on structs with named fields.\n\
         \n\
         Supported:\n\
           pub struct User {{ pub id: u64, pub name: String }}  // ✓\n\
         \n\
         Not supported:\n\
           pub struct User(u64, String);  // ✗ Tuple structs\n\
           pub struct User;               // ✗ Unit structs\n\
           pub enum User {{ ... }}          // ✗ Enums"
    )]
    IncorrectModelType,

    #[error("Failed to parse Link path")]
    LinkPath(#[from] LinkPathError),

    #[error(
        "Macro visitor encountered an error while processing the model.\n\
         \n\
         Help: This is usually caused by:\n\
           1. Missing required derives: Clone, bincode::Encode, bincode::Decode\n\
           2. Invalid field types that don't implement required traits\n\
           3. Syntax errors in attributes\n\
         \n\
         Make sure your struct looks like:\n\
         #[derive(NetabaseModel, Clone, bincode::Encode, bincode::Decode)]\n\
         #[netabase(YourDefinition)]\n\
         pub struct YourModel {{\n\
             #[primary_key]\n\
             pub id: u64,\n\
         }}"
    )]
    MacroVisitorError,
}

#[derive(Error, Debug)]
pub enum LinkPathError {
    #[error("There was an eror parsing the link attribute")]
    Parse(proc_macro2::TokenStream),
    #[error("The link attribute was not a metalist")]
    IncorrectAttribute,
}
