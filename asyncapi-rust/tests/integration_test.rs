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

    // Each variant should have its own message
    assert_eq!(messages[0].name, Some("Echo".to_string()));
    assert_eq!(messages[1].name, Some("Broadcast".to_string()));

    // Both should have schemas
    assert!(messages[0].payload.is_some());
    assert!(messages[1].payload.is_some());
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
