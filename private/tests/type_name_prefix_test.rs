//! Tests that a `type_name_prefix` transform on the owning crate is visible
//! to dependents: `prefixed_proto`'s extern-path manifest records the
//! prefixed names, so `prefix_user_proto` resolves `tests.prefixed.Thing` to
//! `RpcThing` without a transform of its own.

use buffa::Message;
use prefix_user_proto::tests::prefix_user::Holder;
use prefixed_proto::tests::prefixed::thing::RpcPart;
use prefixed_proto::tests::prefixed::RpcThing;

#[test]
fn type_name_prefix_flows_through_manifest() {
    let holder = Holder {
        thing: RpcThing {
            name: "n".to_string(),
            part: RpcPart {
                n: 1,
                ..Default::default()
            }
            .into(),
            ..Default::default()
        }
        .into(),
        part: RpcPart {
            n: 2,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    let bytes = holder.encode_to_vec();
    assert_eq!(holder, Holder::decode_from_slice(&bytes).unwrap());
}
