//! Tests `extern_path`: a user mapping beats `stamp_proto`'s manifest entry
//! for `.tests.stamp.Stamp`, so `Event::at` is a WKT `Timestamp`.

use buffa::Message;

#[test]
fn extern_path_overrides_manifest() {
    let event = stamped_proto::tests::stamped::Event {
        at: buffa_types::google::protobuf::Timestamp {
            seconds: 5,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    let bytes = event.encode_to_vec();
    let decoded = stamped_proto::tests::stamped::Event::decode_from_slice(&bytes).unwrap();
    assert_eq!(event, decoded);
    assert_eq!(5, decoded.at.seconds);
}
