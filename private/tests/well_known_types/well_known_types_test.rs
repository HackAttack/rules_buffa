//! Tests the Google well-known types.
//!
//! Unlike prost, buffa does not generate crates for the well-known types;
//! `google.protobuf.*` resolves to `buffa_types`, and `descriptor.proto` /
//! `compiler/plugin.proto` resolve to `buffa_descriptor::generated`.

use buffa_descriptor::generated::compiler::Version;
use buffa_descriptor::generated::descriptor::DescriptorProto;
use buffa_types::google::protobuf::field::{Cardinality, Kind};
use buffa_types::google::protobuf::{
    Any, Api, BoolValue, BytesValue, DoubleValue, Duration, Empty, Field, FieldMask, FloatValue,
    Int32Value, Int64Value, Method, Mixin, Option, SourceContext, StringValue, Struct, Syntax,
    Timestamp, Type, UInt32Value, UInt64Value, Value,
};
use well_known_types_proto::wkt::WellKnownTypes;

#[test]
fn test_well_known_types() {
    let wkt = WellKnownTypes {
        any: Any {
            type_url: "type.googleapis.com/google.protobuf.Any".to_string(),
            value: vec![].into(),
            ..Default::default()
        }
        .into(),
        api: Api {
            name: "api".to_string(),
            methods: vec![Method {
                name: "method".to_string(),
                ..Default::default()
            }],
            options: vec![Option {
                name: "option".to_string(),
                ..Default::default()
            }],
            version: "1.2.3".to_string(),
            source_context: SourceContext {
                file_name: "file_name".to_string(),
                ..Default::default()
            }
            .into(),
            mixins: vec![Mixin {
                name: "mixin".to_string(),
                ..Default::default()
            }],
            syntax: Syntax::Proto3.into(),
            ..Default::default()
        }
        .into(),
        method: Method {
            name: "method".to_string(),
            ..Default::default()
        }
        .into(),
        mixin: Mixin {
            name: "mixin".to_string(),
            ..Default::default()
        }
        .into(),
        compiler_version: Version {
            major: Some(1),
            minor: Some(2),
            patch: Some(3),
            suffix: Some("suffix".to_string()),
            ..Default::default()
        }
        .into(),
        descriptor_proto: DescriptorProto::default().into(),
        empty: Empty::default().into(),
        duration: Duration {
            seconds: 1,
            nanos: 2,
            ..Default::default()
        }
        .into(),
        field_mask: FieldMask {
            paths: vec!["path".to_string()],
            ..Default::default()
        }
        .into(),
        source_context: SourceContext {
            file_name: "file_name".to_string(),
            ..Default::default()
        }
        .into(),
        r#struct: Struct {
            fields: vec![("field".to_string(), Value::default())]
                .into_iter()
                .collect(),
            ..Default::default()
        }
        .into(),
        timestamp: Timestamp {
            seconds: 1,
            nanos: 2,
            ..Default::default()
        }
        .into(),
        r#type: Type {
            name: "type".to_string(),
            fields: vec![Field {
                kind: Kind::TypeDouble.into(),
                cardinality: Cardinality::Required.into(),
                ..Default::default()
            }],
            ..Default::default()
        }
        .into(),
        bool_value: BoolValue {
            value: true,
            ..Default::default()
        }
        .into(),
        bytes_value: BytesValue {
            value: vec![],
            ..Default::default()
        }
        .into(),
        double_value: DoubleValue {
            value: 1.0,
            ..Default::default()
        }
        .into(),
        float_value: FloatValue {
            value: 1.1,
            ..Default::default()
        }
        .into(),
        int32_value: Int32Value {
            value: 2,
            ..Default::default()
        }
        .into(),
        int64_value: Int64Value {
            value: 3,
            ..Default::default()
        }
        .into(),
        string_value: StringValue {
            value: "value".to_string(),
            ..Default::default()
        }
        .into(),
        uint32_value: UInt32Value {
            value: 4,
            ..Default::default()
        }
        .into(),
        uint64_value: UInt64Value {
            value: 5,
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };

    assert!(wkt.any.is_set());
}
