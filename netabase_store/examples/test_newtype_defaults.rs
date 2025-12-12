use strum::IntoEnumIterator;

// Test just the type definitions without the complex trait implementations
use bincode::{Decode, Encode};
use strum::{AsRefStr, EnumDiscriminants, EnumIter};

// =================================================================================
// EXTRACTED TYPE DEFINITIONS FOR TESTING
// =================================================================================

// User types
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct UserId(pub u64);

impl Default for UserId {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct UserEmail(pub String);

impl Default for UserEmail {
    fn default() -> Self {
        Self(String::new())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct UserName(pub String);

impl Default for UserName {
    fn default() -> Self {
        Self(String::new())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct UserAge(pub u32);

impl Default for UserAge {
    fn default() -> Self {
        Self(0)
    }
}

// Product types
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ProductId(pub u64);

impl Default for ProductId {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ProductTitle(pub String);

impl Default for ProductTitle {
    fn default() -> Self {
        Self(String::new())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct ProductScore(pub u32);

impl Default for ProductScore {
    fn default() -> Self {
        Self(0)
    }
}

// Enum with EnumIter that uses these newtypes
#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
pub enum UserSecondaryKeys {
    Email(UserEmail),
    Name(UserName),
    Age(UserAge),
}

#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
pub enum ProductSecondaryKeys {
    Title(ProductTitle),
    Score(ProductScore),
}

#[derive(Debug, Clone, EnumDiscriminants, EnumIter, Encode, Decode)]
pub enum RelationalKeys {
    UserProduct(UserId, ProductId),
    ProductUser(ProductId, UserId),
}

// Tree names with Default
#[derive(Debug, Clone, Copy, PartialEq, AsRefStr, EnumIter)]
#[strum(serialize_all = "PascalCase")]
pub enum TreeName {
    Main,
    Secondary,
    Relational,
}

impl Default for TreeName {
    fn default() -> Self {
        Self::Main
    }
}

fn main() {
    println!("=== Testing Default implementations for newtypes used in EnumIter ===");
    
    // Test newtype defaults
    let user_id = UserId::default();
    let user_email = UserEmail::default();
    let user_name = UserName::default();
    let user_age = UserAge::default();
    
    println!("UserId::default(): {:?}", user_id);
    println!("UserEmail::default(): {:?}", user_email);
    println!("UserName::default(): {:?}", user_name);
    println!("UserAge::default(): {:?}", user_age);
    
    assert_eq!(user_id, UserId(0));
    assert_eq!(user_email, UserEmail(String::new()));
    assert_eq!(user_name, UserName(String::new()));
    assert_eq!(user_age, UserAge(0));
    
    // Test product defaults
    let product_id = ProductId::default();
    let product_title = ProductTitle::default();
    let product_score = ProductScore::default();
    
    println!("ProductId::default(): {:?}", product_id);
    println!("ProductTitle::default(): {:?}", product_title);
    println!("ProductScore::default(): {:?}", product_score);
    
    assert_eq!(product_id, ProductId(0));
    assert_eq!(product_title, ProductTitle(String::new()));
    assert_eq!(product_score, ProductScore(0));
    
    // Test tree name default
    let tree_name = TreeName::default();
    println!("TreeName::default(): {:?}", tree_name);
    assert_eq!(tree_name, TreeName::Main);
    
    // Test that enums with EnumIter work with these defaults
    println!("\n=== Testing EnumIter with default newtypes ===");
    
    // Create enum variants using defaults
    let user_email_key = UserSecondaryKeys::Email(UserEmail::default());
    let user_name_key = UserSecondaryKeys::Name(UserName::default());
    let user_age_key = UserSecondaryKeys::Age(UserAge::default());
    
    println!("UserSecondaryKeys variants with defaults:");
    println!("  Email: {:?}", user_email_key);
    println!("  Name: {:?}", user_name_key);
    println!("  Age: {:?}", user_age_key);
    
    // Test iteration
    println!("\nIterating over UserSecondaryKeys variants:");
    // Note: This is testing the discriminant iteration, not the values
    for variant in UserSecondaryKeys::iter() {
        match variant {
            UserSecondaryKeys::Email(email) => println!("  Email variant: {:?}", email),
            UserSecondaryKeys::Name(name) => println!("  Name variant: {:?}", name),
            UserSecondaryKeys::Age(age) => println!("  Age variant: {:?}", age),
        }
    }
    
    println!("\nIterating over TreeName variants:");
    for variant in TreeName::iter() {
        println!("  TreeName: {:?} (as_ref: '{}')", variant, variant.as_ref());
    }
    
    // Test type safety - different newtype structs cannot be mixed
    let _user_id: UserId = UserId::default();
    let _product_id: ProductId = ProductId::default();
    // This would be a compile error:
    // let _mixed: UserId = ProductId::default();
    
    println!("\n✅ All newtype Default implementations working correctly!");
    println!("✅ EnumIter works with default newtype values!");
    println!("✅ Type safety maintained - cannot mix incompatible newtypes!");
    println!("✅ Default trait bounds satisfied for EnumIter enums!");
}