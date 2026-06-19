use asyncapi_rust::{AsyncApi, ToAsyncApiMessage, schemars::JsonSchema};
use serde::{Deserialize, Serialize};

// Test basic enum without serde attributes
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
pub enum BasicMessage {
    Ping,
    Pong,
}

// Test enum with serde tag
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum TaggedMessage {
    Echo { text: String },
    Broadcast { room: String, text: String },
}

// Test enum with serde rename on variants
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "message")]
pub enum RenamedMessage {
    #[serde(rename = "user.join")]
    UserJoin { username: String },
    #[serde(rename = "user.leave")]
    UserLeave { username: String },
    #[serde(rename = "chat.message")]
    ChatMessage { username: String, text: String },
}

// Test struct
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
pub struct SimpleMessage {
    pub id: u64,
    pub text: String,
}

// Test enum with asyncapi attributes
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum DocumentedMessage {
    /// Join a room
    #[asyncapi(
        summary = "User joins a chat room",
        description = "Sent when a user enters a room"
    )]
    Join { username: String, room: String },

    /// Leave a room
    #[asyncapi(
        summary = "User leaves a chat room",
        description = "Sent when a user exits a room",
        title = "Leave Room"
    )]
    Leave { username: String, room: String },

    /// Binary file transfer
    #[asyncapi(content_type = "application/octet-stream")]
    File { filename: String, data: Vec<u8> },

    /// Binary data with triggers_binary flag
    #[asyncapi(triggers_binary)]
    Binary { data: Vec<u8> },
}

#[test]
fn test_basic_enum_messages() {
    let names = BasicMessage::asyncapi_message_names();
    assert_eq!(names, vec!["Ping", "Pong"]);
    assert_eq!(BasicMessage::asyncapi_message_count(), 2);
    assert_eq!(BasicMessage::asyncapi_tag_field(), None);
}

#[test]
fn test_tagged_enum() {
    let names = TaggedMessage::asyncapi_message_names();
    assert_eq!(names, vec!["Echo", "Broadcast"]);
    assert_eq!(TaggedMessage::asyncapi_message_count(), 2);
    assert_eq!(TaggedMessage::asyncapi_tag_field(), Some("type"));
}

#[test]
fn test_renamed_enum() {
    let names = RenamedMessage::asyncapi_message_names();
    // Message names are variant identifiers, not serde rename strings.
    // The serde rename ("user.join" etc.) stays as the wire discriminant in the payload schema.
    assert_eq!(names, vec!["UserJoin", "UserLeave", "ChatMessage"]);
    assert_eq!(RenamedMessage::asyncapi_message_count(), 3);
    assert_eq!(RenamedMessage::asyncapi_tag_field(), Some("message"));
}

#[test]
fn test_struct_message() {
    let names = SimpleMessage::asyncapi_message_names();
    assert_eq!(names, vec!["SimpleMessage"]);
    assert_eq!(SimpleMessage::asyncapi_message_count(), 1);
    assert_eq!(SimpleMessage::asyncapi_tag_field(), None);
}

#[test]
fn test_schema_generation() {
    let messages = SimpleMessage::asyncapi_messages();
    assert_eq!(messages.len(), 1);

    let message = &messages[0];
    assert_eq!(message.name, Some("SimpleMessage".to_string()));
    assert_eq!(message.content_type, Some("application/json".to_string()));
    assert!(message.payload.is_some());

    // Verify the schema was generated
    if let Some(schema) = &message.payload {
        // Schema should have been converted from schemars output
        // Just verify it exists - the exact structure depends on schemars
        assert!(matches!(schema, asyncapi_rust::Schema::Object(_)));
    }
}

#[test]
fn test_enum_schema_generation() {
    let messages = TaggedMessage::asyncapi_messages();
    assert_eq!(messages.len(), 2);

    let echo = messages
        .iter()
        .find(|m| m.name.as_deref() == Some("Echo"))
        .expect("Echo message should exist");
    let broadcast = messages
        .iter()
        .find(|m| m.name.as_deref() == Some("Broadcast"))
        .expect("Broadcast message should exist");

    // Each message must carry only its own variant schema — no top-level oneOf
    let echo_json = serde_json::to_value(&echo.payload).unwrap();
    let broadcast_json = serde_json::to_value(&broadcast.payload).unwrap();

    assert!(
        echo_json.get("oneOf").is_none(),
        "Echo payload must not be the whole-enum oneOf"
    );
    assert!(
        broadcast_json.get("oneOf").is_none(),
        "Broadcast payload must not be the whole-enum oneOf"
    );

    // Each payload must contain the const for its own discriminant only
    let echo_type_const = echo_json
        .pointer("/properties/type/const")
        .and_then(|v| v.as_str());
    assert_eq!(echo_type_const, Some("Echo"));

    let broadcast_type_const = broadcast_json
        .pointer("/properties/type/const")
        .and_then(|v| v.as_str());
    assert_eq!(broadcast_type_const, Some("Broadcast"));
}

// Regression for #6: per-variant schemas work with a non-"type" tag field name
#[test]
fn test_per_variant_schema_non_type_tag() {
    #[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
    #[serde(tag = "message")]
    pub enum ChannelEvent {
        #[serde(rename = "channel-list")]
        ChannelList { channels: Vec<String> },
        #[serde(rename = "data-changed")]
        DataChanged { project_id: i64 },
    }

    let messages = ChannelEvent::asyncapi_messages();
    assert_eq!(messages.len(), 2);

    let channel_list = messages
        .iter()
        .find(|m| m.name.as_deref() == Some("ChannelList"))
        .expect("ChannelList should exist");
    let data_changed = messages
        .iter()
        .find(|m| m.name.as_deref() == Some("DataChanged"))
        .expect("DataChanged should exist");

    let cl_json = serde_json::to_value(&channel_list.payload).unwrap();
    let dc_json = serde_json::to_value(&data_changed.payload).unwrap();

    // Must be per-variant, not the whole union
    assert!(cl_json.get("oneOf").is_none());
    assert!(dc_json.get("oneOf").is_none());

    // Discriminant must match (tag field is "message", not "type")
    assert_eq!(
        cl_json
            .pointer("/properties/message/const")
            .and_then(|v| v.as_str()),
        Some("channel-list")
    );
    assert_eq!(
        dc_json
            .pointer("/properties/message/const")
            .and_then(|v| v.as_str()),
        Some("data-changed")
    );

    // channel-list payload must have "channels", not "project_id"
    assert!(cl_json.pointer("/properties/channels").is_some());
    assert!(cl_json.pointer("/properties/project_id").is_none());
}

#[test]
fn test_asyncapi_attributes() {
    let messages = DocumentedMessage::asyncapi_messages();
    assert_eq!(messages.len(), 4);

    // Test Join message with summary and description
    let join = &messages[0];
    assert_eq!(join.name, Some("Join".to_string()));
    assert_eq!(join.summary, Some("User joins a chat room".to_string()));
    assert_eq!(
        join.description,
        Some("Sent when a user enters a room".to_string())
    );
    assert_eq!(join.content_type, Some("application/json".to_string()));

    // Test Leave message with custom title
    let leave = &messages[1];
    assert_eq!(leave.name, Some("Leave".to_string()));
    assert_eq!(leave.title, Some("Leave Room".to_string()));
    assert_eq!(leave.summary, Some("User leaves a chat room".to_string()));
    assert_eq!(
        leave.description,
        Some("Sent when a user exits a room".to_string())
    );

    // Test File message with custom content type
    let file = &messages[2];
    assert_eq!(file.name, Some("File".to_string()));
    assert_eq!(
        file.content_type,
        Some("application/octet-stream".to_string())
    );

    // Test Binary message with triggers_binary flag
    let binary = &messages[3];
    assert_eq!(binary.name, Some("Binary".to_string()));
    assert_eq!(
        binary.content_type,
        Some("application/octet-stream".to_string())
    );
}

// Test AsyncApi derive macro
#[derive(AsyncApi)]
#[asyncapi(
    title = "Test API",
    version = "1.0.0",
    description = "A test API specification"
)]
struct TestApi;

#[test]
fn test_asyncapi_derive() {
    let spec = TestApi::asyncapi_spec();

    // Verify basic fields
    assert_eq!(spec.asyncapi, "3.0.0");
    assert_eq!(spec.info.title, "Test API");
    assert_eq!(spec.info.version, "1.0.0");
    assert_eq!(
        spec.info.description,
        Some("A test API specification".to_string())
    );

    // Verify optional fields are None
    assert!(spec.servers.is_none());
    assert!(spec.channels.is_none());
    assert!(spec.operations.is_none());
    assert!(spec.components.is_none());
}

// Test AsyncApi without description
#[derive(AsyncApi)]
#[asyncapi(title = "Minimal API", version = "0.1.0")]
struct MinimalApi;

#[test]
fn test_asyncapi_minimal() {
    let spec = MinimalApi::asyncapi_spec();

    assert_eq!(spec.asyncapi, "3.0.0");
    assert_eq!(spec.info.title, "Minimal API");
    assert_eq!(spec.info.version, "0.1.0");
    assert_eq!(spec.info.description, None);
}

// Test AsyncApi with servers, channels, and operations
#[allow(clippy::duplicated_attributes)] // False positive - different operations can reference same channel
#[derive(AsyncApi)]
#[asyncapi(
    title = "Full API",
    version = "1.0.0",
    description = "Complete API spec"
)]
#[asyncapi_server(
    name = "production",
    host = "api.example.com",
    protocol = "wss",
    description = "Production server"
)]
#[asyncapi_server(name = "development", host = "localhost:8080", protocol = "ws")]
#[asyncapi_channel(name = "chat", address = "/ws/chat")]
#[asyncapi_operation(name = "sendMessage", action = "send", channel = "chat")]
#[asyncapi_operation(name = "receiveMessage", action = "receive", channel = "chat")]
struct FullApi;

#[test]
fn test_asyncapi_full() {
    let spec = FullApi::asyncapi_spec();

    // Verify Info
    assert_eq!(spec.info.title, "Full API");
    assert_eq!(spec.info.version, "1.0.0");
    assert_eq!(spec.info.description, Some("Complete API spec".to_string()));

    // Verify Servers
    let servers = spec.servers.expect("Should have servers");
    assert_eq!(servers.len(), 2);

    let prod_server = servers
        .get("production")
        .expect("Should have production server");
    assert_eq!(prod_server.host, "api.example.com");
    assert_eq!(prod_server.protocol, "wss");
    assert_eq!(
        prod_server.description,
        Some("Production server".to_string())
    );

    let dev_server = servers
        .get("development")
        .expect("Should have development server");
    assert_eq!(dev_server.host, "localhost:8080");
    assert_eq!(dev_server.protocol, "ws");
    assert_eq!(dev_server.description, None);

    // Verify Channels
    let channels = spec.channels.expect("Should have channels");
    assert_eq!(channels.len(), 1);

    let chat_channel = channels.get("chat").expect("Should have chat channel");
    assert_eq!(chat_channel.address, Some("/ws/chat".to_string()));

    // Verify Operations
    let operations = spec.operations.expect("Should have operations");
    assert_eq!(operations.len(), 2);

    let send_op = operations
        .get("sendMessage")
        .expect("Should have sendMessage operation");
    assert!(matches!(
        send_op.action,
        asyncapi_rust::OperationAction::Send
    ));
    assert_eq!(send_op.channel.reference, "#/channels/chat");

    let receive_op = operations
        .get("receiveMessage")
        .expect("Should have receiveMessage operation");
    assert!(matches!(
        receive_op.action,
        asyncapi_rust::OperationAction::Receive
    ));
    assert_eq!(receive_op.channel.reference, "#/channels/chat");
}

// Test AsyncApi with message integration
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
enum ApiMessage {
    #[serde(rename = "user.join")]
    #[asyncapi(summary = "User joins", description = "User enters a room")]
    UserJoin { username: String, room: String },

    #[serde(rename = "user.leave")]
    #[asyncapi(summary = "User leaves")]
    UserLeave { username: String, room: String },
}

#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
enum SystemMessage {
    #[serde(rename = "system.status")]
    #[asyncapi(summary = "System status")]
    SystemStatus { status: String },
}

#[derive(AsyncApi)]
#[asyncapi(title = "Message Integration API", version = "1.0.0")]
#[asyncapi_messages(ApiMessage, SystemMessage)]
struct MessageIntegrationApi;

#[test]
fn test_asyncapi_with_messages() {
    let spec = MessageIntegrationApi::asyncapi_spec();

    // Verify Info
    assert_eq!(spec.info.title, "Message Integration API");
    assert_eq!(spec.info.version, "1.0.0");

    // Verify Components exist and have messages
    let components = spec.components.expect("Should have components");
    let messages = components
        .messages
        .expect("Should have messages in components");

    // Verify we have all 3 messages (2 from ApiMessage, 1 from SystemMessage).
    // Keys and name fields use variant identifiers, not serde rename strings.
    assert_eq!(messages.len(), 3);

    // Verify UserJoin message
    let user_join = messages
        .get("UserJoin")
        .expect("Should have UserJoin message");
    assert_eq!(user_join.name, Some("UserJoin".to_string()));
    assert_eq!(user_join.summary, Some("User joins".to_string()));
    assert_eq!(
        user_join.description,
        Some("User enters a room".to_string())
    );
    assert!(user_join.payload.is_some());

    // Verify UserLeave message
    let user_leave = messages
        .get("UserLeave")
        .expect("Should have UserLeave message");
    assert_eq!(user_leave.name, Some("UserLeave".to_string()));
    assert_eq!(user_leave.summary, Some("User leaves".to_string()));

    // Verify SystemStatus message
    let system_status = messages
        .get("SystemStatus")
        .expect("Should have SystemStatus message");
    assert_eq!(system_status.name, Some("SystemStatus".to_string()));
    assert_eq!(system_status.summary, Some("System status".to_string()));
}

// Regression test for issue #4: ToAsyncApiMessage panics on enums with serde_json::Value fields
#[test]
fn test_enum_with_json_value_fields() {
    #[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
    #[serde(tag = "type")]
    pub enum MsgWithValue {
        #[serde(rename = "hello")]
        Hello { version: String },
        #[serde(rename = "result")]
        Result {
            ok: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            payload: Option<serde_json::Value>,
        },
    }

    // Should not panic
    let messages = MsgWithValue::asyncapi_messages();
    assert_eq!(messages.len(), 2);

    let hello = messages
        .iter()
        .find(|m| m.name.as_deref() == Some("Hello"))
        .expect("Hello message should exist");
    assert!(hello.payload.is_some());

    let result = messages
        .iter()
        .find(|m| m.name.as_deref() == Some("Result"))
        .expect("Result message should exist");
    assert!(result.payload.is_some());
}

// Regression for #7: shared $defs hoisted to components.schemas; payloads have no $defs block
// and $refs point to #/components/schemas/X instead of #/$defs/X.
#[test]
fn test_shared_defs_hoisted_to_components_schemas() {
    // SharedInfo will appear in schemars' $defs because it's used by multiple variants
    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct SharedInfo {
        pub id: i64,
        pub label: String,
    }

    #[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
    #[serde(tag = "type")]
    pub enum EventMsg {
        #[serde(rename = "event-a")]
        EventA { info: SharedInfo },
        #[serde(rename = "event-b")]
        EventB { info: SharedInfo, extra: String },
    }

    #[derive(AsyncApi)]
    #[asyncapi(title = "Hoisting Test", version = "1.0.0")]
    #[asyncapi_messages(EventMsg)]
    struct HoistApi;

    // asyncapi_schemas() must expose the shared type
    let schemas = EventMsg::asyncapi_schemas();
    assert!(
        schemas.contains_key("SharedInfo"),
        "SharedInfo must appear in asyncapi_schemas()"
    );

    // No message payload may contain a $defs block
    let messages = EventMsg::asyncapi_messages();
    for msg in &messages {
        let payload_json = serde_json::to_value(&msg.payload).unwrap();
        assert!(
            payload_json.get("$defs").is_none(),
            "payload for '{}' must not contain $defs",
            msg.name.as_deref().unwrap_or("?")
        );
    }

    // Any $ref in a payload must point to #/components/schemas/, not #/$defs/
    fn find_bad_refs(v: &serde_json::Value) -> Vec<String> {
        let mut bad = Vec::new();
        match v {
            serde_json::Value::Object(map) => {
                if let Some(r) = map.get("$ref").and_then(|r| r.as_str()) {
                    if r.starts_with("#/$defs/") {
                        bad.push(r.to_string());
                    }
                }
                for val in map.values() {
                    bad.extend(find_bad_refs(val));
                }
            }
            serde_json::Value::Array(arr) => {
                for val in arr {
                    bad.extend(find_bad_refs(val));
                }
            }
            _ => {}
        }
        bad
    }

    for msg in &messages {
        let payload_json = serde_json::to_value(&msg.payload).unwrap();
        let bad = find_bad_refs(&payload_json);
        assert!(
            bad.is_empty(),
            "payload for '{}' still has #/$defs/ refs: {:?}",
            msg.name.as_deref().unwrap_or("?"),
            bad
        );
    }

    // The full spec must have components.schemas populated with SharedInfo
    let spec = HoistApi::asyncapi_spec();
    let comp_schemas = spec
        .components
        .as_ref()
        .and_then(|c| c.schemas.as_ref())
        .expect("components.schemas must be populated");

    assert!(
        comp_schemas.contains_key("SharedInfo"),
        "components.schemas must contain SharedInfo"
    );

    // And each message payload in components.messages must also be clean
    let comp_messages = spec
        .components
        .as_ref()
        .and_then(|c| c.messages.as_ref())
        .expect("components.messages must be populated");

    for (name, msg) in comp_messages {
        let payload_json = serde_json::to_value(&msg.payload).unwrap();
        assert!(
            payload_json.get("$defs").is_none(),
            "components.messages.{name}.payload must not contain $defs"
        );
        let bad = find_bad_refs(&payload_json);
        assert!(
            bad.is_empty(),
            "components.messages.{name}.payload has unrewritten refs: {bad:?}"
        );
    }
}

// Regression for #8: empty serde rename falls back to variant identifier
#[test]
fn test_empty_serde_rename_fallback() {
    #[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
    #[serde(tag = "message")]
    pub enum ResponseMsg {
        #[serde(rename = "ok")]
        Ok { value: String },
        #[serde(rename = "")]
        Empty,
    }

    let names = ResponseMsg::asyncapi_message_names();
    // "Empty" (variant ident) not "" (serde rename)
    assert!(
        names.contains(&"Empty"),
        "empty serde rename must fall back to variant ident; got: {names:?}"
    );
    assert!(
        !names.contains(&""),
        "empty string must not appear as a message name; got: {names:?}"
    );

    let messages = ResponseMsg::asyncapi_messages();
    assert!(
        messages.iter().any(|m| m.name.as_deref() == Some("Empty")),
        "Empty message must be findable by variant ident"
    );
    assert!(
        !messages.iter().any(|m| m.name.as_deref() == Some("")),
        "empty-string message name must not appear"
    );
}

// Regression for #8: cross-enum discriminant collision resolved via message_name override
#[test]
fn test_cross_enum_collision_resolved_with_message_name() {
    // Two enums with different roles but the same variant identifier / serde rename.
    // The response variant uses #[asyncapi(message_name = "...")] to disambiguate.
    #[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
    #[serde(tag = "message")]
    pub enum Operation {
        #[serde(rename = "get-info")]
        GetInfo { project_id: i64 },
    }

    #[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
    #[serde(tag = "message")]
    pub enum OperationResponse {
        #[serde(rename = "get-info")]
        #[asyncapi(message_name = "GetInfoResponse")]
        GetInfo { id: i64, label: String },
    }

    #[derive(AsyncApi)]
    #[asyncapi(title = "Collision Test", version = "1.0.0")]
    #[asyncapi_messages(Operation, OperationResponse)]
    struct CollisionApi;

    let spec = CollisionApi::asyncapi_spec();
    let messages = spec
        .components
        .as_ref()
        .and_then(|c| c.messages.as_ref())
        .expect("components.messages must be present");

    // Both messages must be present under distinct keys
    assert!(
        messages.contains_key("GetInfo"),
        "GetInfo (request) must be in components.messages"
    );
    assert!(
        messages.contains_key("GetInfoResponse"),
        "GetInfoResponse must be in components.messages"
    );
    assert_eq!(
        messages.len(),
        2,
        "both messages must survive — no silent overwrite"
    );

    // Payloads must be distinct (request has project_id, response has label)
    let req_json = serde_json::to_value(&messages["GetInfo"].payload).unwrap();
    let res_json = serde_json::to_value(&messages["GetInfoResponse"].payload).unwrap();
    assert!(
        req_json.pointer("/properties/project_id").is_some(),
        "GetInfo payload must have project_id"
    );
    assert!(
        res_json.pointer("/properties/label").is_some(),
        "GetInfoResponse payload must have label"
    );

    // Wire discriminant in both payloads must still be "get-info"
    assert_eq!(
        req_json
            .pointer("/properties/message/const")
            .and_then(|v| v.as_str()),
        Some("get-info"),
        "GetInfo discriminant must be 'get-info'"
    );
    assert_eq!(
        res_json
            .pointer("/properties/message/const")
            .and_then(|v| v.as_str()),
        Some("get-info"),
        "GetInfoResponse discriminant must be 'get-info'"
    );
}

// Regression for #8: message_name attribute is honoured for naming in components.messages
#[test]
fn test_message_name_override_attribute() {
    #[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
    #[serde(tag = "type")]
    pub enum Event {
        #[serde(rename = "editor-update")]
        #[asyncapi(message_name = "EditorUpdate", summary = "Editor state changed")]
        Update { content: String },
    }

    let names = Event::asyncapi_message_names();
    assert_eq!(names, vec!["EditorUpdate"]);

    let messages = Event::asyncapi_messages();
    let msg = &messages[0];
    assert_eq!(msg.name.as_deref(), Some("EditorUpdate"));
    assert_eq!(msg.summary.as_deref(), Some("Editor state changed"));

    // Wire discriminant in the payload must still be "editor-update"
    let payload_json = serde_json::to_value(&msg.payload).unwrap();
    assert_eq!(
        payload_json
            .pointer("/properties/type/const")
            .and_then(|v| v.as_str()),
        Some("editor-update")
    );
}
