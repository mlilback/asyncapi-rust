//! Tagging servers, channels, operations, and messages
//!
//! Demonstrates `#[asyncapi_tag(...)]` and the `tags = [...]` references that
//! point at it. Each declaration becomes one entry in `components.tags`; every
//! use site emits a `$ref` to that entry rather than repeating the definition.
//!
//! Run with: `cargo run --example tags`

use asyncapi_rust::{AsyncApi, ToAsyncApiMessage, schemars::JsonSchema};
use serde::{Deserialize, Serialize};

/// Messages exchanged on the chat channel
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum ChatMessage {
    /// A user sends a message to a room
    #[serde(rename = "chat.message")]
    #[asyncapi(summary = "Send a chat message", tags = ["chat"])]
    Chat {
        /// Target room
        room: String,
        /// Message text
        text: String,
    },

    /// An operator forcibly closes a room
    #[serde(rename = "room.close")]
    #[asyncapi(summary = "Close a room", tags = ["chat", "admin"])]
    CloseRoom {
        /// Room to close
        room: String,
    },
}

#[derive(AsyncApi)]
#[asyncapi(
    title = "Tagged Chat API",
    version = "1.0.0",
    description = "Demonstrates reusable tags",
    tags = ["public"]
)]
#[asyncapi_tag(name = "public", description = "Stable, publicly documented surface")]
#[asyncapi_tag(name = "chat", description = "Chat room messaging")]
#[asyncapi_tag(name = "admin", description = "Requires elevated privileges")]
#[asyncapi_server(
    name = "production",
    host = "api.example.com",
    protocol = "wss",
    description = "Production WebSocket server",
    tags = ["public"]
)]
#[asyncapi_channel(
    name = "chat",
    address = "/ws/chat",
    description = "Chat room traffic",
    tags = ["chat"]
)]
#[asyncapi_operation(
    name = "sendChatMessage",
    action = "send",
    channel = "chat",
    description = "Send a message to a room",
    messages = [ChatMessage],
    tags = ["chat", "public"]
)]
pub struct TaggedChatApi;

fn main() {
    let spec = TaggedChatApi::asyncapi_spec();
    println!("{}", serde_json::to_string_pretty(&spec).unwrap());

    // Tags are defined once and referenced everywhere else.
    let components = spec.components.as_ref().expect("components");
    let tags = components.tags.as_ref().expect("components.tags");
    assert_eq!(tags.len(), 3);
    assert!(tags.contains_key("admin"));
}
