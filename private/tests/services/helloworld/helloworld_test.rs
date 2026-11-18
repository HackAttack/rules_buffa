//! Tests the `connect_buffa_toolchain`: the generated crate contains the
//! `Greeter` service trait and its service-name constant in addition to the
//! message types. Under the default (message-only) toolchain this file fails
//! to compile.

use buffa::bytes::Bytes;
use buffa::{Message, MessageView};
use connectrpc::{CodecFormat, Encodable, RequestContext, Response, ServiceRequest, ServiceResult};
use futures::executor::block_on;
use helloworld_proto::helloworld::__buffa::view::HelloRequestView;
use helloworld_proto::helloworld::{GREETER_SERVICE_NAME, Greeter, HelloReply, HelloRequest};

struct MyGreeter;

#[allow(refining_impl_trait_internal)]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, HelloRequest>,
    ) -> ServiceResult<HelloReply> {
        Response::ok(HelloReply {
            message: format!("Hello {}!", request.name),
            ..Default::default()
        })
    }
}

#[test]
fn unary_handler_round_trip() {
    assert_eq!(GREETER_SERVICE_NAME, "helloworld.Greeter");

    let body = Bytes::from(
        HelloRequest {
            name: "ada".to_string(),
            ..Default::default()
        }
        .encode_to_vec(),
    );
    let view = HelloRequestView::decode_view(&body).unwrap();
    let request = ServiceRequest::<HelloRequest>::from_parts(&view, &body);
    let response = block_on(MyGreeter.say_hello(RequestContext::default(), request)).unwrap();
    let reply = HelloReply::decode_from_slice(
        &Encodable::encode(&response.body, CodecFormat::Proto).unwrap(),
    )
    .unwrap();
    assert_eq!(reply.message, "Hello ada!");
}
