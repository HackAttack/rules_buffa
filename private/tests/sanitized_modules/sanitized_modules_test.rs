//! Tests protos with various capitalizations in their package names are
//! consumable in an expected way.
//!
//! Unlike prost, buffa preserves package segment casing verbatim in module
//! names (`bAR`, `gaRply`); only nested-message modules are snake_case.

use bar_proto::bAR::bAz::QAZ::QuX::bar::Baz as BazMessage;
use bar_proto::bAR::bAz::QAZ::QuX::Bar as BarMessage;
use bar_proto::foo_proto::Foo::QuuX::CoRgE::GRAULT::gaRply::foo::NestedFoo as NestedFooMessage;
use bar_proto::foo_proto::Foo::QuuX::CoRgE::GRAULT::gaRply::Foo as FooMessage;

#[test]
fn test_packages() {
    let bar_message = BarMessage {
        name: "bar".to_string(),
        foo: FooMessage {
            name: "foo".to_string(),
            ..Default::default()
        }
        .into(),
        nested_foo: NestedFooMessage {
            name: "nested_foo".to_string(),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };
    let baz_message = BazMessage {
        name: "baz".to_string(),
        ..Default::default()
    };

    assert_eq!(bar_message.name, "bar");
    assert_eq!(baz_message.name, "baz");
}
