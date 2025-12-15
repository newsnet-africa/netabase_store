pub mod databases;
pub mod errors;
pub mod relational;
pub mod traits;

#[cfg(test)]
mod tests {
    use super::*;
    
    // Include the cross-definition test
    include!("../example/cross_definition_test.rs");
}
