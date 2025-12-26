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
    assert_eq!(names, vec!["user.join", "user.leave", "chat.message"]);
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

    // Find messages by name (more maintainable than hard-coded indices)
    let echo_msg = messages
        .iter()
        .find(|m| m.name.as_ref().is_some_and(|n| n == "Echo"))
        .expect("Echo message should exist");

    let broadcast_msg = messages
        .iter()
        .find(|m| m.name.as_ref().is_some_and(|n| n == "Broadcast"))
        .expect("Broadcast message should exist");

    // Both should have schemas
    assert!(echo_msg.payload.is_some());
    assert!(broadcast_msg.payload.is_some());

    // Verify that each message has ONLY its specific variant schema (no oneOf)
    // Convert to JSON to inspect the structure
    let echo_payload_json = serde_json::to_value(&echo_msg.payload).unwrap();
    let broadcast_payload_json = serde_json::to_value(&broadcast_msg.payload).unwrap();

    // Echo message should NOT have oneOf at the top level
    assert!(
        echo_payload_json.get("oneOf").is_none(),
        "Echo message should not have oneOf in its schema"
    );

    // Broadcast message should NOT have oneOf at the top level
    assert!(
        broadcast_payload_json.get("oneOf").is_none(),
        "Broadcast message should not have oneOf in its schema"
    );

    // Echo message should have the const value "Echo" in its type property
    if let Some(properties) = echo_payload_json.get("properties") {
        if let Some(type_prop) = properties.get("type") {
            assert_eq!(
                type_prop.get("const"),
                Some(&serde_json::json!("Echo")),
                "Echo message should have const 'Echo' in type property"
            );
        }
    }

    // Broadcast message should have the const value "Broadcast" in its type property
    if let Some(properties) = broadcast_payload_json.get("properties") {
        if let Some(type_prop) = properties.get("type") {
            assert_eq!(
                type_prop.get("const"),
                Some(&serde_json::json!("Broadcast")),
                "Broadcast message should have const 'Broadcast' in type property"
            );
        }
    }
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

    // Verify we have all 3 messages (2 from ApiMessage, 1 from SystemMessage)
    assert_eq!(messages.len(), 3);

    // Verify user.join message
    let user_join = messages
        .get("user.join")
        .expect("Should have user.join message");
    assert_eq!(user_join.name, Some("user.join".to_string()));
    assert_eq!(user_join.summary, Some("User joins".to_string()));
    assert_eq!(
        user_join.description,
        Some("User enters a room".to_string())
    );
    assert!(user_join.payload.is_some());

    // Verify user.leave message
    let user_leave = messages
        .get("user.leave")
        .expect("Should have user.leave message");
    assert_eq!(user_leave.name, Some("user.leave".to_string()));
    assert_eq!(user_leave.summary, Some("User leaves".to_string()));

    // Verify system.status message
    let system_status = messages
        .get("system.status")
        .expect("Should have system.status message");
    assert_eq!(system_status.name, Some("system.status".to_string()));
    assert_eq!(system_status.summary, Some("System status".to_string()));
}

#[test]
fn test_asyncapi_operation_with_messages() {
    // Define message types for operations
    #[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
    #[serde(tag = "type")]
    pub enum ChatMessage {
        #[serde(rename = "user.join")]
        UserJoin { username: String, room: String },
        #[serde(rename = "chat.message")]
        ChatMessage { username: String, text: String },
    }

    #[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
    #[serde(tag = "type")]
    pub enum SystemMessage {
        #[serde(rename = "system.status")]
        Status { status: String },
    }

    // Define API with operations that specify messages
    #[allow(clippy::duplicated_attributes)]
    #[derive(AsyncApi)]
    #[asyncapi(title = "Chat API", version = "1.0.0")]
    #[asyncapi_channel(name = "chat", address = "/ws/chat")]
    #[asyncapi_operation(name = "sendMessage", action = "send", channel = "chat", messages = [ChatMessage])]
    #[asyncapi_operation(name = "receiveMessage", action = "receive", channel = "chat", messages = [ChatMessage, SystemMessage])]
    #[asyncapi_messages(ChatMessage, SystemMessage)]
    struct ChatApi;

    let spec = ChatApi::asyncapi_spec();

    // Verify operations exist
    let operations = spec.operations.expect("Should have operations");
    assert_eq!(operations.len(), 2);

    // Verify sendMessage operation has messages
    let send_op = operations
        .get("sendMessage")
        .expect("Should have sendMessage operation");
    assert!(send_op.messages.is_some());
    let send_messages = send_op.messages.as_ref().unwrap();
    assert_eq!(send_messages.len(), 2); // ChatMessage has 2 variants

    // Verify receiveMessage operation has messages
    let receive_op = operations
        .get("receiveMessage")
        .expect("Should have receiveMessage operation");
    assert!(receive_op.messages.is_some());
    let receive_messages = receive_op.messages.as_ref().unwrap();
    assert_eq!(receive_messages.len(), 3); // ChatMessage (2 variants) + SystemMessage (1 variant)

    // Verify operation message references point to channel messages (not components directly)
    match &send_messages[0] {
        asyncapi_rust::MessageRef::Reference { reference } => {
            assert!(
                reference == "#/channels/chat/messages/user.join"
                    || reference == "#/channels/chat/messages/chat.message"
            );
        }
        _ => panic!("Expected message reference"),
    }

    // Verify channels exist and have messages
    let channels = spec.channels.expect("Should have channels");
    assert_eq!(channels.len(), 1);

    let chat_channel = channels.get("chat").expect("Should have chat channel");
    assert!(chat_channel.messages.is_some());
    let channel_messages = chat_channel.messages.as_ref().unwrap();
    assert_eq!(channel_messages.len(), 3); // All unique messages from both operations

    // Verify channel messages reference components directly
    assert!(channel_messages.contains_key("user.join"));
    assert!(channel_messages.contains_key("chat.message"));
    assert!(channel_messages.contains_key("system.status"));

    // Verify channel messages reference components (not other channels)
    match channel_messages.get("user.join").unwrap() {
        asyncapi_rust::MessageRef::Reference { reference } => {
            assert_eq!(reference, "#/components/messages/user.join");
        }
        _ => panic!("Expected message reference"),
    }
}
