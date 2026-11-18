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
            // `unknown_fields=false` leaves `name` the only field.
            c: C {
                name: "c".to_string(),
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
        duration: duration.into(),
        types: a_proto::types_proto::Types::default().into(),
        ..Default::default()
    };

    assert_eq!(
        "Display of: A",
        format!("{}", a),
        "Unexpected `Display` implementation for {:#?}",
        a
    );
}

#[test]
fn test_b() {
    use b_proto::Greeting;

    let b = b_proto::a::b::B {
        name: "b".to_string(),
        // `unknown_fields=false` leaves `name` the only field.
        c: C {
            name: "c".to_string(),
        }
        .into(),
        ..Default::default()
    };

    assert_eq!("Hallo, Bazel, my name is B!", b.greet("Bazel"));
}
