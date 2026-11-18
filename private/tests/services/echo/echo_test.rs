//! Exercises the four RPC shapes `connect_buffa_toolchain` stubs generate:
//! unary, server streaming, client streaming, and bidi streaming.

use buffa::bytes::Bytes;
use buffa::{Message, MessageView};
use connectrpc::{
    CodecFormat, Encodable, InboundStream, RequestContext, Response, ServiceRequest, ServiceResult,
    ServiceStream, StreamMessage,
};
use echo_proto::echo::__buffa::view::EchoRequestView;
use echo_proto::echo::{ECHO_SERVICE_NAME, Echo, EchoRequest, EchoResponse};
use futures::executor::block_on;
use futures::{StreamExt, TryStreamExt, stream};

struct MyEcho;

#[allow(refining_impl_trait_internal)]
impl Echo for MyEcho {
    async fn unary_echo(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, EchoRequest>,
    ) -> ServiceResult<EchoResponse> {
        Response::ok(EchoResponse {
            message: request.message.to_string(),
            ..Default::default()
        })
    }

    async fn server_streaming_echo(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, EchoRequest>,
    ) -> ServiceResult<ServiceStream<EchoResponse>> {
        let message = request.message.to_string();
        let stream = stream::iter([
            Ok(EchoResponse {
                message: message.clone(),
                ..Default::default()
            }),
            Ok(EchoResponse {
                message,
                ..Default::default()
            }),
        ]);
        Response::stream_ok(stream)
    }

    async fn client_streaming_echo(
        &self,
        _ctx: RequestContext,
        mut requests: InboundStream<EchoRequest>,
    ) -> ServiceResult<EchoResponse> {
        let mut message = String::new();
        while let Some(item) = requests.try_next().await? {
            message.push_str(item.message());
        }
        Response::ok(EchoResponse {
            message,
            ..Default::default()
        })
    }

    async fn bidirectional_streaming_echo(
        &self,
        _ctx: RequestContext,
        requests: InboundStream<EchoRequest>,
    ) -> ServiceResult<ServiceStream<EchoResponse>> {
        let stream = requests.map(|item| {
            item.map(|request| EchoResponse {
                message: request.to_owned_message().message,
                ..Default::default()
            })
        });
        Response::stream_ok(stream)
    }
}

fn echo_request(message: &str) -> EchoRequest {
    EchoRequest {
        message: message.to_string(),
        ..Default::default()
    }
}

#[test]
fn all_rpc_shapes_round_trip() {
    assert_eq!(ECHO_SERVICE_NAME, "echo.Echo");

    // Unary.
    let body = Bytes::from(echo_request("hi").encode_to_vec());
    let view = EchoRequestView::decode_view(&body).unwrap();
    let request = ServiceRequest::<EchoRequest>::from_parts(&view, &body);
    let response = block_on(MyEcho.unary_echo(RequestContext::default(), request)).unwrap();
    assert_eq!(
        EchoResponse::decode_from_slice(
            &Encodable::encode(&response.body, CodecFormat::Proto).unwrap()
        )
        .unwrap()
        .message,
        "hi"
    );

    // Server streaming.
    let body = Bytes::from(echo_request("hello").encode_to_vec());
    let view = EchoRequestView::decode_view(&body).unwrap();
    let request = ServiceRequest::<EchoRequest>::from_parts(&view, &body);
    let response =
        block_on(MyEcho.server_streaming_echo(RequestContext::default(), request)).unwrap();
    let items: Vec<_> = block_on(response.body.try_collect()).unwrap();
    let messages: Vec<_> = items
        .iter()
        .map(|item| {
            EchoResponse::decode_from_slice(&Encodable::encode(item, CodecFormat::Proto).unwrap())
                .unwrap()
                .message
        })
        .collect();
    assert_eq!(messages, ["hello", "hello"]);

    // Client streaming.
    let requests: InboundStream<EchoRequest> = Box::pin(stream::iter([
        Ok(StreamMessage::from_message(&echo_request("foo"))),
        Ok(StreamMessage::from_message(&echo_request("bar"))),
    ]));
    let response =
        block_on(MyEcho.client_streaming_echo(RequestContext::default(), requests)).unwrap();
    assert_eq!(
        EchoResponse::decode_from_slice(
            &Encodable::encode(&response.body, CodecFormat::Proto).unwrap()
        )
        .unwrap()
        .message,
        "foobar"
    );

    // Bidi streaming.
    let requests: InboundStream<EchoRequest> = Box::pin(stream::iter([
        Ok(StreamMessage::from_message(&echo_request("one"))),
        Ok(StreamMessage::from_message(&echo_request("two"))),
    ]));
    let response =
        block_on(MyEcho.bidirectional_streaming_echo(RequestContext::default(), requests)).unwrap();
    let items: Vec<_> = block_on(response.body.try_collect()).unwrap();
    let messages: Vec<_> = items
        .iter()
        .map(|item| {
            EchoResponse::decode_from_slice(&Encodable::encode(item, CodecFormat::Proto).unwrap())
                .unwrap()
                .message
        })
        .collect();
    assert_eq!(messages, ["one", "two"]);
}
