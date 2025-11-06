use asyncapi_rust::{ToAsyncApiMessage, schemars::JsonSchema};
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
