//! Tests `rust_buffa_transform`: a crate-name override, a codegen option,
//! an extra dependency, and a source file appended to the generated crate.

use buffa::Message;

#[test]
fn transform_applies() {
    let tagged = renamed_d::tests::transformed::Tagged {
        label: "x".to_string(),
        ..Default::default()
    };
    assert_eq!("X", tagged.shout());
    assert_eq!(
        r#"{"label":"x"}"#,
        buffa::serde_json::to_string(&tagged).unwrap()
    );

    let holder = e_proto::tests::transformed::user::Holder {
        tagged: tagged.into(),
        ..Default::default()
    };
    let bytes = holder.encode_to_vec();
    let decoded = e_proto::tests::transformed::user::Holder::decode_from_slice(&bytes).unwrap();
    assert_eq!(holder, decoded);
}
