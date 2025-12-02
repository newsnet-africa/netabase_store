//! Test file for RecursionLevel functionality without relational models

use netabase_store::links::RecursionLevel;

#[test]
fn test_recursion_level_should_recurse() {
    // Test RecursionLevel::Full
    let full = RecursionLevel::Full;
    assert!(
        full.should_recurse(0),
        "Full recursion should allow recursion at depth 0"
    );
    assert!(
        full.should_recurse(5),
        "Full recursion should allow recursion at depth 5"
    );
    assert!(
        full.should_recurse(100),
        "Full recursion should allow recursion at any depth"
    );

    // Test RecursionLevel::None
    let none = RecursionLevel::None;
    assert!(
        !none.should_recurse(0),
        "None recursion should not allow recursion at depth 0"
    );
    assert!(
        !none.should_recurse(1),
        "None recursion should not allow recursion at any depth"
    );

    // Test RecursionLevel::Value
    let limited_3 = RecursionLevel::Value(3);
    assert!(
        limited_3.should_recurse(0),
        "Value(3) should allow recursion at depth 0"
    );
    assert!(
        limited_3.should_recurse(1),
        "Value(3) should allow recursion at depth 1"
    );
    assert!(
        limited_3.should_recurse(2),
        "Value(3) should allow recursion at depth 2"
    );
    assert!(
        !limited_3.should_recurse(3),
        "Value(3) should not allow recursion at depth 3"
    );
    assert!(
        !limited_3.should_recurse(4),
        "Value(3) should not allow recursion at depth 4"
    );

    let limited_0 = RecursionLevel::Value(0);
    assert!(
        !limited_0.should_recurse(0),
        "Value(0) should not allow recursion at depth 0"
    );
    assert!(
        !limited_0.should_recurse(1),
        "Value(0) should not allow recursion at any depth"
    );

    let limited_1 = RecursionLevel::Value(1);
    assert!(
        limited_1.should_recurse(0),
        "Value(1) should allow recursion at depth 0"
    );
    assert!(
        !limited_1.should_recurse(1),
        "Value(1) should not allow recursion at depth 1"
    );
}

#[test]
fn test_recursion_level_next_level() {
    // Test RecursionLevel::Full
    let full = RecursionLevel::Full;
    assert_eq!(
        full.next_level(),
        RecursionLevel::Full,
        "Full.next_level() should remain Full"
    );

    // Test RecursionLevel::None
    let none = RecursionLevel::None;
    assert_eq!(
        none.next_level(),
        RecursionLevel::None,
        "None.next_level() should remain None"
    );

    // Test RecursionLevel::Value
    let value_5 = RecursionLevel::Value(5);
    assert_eq!(
        value_5.next_level(),
        RecursionLevel::Value(4),
        "Value(5).next_level() should be Value(4)"
    );

    let value_1 = RecursionLevel::Value(1);
    assert_eq!(
        value_1.next_level(),
        RecursionLevel::Value(0),
        "Value(1).next_level() should be Value(0)"
    );

    let value_0 = RecursionLevel::Value(0);
    assert_eq!(
        value_0.next_level(),
        RecursionLevel::None,
        "Value(0).next_level() should be None"
    );
}

#[test]
fn test_recursion_level_default() {
    let default = RecursionLevel::default();
    assert_eq!(
        default,
        RecursionLevel::Value(1),
        "Default RecursionLevel should be Value(1)"
    );
}

#[test]
fn test_recursion_level_chain() {
    // Test chaining next_level calls
    let start = RecursionLevel::Value(3);

    let level_1 = start.next_level();
    assert_eq!(level_1, RecursionLevel::Value(2));
    assert!(level_1.should_recurse(0));
    assert!(level_1.should_recurse(1));
    assert!(!level_1.should_recurse(2));

    let level_2 = level_1.next_level();
    assert_eq!(level_2, RecursionLevel::Value(1));
    assert!(level_2.should_recurse(0));
    assert!(!level_2.should_recurse(1));

    let level_3 = level_2.next_level();
    assert_eq!(level_3, RecursionLevel::Value(0));
    assert!(!level_3.should_recurse(0));

    let level_4 = level_3.next_level();
    assert_eq!(level_4, RecursionLevel::None);
    assert!(!level_4.should_recurse(0));

    let level_5 = level_4.next_level();
    assert_eq!(level_5, RecursionLevel::None);
    assert!(!level_5.should_recurse(0));
}

#[test]
fn test_recursion_level_clone_debug() {
    let original = RecursionLevel::Value(42);
    let cloned = original.clone();
    assert_eq!(original, cloned);

    // Test Debug formatting works
    let debug_string = format!("{:?}", original);
    assert!(debug_string.contains("Value"));
    assert!(debug_string.contains("42"));

    let full_debug = format!("{:?}", RecursionLevel::Full);
    assert!(full_debug.contains("Full"));

    let none_debug = format!("{:?}", RecursionLevel::None);
    assert!(none_debug.contains("None"));
}
