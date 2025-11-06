//! Example: Using #[derive(AsyncApi)] to generate specifications
//!
//! This example demonstrates the AsyncApi derive macro for generating
//! basic AsyncAPI specifications with the Info section pre-filled.

use asyncapi_rust::{AsyncApi, AsyncApiSpec};
use serde_json;

/// Chat WebSocket API specification
///
/// The AsyncApi derive macro generates a `asyncapi_spec()` method that
/// returns a basic AsyncApiSpec with the Info section populated from
/// the #[asyncapi(...)] attributes.
#[derive(AsyncApi)]
#[asyncapi(
    title = "Chat WebSocket API",
    version = "1.0.0",
    description = "Real-time chat application using WebSocket for bidirectional communication"
)]
struct ChatApi;

/// Minimal API specification without description
#[derive(AsyncApi)]
#[asyncapi(title = "Minimal API", version = "0.1.0")]
struct MinimalApi;

fn main() {
    println!("🚀 AsyncApi Derive Macro Example\n");

    // Example 1: Full specification with description
    println!("📋 Example 1: Chat API with description");
    let chat_spec: AsyncApiSpec = ChatApi::asyncapi_spec();

    println!("  Title: {}", chat_spec.info.title);
    println!("  Version: {}", chat_spec.info.version);
    if let Some(desc) = &chat_spec.info.description {
        println!("  Description: {}", desc);
    }
    println!();

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&chat_spec).expect("Failed to serialize");
    println!("  JSON Output:");
    println!("{}", json);
    println!();

    // Example 2: Minimal specification
    println!("📋 Example 2: Minimal API without description");
    let minimal_spec = MinimalApi::asyncapi_spec();

    println!("  Title: {}", minimal_spec.info.title);
    println!("  Version: {}", minimal_spec.info.version);
    println!("  Description: {:?}", minimal_spec.info.description);
    println!();

    // Example 3: Building on the generated spec
    println!("💡 Tip: You can extend the generated spec programmatically");
    println!("   let mut spec = ChatApi::asyncapi_spec();");
    println!("   spec.servers = Some(my_servers);");
    println!("   spec.channels = Some(my_channels);");
    println!("   // ... add operations, components, etc.");
}
