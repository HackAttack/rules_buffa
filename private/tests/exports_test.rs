//! Tests `import public`: `reexport_proto` re-exports `a_proto`'s crate, so
//! `importer_proto` resolves `tests.shared.Inner` while depending only on
//! `reexport_proto`.

use buffa::Message;

#[test]
fn public_import_reexports_dep_crate() {
    let uses = importer_proto::tests::importer::Uses {
        inner: a_proto::tests::shared::Inner {
            name: "inner".to_string(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    let bytes = uses.encode_to_vec();
    let decoded = importer_proto::tests::importer::Uses::decode_from_slice(&bytes).unwrap();
    assert_eq!(uses, decoded);
    assert_eq!("inner", decoded.inner.name);
}
