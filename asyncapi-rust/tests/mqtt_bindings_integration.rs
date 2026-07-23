//! Integration tests that derive a type with `mqtt(...)` bindings on a server,
//! operation, and message and assert the bindings appear in the generated
//! `AsyncApiSpec`. Unlike the `extract_*` unit tests (which only cover parsing)
//! and the `mqtt_bindings` example (which only prints), these assert the emitted
//! output, guarding against silent codegen regressions.

use asyncapi_rust::{AsyncApi, ToAsyncApiMessage, schemars::JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[asyncapi(mqtt(
    payload_format_indicator = 1,
    content_type = "application/json",
    binding_version = "1.0"
))]
pub struct SensorReading {
    pub value: f64,
}

#[derive(AsyncApi)]
#[asyncapi(title = "MQTT Binding Test API", version = "1.0.0")]
#[asyncapi_server(
    name = "production",
    host = "mqtt.example.com",
    protocol = "mqtt",
    mqtt(
        client_id = "test-client",
        clean_session = false,
        keep_alive = 60,
        session_expiry_interval = 3600,
        maximum_packet_size = 1200,
        binding_version = "1.0"
    )
)]
#[asyncapi_channel(name = "sensors", address = "/sensors")]
#[asyncapi_operation(
    name = "publishReading",
    action = "send",
    channel = "sensors",
    mqtt(
        qos = 2,
        retain = true,
        message_expiry_interval = 300,
        binding_version = "1.0"
    )
)]
#[asyncapi_messages(SensorReading)]
pub struct MqttBindingApi;

#[test]
fn server_mqtt_bindings_emitted() {
    let spec = MqttBindingApi::asyncapi_spec();
    let servers = spec.servers.expect("servers present");
    let server = servers
        .get("production")
        .expect("production server present");
    let mqtt = server
        .bindings
        .as_ref()
        .and_then(|b| b.mqtt.as_ref())
        .expect("server mqtt bindings present");

    assert_eq!(mqtt.client_id.as_deref(), Some("test-client"));
    assert_eq!(mqtt.clean_session, Some(false));
    assert_eq!(mqtt.keep_alive, Some(60));
    assert_eq!(mqtt.binding_version.as_deref(), Some("1.0"));
    // Numeric-or-schema fields: assert they were carried through.
    assert!(mqtt.session_expiry_interval.is_some());
    assert!(mqtt.max_packet_size.is_some());
}

#[test]
fn operation_mqtt_bindings_emitted() {
    let spec = MqttBindingApi::asyncapi_spec();
    let operations = spec.operations.expect("operations present");
    let op = operations
        .get("publishReading")
        .expect("publishReading operation present");
    let mqtt = op
        .bindings
        .as_ref()
        .and_then(|b| b.mqtt.as_ref())
        .expect("operation mqtt bindings present");

    assert_eq!(mqtt.qos, Some(2));
    assert_eq!(mqtt.retain, Some(true));
    assert_eq!(mqtt.binding_version.as_deref(), Some("1.0"));
    assert!(mqtt.message_expiry_interval.is_some());
}

#[test]
fn message_mqtt_bindings_emitted() {
    let spec = MqttBindingApi::asyncapi_spec();
    let components = spec.components.expect("components present");
    let messages = components.messages.expect("messages present");
    let message = messages
        .get("SensorReading")
        .expect("SensorReading message present");
    let mqtt = message
        .bindings
        .as_ref()
        .and_then(|b| b.mqtt.as_ref())
        .expect("message mqtt bindings present");

    assert_eq!(mqtt.payload_format_indicator, Some(1));
    assert_eq!(mqtt.content_type.as_deref(), Some("application/json"));
    assert_eq!(mqtt.binding_version.as_deref(), Some("1.0"));
}
