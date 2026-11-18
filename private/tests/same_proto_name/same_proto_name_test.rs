//! Tests three `models` proto_libraries disambiguated by transform
//! `crate_name` overrides produce usable crates.

use package1_models::package1::Model as Package1Model;
use package2_models::package2::Model as Package2Model;
use package3_models::package3::Model as Package3Model;

#[test]
fn same_proto_name() {
    let model = Package1Model {
        model: Package2Model {
            model: Package3Model {
                stuff: "x".to_string(),
                ..Default::default()
            }
            .into(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    assert_eq!("x", model.model.model.stuff);
}
