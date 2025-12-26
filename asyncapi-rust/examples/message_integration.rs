//! Example demonstrating message integration with AsyncApi
//!
//! This example shows how to:
//! 1. Define message types using #[derive(ToAsyncApiMessage)]
//! 2. Reference those message types in #[derive(AsyncApi)]
//! 3. Automatically populate the components/messages section of your spec
//!
//! Run with: cargo run --example message_integration

use asyncapi_rust::{AsyncApi, ToAsyncApiMessage, schemars::JsonSchema};
use serde::{Deserialize, Serialize};

/// Chat messages for a chat application
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum ChatMessage {
    /// User joins a chat room
    #[serde(rename = "user.join")]
    #[asyncapi(
        summary = "User joins",
        description = "Sent when a user enters a chat room"
    )]
    UserJoin { username: String, room: String },

    /// User sends a chat message
    #[serde(rename = "chat.message")]
    #[asyncapi(
        summary = "Chat message",
        description = "A message from a user in a room"
    )]
    ChatMessage {
        username: String,
        room: String,
        message: String,
    },

    /// User leaves a chat room
    #[serde(rename = "user.leave")]
    #[asyncapi(summary = "User leaves", description = "Sent when a user exits a room")]
    UserLeave { username: String, room: String },
}

/// System messages for status and errors
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum SystemMessage {
    /// Server status update
    #[serde(rename = "system.status")]
    #[asyncapi(summary = "System status", description = "Server status update")]
    Status { status: String, timestamp: u64 },

    /// Error message
    #[serde(rename = "system.error")]
    #[asyncapi(summary = "System error", description = "Error occurred on the server")]
    Error { code: String, message: String },
}

/// Complete Chat API specification with message integration
///
/// The #[asyncapi_messages(...)] attribute automatically includes
/// message definitions from the specified types in the components section.
/// The messages parameter in operations specifies which messages can be used,
/// and these messages are automatically added to the channel that the operation references.
///
/// Per AsyncAPI 3.0 spec:
/// - Operations reference channel messages: #/channels/{channel}/messages/{message}
/// - Channels reference component messages: #/components/messages/{message}
#[allow(clippy::duplicated_attributes)]
#[derive(AsyncApi)]
#[asyncapi(
    title = "Chat API with Message Integration",
    version = "1.0.0",
    description = "Real-time chat API demonstrating automatic message integration"
)]
#[asyncapi_server(
    name = "production",
    host = "chat.example.com",
    protocol = "wss",
    description = "Production WebSocket server"
)]
#[asyncapi_channel(name = "chat", address = "/ws/chat")]
#[asyncapi_operation(name = "sendMessage", action = "send", channel = "chat", messages = [ChatMessage])]
#[asyncapi_operation(name = "receiveMessage", action = "receive", channel = "chat", messages = [ChatMessage, SystemMessage])]
#[asyncapi_messages(ChatMessage, SystemMessage)]
struct ChatApi;

fn main() {
    println!("=== Message Integration Example ===\n");

    // Generate the complete spec with messages automatically included
    let spec = ChatApi::asyncapi_spec();

    // Display basic info
    println!("API: {} v{}", spec.info.title, spec.info.version);
    if let Some(desc) = &spec.info.description {
        println!("Description: {}", desc);
    }
    println!();

    // Display servers
    if let Some(servers) = &spec.servers {
        println!("Servers:");
        for (name, server) in servers {
            println!("  - {} ({}://{})", name, server.protocol, server.host);
        }
        println!();
    }

    // Display channels
    if let Some(channels) = &spec.channels {
        println!("Channels:");
        for (name, channel) in channels {
            if let Some(address) = &channel.address {
                println!("  - {}: {}", name, address);
            }
            if let Some(messages) = &channel.messages {
                println!("    Channel Messages:");
                for (msg_name, msg_ref) in messages {
                    if let asyncapi_rust::MessageRef::Reference { reference } = msg_ref {
                        println!("      - {}: {}", msg_name, reference);
                    }
                }
            }
        }
        println!();
    }

    // Display operations
    if let Some(operations) = &spec.operations {
        println!("Operations:");
        for (name, operation) in operations {
            let action = match operation.action {
                asyncapi_rust::OperationAction::Send => "send",
                asyncapi_rust::OperationAction::Receive => "receive",
            };
            println!(
                "  - {}: {} to {}",
                name, action, operation.channel.reference
            );
            if let Some(messages) = &operation.messages {
                println!("    Messages:");
                for msg in messages {
                    if let asyncapi_rust::MessageRef::Reference { reference } = msg {
                        println!("      - {}", reference);
                    }
                }
            }
        }
        println!();
    }

    // Display messages (automatically populated from ChatMessage and SystemMessage)
    if let Some(components) = &spec.components {
        if let Some(messages) = &components.messages {
            println!("Messages (automatically included from message types):");
            for (name, message) in messages {
                println!("  - {}", name);
                if let Some(summary) = &message.summary {
                    println!("    Summary: {}", summary);
                }
                if let Some(desc) = &message.description {
                    println!("    Description: {}", desc);
                }
            }
            println!();
        }
    }

    // Serialize to JSON
    println!("=== Complete AsyncAPI Specification (JSON) ===\n");
    let json = serde_json::to_string_pretty(&spec).expect("Failed to serialize spec");
    println!("{}", json);
}
