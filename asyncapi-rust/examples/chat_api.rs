//! Complete example: Chat WebSocket API with AsyncAPI spec generation
//!
//! This example demonstrates:
//! - Defining WebSocket message types with serde
//! - Using ToAsyncApiMessage derive macro
//! - Generating JSON schemas automatically
//! - Building a complete AsyncAPI 3.0 specification
//! - Serializing to JSON for documentation

use asyncapi_rust::{
    AsyncApiSpec, Channel, Components, Info, Message, Operation, OperationAction, Server,
    ToAsyncApiMessage, schemars::JsonSchema,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WebSocket messages for a chat application
///
/// The `tag = "type"` attribute means messages are tagged with a "type" field
/// in JSON, allowing the discriminator to identify which variant is used.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum ChatMessage {
    /// User joins a chat room
    #[serde(rename = "user.join")]
    UserJoin {
        /// Username of the joining user
        username: String,
        /// Room to join
        room: String,
    },

    /// User leaves a chat room
    #[serde(rename = "user.leave")]
    UserLeave {
        /// Username of the leaving user
        username: String,
        /// Room being left
        room: String,
    },

    /// Send a chat message
    #[serde(rename = "chat.message")]
    Chat {
        /// Username of sender
        username: String,
        /// Target room
        room: String,
        /// Message text
        text: String,
        /// Unix timestamp
        timestamp: u64,
    },

    /// User is typing indicator
    #[serde(rename = "user.typing")]
    Typing {
        /// Username of typing user
        username: String,
        /// Room where user is typing
        room: String,
    },
}

fn main() {
    println!("🚀 Generating AsyncAPI specification for Chat WebSocket API\n");

    // Get message metadata
    println!("📋 Message types discovered:");
    for (i, name) in ChatMessage::asyncapi_message_names().iter().enumerate() {
        println!("  {}. {}", i + 1, name);
    }
    println!();

    // Generate messages with schemas
    println!("🔧 Generating JSON schemas...");
    let messages = ChatMessage::asyncapi_messages();
    println!("   Generated {} message schemas\n", messages.len());

    // Build the complete AsyncAPI spec
    let spec = build_asyncapi_spec(messages);

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&spec).expect("Failed to serialize spec");

    println!("📄 Complete AsyncAPI 3.0 Specification:\n");
    println!("{}", json);
    println!();

    // Show some stats
    println!("📊 Statistics:");
    println!("   Version: {}", spec.asyncapi);
    println!("   Title: {}", spec.info.title);
    println!("   API Version: {}", spec.info.version);
    if let Some(channels) = &spec.channels {
        println!("   Channels: {}", channels.len());
    }
    if let Some(operations) = &spec.operations {
        println!("   Operations: {}", operations.len());
    }
    if let Some(components) = &spec.components {
        if let Some(msgs) = &components.messages {
            println!("   Messages: {}", msgs.len());
        }
    }
}

/// Build a complete AsyncAPI specification with server, channels, and operations
fn build_asyncapi_spec(messages: Vec<Message>) -> AsyncApiSpec {
    // Define server
    let mut servers = HashMap::new();
    servers.insert(
        "production".to_string(),
        Server {
            host: "api.example.com".to_string(),
            protocol: "wss".to_string(),
            description: Some("Production WebSocket server".to_string()),
        },
    );

    // Define channel
    let mut channels = HashMap::new();
    channels.insert(
        "chat".to_string(),
        Channel {
            address: Some("/ws/chat".to_string()),
            messages: None, // Messages defined in components
        },
    );

    // Define operations (send and receive)
    let mut operations = HashMap::new();

    operations.insert(
        "sendMessage".to_string(),
        Operation {
            action: OperationAction::Send,
            channel: asyncapi_rust::ChannelRef {
                reference: "#/channels/chat".to_string(),
            },
            messages: Some(
                messages
                    .iter()
                    .enumerate()
                    .map(|(i, msg)| asyncapi_rust::MessageRef::Reference {
                        reference: format!(
                            "#/components/messages/{}",
                            msg.name.as_ref().unwrap_or(&format!("message_{}", i))
                        ),
                    })
                    .collect(),
            ),
        },
    );

    operations.insert(
        "receiveMessage".to_string(),
        Operation {
            action: OperationAction::Receive,
            channel: asyncapi_rust::ChannelRef {
                reference: "#/channels/chat".to_string(),
            },
            messages: Some(
                messages
                    .iter()
                    .enumerate()
                    .map(|(i, msg)| asyncapi_rust::MessageRef::Reference {
                        reference: format!(
                            "#/components/messages/{}",
                            msg.name.as_ref().unwrap_or(&format!("message_{}", i))
                        ),
                    })
                    .collect(),
            ),
        },
    );

    // Define components with messages
    let mut component_messages = HashMap::new();
    for message in messages {
        if let Some(name) = &message.name {
            component_messages.insert(name.clone(), message);
        }
    }

    let components = Components {
        messages: Some(component_messages),
        schemas: None,
    };

    // Build the complete spec
    AsyncApiSpec {
        asyncapi: "3.0.0".to_string(),
        info: Info {
            title: "Chat WebSocket API".to_string(),
            version: "1.0.0".to_string(),
            description: Some(
                "Real-time chat application using WebSocket for bidirectional communication"
                    .to_string(),
            ),
        },
        servers: Some(servers),
        channels: Some(channels),
        operations: Some(operations),
        components: Some(components),
    }
}
