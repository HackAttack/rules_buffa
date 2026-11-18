//! Tests transitive dependencies.

use a_proto::a::A;
use a_proto::b_proto::c_proto::a::b::c::C;
use buffa_types::google::protobuf::{Duration, Timestamp};

#[test]
fn test_a() {
    let duration = Duration {
        seconds: 1,
        nanos: 2,
        ..Default::default()
    };

    let a = A {
        name: "a".to_string(),
        // Ensure the external `b_proto` dependency is compatible with `a_proto`'s `B`.
        b: b_proto::a::b::B {
            name: "b".to_string(),
            c: C {
                name: "c".to_string(),
                duration: duration.clone().into(),
                ..Default::default()
            }
            .into(),
            ..Default::default()
        }
        .into(),
        timestamp: Timestamp {
            seconds: 1,
            nanos: 2,
            ..Default::default()
        }
        .into(),
        duration: duration.clone().into(),
        types: a_proto::types_proto::Types::default().into(),
        ..Default::default()
    };

    let b_ref = a.b.as_option().unwrap();
    let c_ref = b_ref.c.as_option().unwrap();

    assert_eq!(a.name, "a");
    assert_eq!(b_ref.name, "b");
    assert_eq!(c_ref.name, "c");
    assert_eq!(a.duration.as_option(), c_ref.duration.as_option());
}
