//! Tests buffa codegen across crates, including a proto package split across
//! two `proto_library` targets (`a_proto` and `b_proto`).

use buffa::Message;

#[test]
fn round_trip_across_crates() {
    let wrapper = c_proto::tests::other::Wrapper {
        outer: b_proto::tests::shared::Outer {
            inner: a_proto::tests::shared::Inner {
                name: "inner".to_string(),
                nested: a_proto::tests::shared::inner::Nested {
                    id: 7,
                    ..Default::default()
                }
                .into(),
                ..Default::default()
            }
            .into(),
            nested: a_proto::tests::shared::inner::Nested {
                id: 42,
                ..Default::default()
            }
            .into(),
            kind: a_proto::tests::shared::Kind::KIND_ONE.into(),
            created: buffa_types::google::protobuf::Timestamp {
                seconds: 1234,
                nanos: 0,
                ..Default::default()
            }
            .into(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    let bytes = wrapper.encode_to_vec();
    let decoded = c_proto::tests::other::Wrapper::decode_from_slice(&bytes).unwrap();

    assert_eq!(wrapper, decoded);
    assert_eq!(42, decoded.outer.nested.id);
}
