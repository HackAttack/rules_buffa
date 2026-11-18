//! Tests transitive dependencies.

use b_proto::a::b::B;
use b_proto::c_proto::a::b::c::C;

#[test]
fn test_b() {
    let b = B {
        name: "b".to_string(),
        c: C {
            name: "c".to_string(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    assert_eq!(b.name, "b");
    assert_eq!(b.c.as_option().unwrap().name, "c");
}
