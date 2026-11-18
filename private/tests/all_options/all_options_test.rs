//! Every codegen option that needs a runtime crate feature works with the
//! module's vendored runtime, and the `jiff` and `chrono` conversions apply to
//! generated well-known-type fields.

use all_options_proto::tests::all_options::Everything;
use arbitrary::{Arbitrary, Unstructured};
use buffa_descriptor::reflect::{Reflectable, ValueRef};
use buffa_types::google::protobuf::{Duration, Timestamp};

fn everything() -> Everything {
    Everything {
        name: "x".to_string(),
        created: Timestamp::from(jiff::Timestamp::from_second(1_700_000_000).unwrap()).into(),
        elapsed: Duration::from(chrono::TimeDelta::seconds(90)).into(),
        ..Default::default()
    }
}

#[test]
fn json() {
    assert_eq!(
        buffa::serde_json::to_string(&everything()).unwrap(),
        r#"{"name":"x","created":"2023-11-14T22:13:20Z","elapsed":"90s"}"#
    );
}

#[test]
fn text() {
    let text = buffa::text::encode_to_string(&everything());
    let decoded: Everything = buffa::text::decode_from_str(&text).unwrap();
    assert_eq!(decoded, everything());
}

#[test]
fn reflection() {
    let msg = everything();
    let reflected = msg.reflect();
    let descriptor = reflected.message_descriptor();
    assert_eq!(descriptor.full_name(), "tests.all_options.Everything");
    let name = descriptor.field_by_name("name").unwrap();
    assert!(matches!(reflected.get(name), ValueRef::String("x")));
}

#[test]
fn arbitrary() {
    Everything::arbitrary(&mut Unstructured::new(&[0x5a; 256])).unwrap();
}

#[test]
fn time_conversions() {
    let msg = everything();
    assert_eq!(
        jiff::Timestamp::try_from((*msg.created).clone()).unwrap(),
        jiff::Timestamp::from_second(1_700_000_000).unwrap()
    );
    assert_eq!(
        chrono::TimeDelta::try_from((*msg.elapsed).clone()).unwrap(),
        chrono::TimeDelta::seconds(90)
    );
}
