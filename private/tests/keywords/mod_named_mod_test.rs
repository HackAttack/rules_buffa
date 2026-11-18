//! Tests module names that are keywords.

use imported_keyword_proto::imported::r#type::B;
use mod_named_mod_proto::r#mod::A;

#[test]
fn test_nested_messages() {
    let a = A {
        name: "a".to_string(),
        b: B {
            name: "b".to_string(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    assert_eq!(a.name, "a");
}
