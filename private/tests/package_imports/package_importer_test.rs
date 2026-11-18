//! Tests package importing with the same package name.

use package_importer_proto::package::import::B;
use package_importer_proto::package_import_proto::package::import::A;

#[test]
fn test_package_importer() {
    let b = B {
        a: A {
            name: "a".to_string(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    assert_eq!(b.a.as_option().unwrap().name, "a");
}
