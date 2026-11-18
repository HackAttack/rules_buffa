//! Tests transitive dependencies.

use c_proto::a::b::c::C;

#[test]
fn test_c() {
    let c = C {
        name: "c".to_string(),
        ..Default::default()
    };

    assert_eq!(c.name, "c");
}
