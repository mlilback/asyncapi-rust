//! Server Variables and Channel Parameters Example
//!
//! Demonstrates how to define dynamic server paths and typed channel parameters
//! for user-specific WebSocket connections.
//!
//! This example shows:
//! - Server variables with pathname, examples, and enum values
//! - Channel parameters with JSON Schema types and formats
//! - Multiple variables and parameters
//! - Complete AsyncAPI 3.0 spec generation
//!
//! Run with: cargo run --example server_variables

use asyncapi_rust::{AsyncApi, ToAsyncApiMessage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// WebSocket messages for a user-specific real-time messaging system
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum UserMessage {
    /// Subscribe to a specific data stream
    #[serde(rename = "subscribe")]
    #[asyncapi(
        summary = "Subscribe to updates",
        description = "Client subscribes to receive real-time updates for a specific resource"
    )]
    Subscribe {
        /// Resource ID to subscribe to
        resource_id: String,
        /// Optional filter criteria
        filters: Option<Vec<String>>,
    },

    /// Real-time update notification
    #[serde(rename = "update")]
    #[asyncapi(
        summary = "Update notification",
        description = "Server sends an update when the subscribed resource changes"
    )]
    Update {
        /// Resource ID that was updated
        resource_id: String,
        /// Update timestamp
        timestamp: i64,
        /// Updated data payload
        data: serde_json::Value,
    },

    /// Unsubscribe from a data stream
    #[serde(rename = "unsubscribe")]
    #[asyncapi(summary = "Unsubscribe from updates")]
    Unsubscribe {
        /// Resource ID to unsubscribe from
        resource_id: String,
    },
}

/// Complete API specification with server variables and channel parameters
#[derive(AsyncApi)]
#[asyncapi(
    title = "User WebSocket API",
    version = "1.0.0",
    description = "Real-time messaging API with user-specific WebSocket connections"
)]
#[asyncapi_server(
    name = "production",
    host = "api.example.com",
    protocol = "wss",
    pathname = "/api/{version}/ws/{userId}",
    description = "Production WebSocket server with user-specific connections",
    variable(
        name = "version",
        description = "API version",
        enum_values = ["v1", "v2"],
        default = "v2",
        examples = ["v1", "v2"]
    ),
    variable(
        name = "userId",
        description = "Authenticated user ID",
        examples = ["12", "13", "100"]
    )
)]
#[asyncapi_server(
    name = "staging",
    host = "staging.example.com",
    protocol = "wss",
    pathname = "/api/{version}/ws/{userId}",
    description = "Staging environment for testing",
    variable(
        name = "version",
        description = "API version",
        enum_values = ["v1", "v2"],
        default = "v2"
    ),
    variable(
        name = "userId",
        description = "User ID for testing",
        examples = ["test-user-1", "test-user-2"]
    )
)]
#[asyncapi_channel(
    name = "userMessaging",
    address = "/api/{version}/ws/{userId}",
    parameter(
        name = "version",
        description = "API version number",
        schema_type = "string"
    ),
    parameter(
        name = "userId",
        description = "Unique identifier for the authenticated user",
        schema_type = "integer",
        format = "int64"
    )
)]
#[asyncapi_operation(
    name = "subscribeToUpdates",
    action = "send",
    channel = "userMessaging"
)]
#[asyncapi_operation(
    name = "receiveUpdates",
    action = "receive",
    channel = "userMessaging"
)]
#[asyncapi_messages(UserMessage)]
struct UserApi;

fn main() {
    println!("=== Server Variables and Channel Parameters Example ===\n");

    // Generate the complete AsyncAPI specification
    let spec = UserApi::asyncapi_spec();

    // Display server information
    println!("🌐 Servers:");
    if let Some(servers) = &spec.servers {
        for (name, server) in servers {
            println!("  • {}", name);
            println!("    Host: {}", server.host);
            println!("    Protocol: {}", server.protocol);

            if let Some(pathname) = &server.pathname {
                println!("    Path: {}", pathname);
            }

            if let Some(variables) = &server.variables {
                println!("    Variables:");
                for (var_name, var) in variables {
                    print!("      - {}", var_name);
                    if let Some(desc) = &var.description {
                        print!(" ({})", desc);
                    }
                    println!();

                    if let Some(default) = &var.default {
                        println!("        Default: {}", default);
                    }
                    if let Some(enum_vals) = &var.enum_values {
                        println!("        Allowed: {:?}", enum_vals);
                    }
                    if let Some(examples) = &var.examples {
                        println!("        Examples: {:?}", examples);
                    }
                }
            }
            println!();
        }
    }

    // Display channel information
    println!("📡 Channels:");
    if let Some(channels) = &spec.channels {
        for (name, channel) in channels {
            println!("  • {}", name);
            if let Some(address) = &channel.address {
                println!("    Address: {}", address);
            }

            if let Some(parameters) = &channel.parameters {
                println!("    Parameters:");
                for (param_name, param) in parameters {
                    print!("      - {}", param_name);
                    if let Some(desc) = &param.description {
                        print!(" ({})", desc);
                    }
                    println!();

                    if let Some(schema) = &param.schema {
                        println!("        Schema: {:?}", schema);
                    }
                }
            }
            println!();
        }
    }

    // Display operations
    println!("⚡ Operations:");
    if let Some(operations) = &spec.operations {
        for (name, operation) in operations {
            let action_str = match operation.action {
                asyncapi_rust::OperationAction::Send => "send",
                asyncapi_rust::OperationAction::Receive => "receive",
            };
            println!("  • {} ({})", name, action_str);
            println!("    Channel: {}", operation.channel.reference);
            println!();
        }
    }

    // Display messages
    println!("📨 Messages:");
    if let Some(components) = &spec.components {
        if let Some(messages) = &components.messages {
            for (name, message) in messages {
                println!("  • {}", name);
                if let Some(summary) = &message.summary {
                    println!("    Summary: {}", summary);
                }
                if let Some(content_type) = &message.content_type {
                    println!("    Content-Type: {}", content_type);
                }
                println!();
            }
        }
    }

    // Generate and save the complete spec
    println!("\n=== Complete AsyncAPI Specification (JSON) ===\n");
    let json = serde_json::to_string_pretty(&spec).unwrap();
    println!("{}", json);

    println!("\n✅ Specification generated successfully!");
    println!("\nKey Features Demonstrated:");
    println!("  • Server variables for dynamic paths (version, userId)");
    println!("  • Multiple servers (production, staging)");
    println!("  • Channel parameters with JSON Schema types");
    println!("  • Typed parameters (string for version, integer/int64 for userId)");
    println!("  • Complete spec with servers, channels, operations, and messages");
}
