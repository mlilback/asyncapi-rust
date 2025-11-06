//! Example: Complete AsyncAPI spec with servers, channels, and operations
//!
//! This example demonstrates the full capabilities of the AsyncApi derive macro,
//! including servers, channels, and operations attributes.

use asyncapi_rust::AsyncApi;

/// Complete Chat API specification
///
/// This demonstrates all supported attributes:
/// - #[asyncapi(...)] - Basic info (title, version, description)
/// - #[asyncapi_server(...)] - Server definitions
/// - #[asyncapi_channel(...)] - Channel definitions
/// - #[asyncapi_operation(...)] - Operation definitions
#[allow(clippy::duplicated_attributes)] // False positive - different operations can reference same channel
#[derive(AsyncApi)]
#[asyncapi(
    title = "Chat WebSocket API",
    version = "1.0.0",
    description = "Real-time chat application using WebSocket for bidirectional communication"
)]
#[asyncapi_server(
    name = "production",
    host = "api.example.com",
    protocol = "wss",
    description = "Production WebSocket server with TLS"
)]
#[asyncapi_server(
    name = "development",
    host = "localhost:8080",
    protocol = "ws"
)]
#[asyncapi_channel(name = "chat", address = "/ws/chat")]
#[asyncapi_channel(name = "notifications", address = "/ws/notifications")]
#[asyncapi_operation(name = "sendChatMessage", action = "send", channel = "chat")]
#[asyncapi_operation(name = "receiveChatMessage", action = "receive", channel = "chat")]
#[asyncapi_operation(name = "receiveNotification", action = "receive", channel = "notifications")]
struct ChatApi;

fn main() {
    println!("🚀 Complete AsyncAPI Specification Example\n");

    // Generate the spec
    let spec = ChatApi::asyncapi_spec();

    // Display Info section
    println!("📋 API Information:");
    println!("  Title: {}", spec.info.title);
    println!("  Version: {}", spec.info.version);
    if let Some(desc) = &spec.info.description {
        println!("  Description: {}", desc);
    }
    println!();

    // Display Servers
    if let Some(servers) = &spec.servers {
        println!("🖥️  Servers ({}):", servers.len());
        for (name, server) in servers {
            println!("  • {}", name);
            println!("    Host: {}", server.host);
            println!("    Protocol: {}", server.protocol);
            if let Some(desc) = &server.description {
                println!("    Description: {}", desc);
            }
        }
        println!();
    }

    // Display Channels
    if let Some(channels) = &spec.channels {
        println!("📡 Channels ({}):", channels.len());
        for (name, channel) in channels {
            println!("  • {}", name);
            if let Some(addr) = &channel.address {
                println!("    Address: {}", addr);
            }
        }
        println!();
    }

    // Display Operations
    if let Some(operations) = &spec.operations {
        println!("⚡ Operations ({}):", operations.len());
        for (name, operation) in operations {
            let action = match operation.action {
                asyncapi_rust::OperationAction::Send => "send",
                asyncapi_rust::OperationAction::Receive => "receive",
            };
            println!("  • {} ({})", name, action);
            println!("    Channel: {}", operation.channel.reference);
        }
        println!();
    }

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&spec).expect("Failed to serialize spec");

    println!("📄 Complete JSON Specification:\n");
    println!("{}", json);

    println!("\n💡 Key Features:");
    println!("   • Multiple servers (production, development)");
    println!("   • Multiple channels (chat, notifications)");
    println!("   • Multiple operations (send/receive)");
    println!("   • Automatic reference generation (#/channels/...)");
    println!("\n✨ All defined declaratively with attributes!");
}
