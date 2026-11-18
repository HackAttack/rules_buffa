//! Tests the `json` toolchain option: generated types derive serde traits when
//! `json=true` is set on the `rust_buffa_toolchain`.

#[test]
fn serialize_to_json() {
    let inner = a_proto::tests::shared::Inner {
        name: "x".to_string(),
        nested: a_proto::tests::shared::inner::Nested {
            id: 7,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    let json = buffa::serde_json::to_string(&inner).unwrap();
    assert_eq!(r#"{"name":"x","nested":{"id":7}}"#, json);
}
