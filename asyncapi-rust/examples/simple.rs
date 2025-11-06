//! Simple example: Basic message type with schema generation
//!
//! This is the minimal example showing how to use asyncapi-rust.

use asyncapi_rust::{ToAsyncApiMessage, schemars::JsonSchema};
use serde::{Deserialize, Serialize};

/// Simple WebSocket message types
#[derive(Debug, Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum Message {
    /// Ping message
    Ping { id: u64 },
    /// Pong response
    Pong { id: u64, timestamp: u64 },
}

fn main() {
    println!("Simple AsyncAPI Example\n");

    // Get message names
    println!("Message types:");
    for name in Message::asyncapi_message_names() {
        println!("  - {}", name);
    }
    println!();

    // Generate messages with schemas
    let messages = Message::asyncapi_messages();

    println!("Generated {} messages with schemas:\n", messages.len());

    for msg in &messages {
        println!("Message: {}", msg.name.as_ref().unwrap_or(&String::new()));
        println!(
            "  Content-Type: {}",
            msg.content_type.as_ref().unwrap_or(&String::new())
        );
        println!("  Has Schema: {}", msg.payload.is_some());
        println!();
    }

    // Serialize first message to JSON
    if let Some(first) = messages.first() {
        let json = serde_json::to_string_pretty(&first).expect("Serialize failed");
        println!("First message as JSON:\n{}", json);
    }
}
