//! # asyncapi-rust
//!
//! Generate [AsyncAPI 3.0](https://www.asyncapi.com/docs/reference/specification/v3.0.0)
//! specifications from Rust code using procedural macros.
//!
//! Similar to how [`utoipa`](https://crates.io/crates/utoipa) generates OpenAPI specs for REST APIs,
//! `asyncapi-rust` generates AsyncAPI specs for WebSocket and other async protocols.
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! asyncapi-rust = "0.1"
//! serde = { version = "1.0", features = ["derive"] }
//! schemars = { version = "0.8", features = ["derive"] }
//! ```
//!
//! Define your WebSocket messages:
//!
//! ```rust,ignore
//! use asyncapi_rust::{AsyncApi, ToAsyncApiMessage, schemars::JsonSchema};
//! use serde::{Deserialize, Serialize};
//!
//! /// WebSocket messages for a chat application
//! #[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
//! #[serde(tag = "type")]
//! pub enum ChatMessage {
//!     /// User joins a chat room
//!     #[serde(rename = "user.join")]
//!     #[asyncapi(summary = "User joins", description = "Sent when a user enters a room")]
//!     UserJoin { username: String, room: String },
//!
//!     /// Send a chat message
//!     #[serde(rename = "chat.message")]
//!     #[asyncapi(summary = "Chat message", description = "Broadcast to all users in a room")]
//!     Chat { username: String, room: String, text: String },
//! }
//!
//! /// Complete API specification
//! #[derive(AsyncApi)]
//! #[asyncapi(title = "Chat API", version = "1.0.0")]
//! #[asyncapi_server(name = "production", host = "api.example.com", protocol = "wss")]
//! #[asyncapi_channel(name = "chat", address = "/ws/chat")]
//! #[asyncapi_operation(name = "sendMessage", action = "send", channel = "chat")]
//! #[asyncapi_operation(name = "receiveMessage", action = "receive", channel = "chat")]
//! struct ChatApi;
//!
//! fn main() {
//!     // Generate complete specification
//!     let spec = ChatApi::asyncapi_spec();
//!
//!     // Generate message schemas
//!     let messages = ChatMessage::asyncapi_messages();
//!
//!     // Serialize to JSON
//!     println!("{}", serde_json::to_string_pretty(&spec).unwrap());
//! }
//! ```
//!
//! ## Core Concepts
//!
//! ### Message Types with `#[derive(ToAsyncApiMessage)]`
//!
//! Generate message metadata and JSON schemas from your Rust types:
//!
//! - Uses [`serde`](https://serde.rs) for JSON serialization
//! - Uses [`schemars`](https://docs.rs/schemars) for JSON Schema generation
//! - Respects `#[serde(...)]` attributes (`rename`, `tag`, etc.)
//! - Supports `#[asyncapi(...)]` helper attributes for documentation
//!
//! ### Complete Specs with `#[derive(AsyncApi)]`
//!
//! Generate complete AsyncAPI specifications declaratively:
//!
//! - `#[asyncapi(...)]` - Basic info (title, version, description)
//! - `#[asyncapi_server(...)]` - Server definitions
//! - `#[asyncapi_channel(...)]` - Channel definitions
//! - `#[asyncapi_operation(...)]` - Operation definitions
//!
//! ## Framework Integration
//!
//! Works with any WebSocket framework:
//!
//! - **actix-web + actix-ws** - See `examples/actix_websocket.rs`
//! - **axum** - See `examples/axum_websocket.rs`
//! - **tungstenite** - See `examples/framework_integration_guide.rs`
//!
//! The same message types are used in both runtime handlers and documentation.
//!
//! ## Features
//!
//! - **Code-first**: Generate specs from Rust types, not YAML
//! - **Compile-time**: Zero runtime cost, all generation at build time
//! - **Type-safe**: Compile errors if documentation drifts from code
//! - **Framework agnostic**: Works with actix-ws, axum, or any serde-compatible types
//! - **Binary protocols**: Support for mixed text/binary WebSocket messages
//!
//! ## Examples
//!
//! See the `examples/` directory for complete working examples:
//!
//! - `simple.rs` - Basic message types with schema generation
//! - `chat_api.rs` - Complete AsyncAPI 3.0 specification
//! - `asyncapi_derive.rs` - Using `#[derive(AsyncApi)]`
//! - `generate_spec_file.rs` - Generating specification files
//! - `full_asyncapi_derive.rs` - Complete spec with servers, channels, operations
//! - `actix_websocket.rs` - Real-world actix-web integration
//! - `axum_websocket.rs` - Real-world axum integration
//! - `framework_integration_guide.rs` - Comprehensive framework guide
//!
//! Run any example:
//! ```bash
//! cargo run --example actix_websocket
//! ```
//!
//! ## Generating Documentation Files
//!
//! Create a binary to generate AsyncAPI spec files:
//!
//! ```rust,ignore
//! // bin/generate-asyncapi.rs
//! use my_project::MyApi;
//!
//! fn main() {
//!     let spec = MyApi::asyncapi_spec();
//!     let json = serde_json::to_string_pretty(&spec).unwrap();
//!     std::fs::write("docs/asyncapi.json", json).unwrap();
//! }
//! ```
//!
//! Then run: `cargo run --bin generate-asyncapi`
//!
//! ## Further Reading
//!
//! - [AsyncAPI Specification](https://www.asyncapi.com/docs/reference/specification/v3.0.0)
//! - [GitHub Repository](https://github.com/mlilback/asyncapi-rust)
//! - [Examples Directory](https://github.com/mlilback/asyncapi-rust/tree/main/asyncapi-rust/examples)

#![deny(missing_docs)]
#![warn(clippy::all)]

// Re-export proc macros from asyncapi-rust-codegen
pub use asyncapi_rust_codegen::{AsyncApi, ToAsyncApiMessage};

// Re-export models
pub use asyncapi_rust_models::*;

// Re-export commonly used types
pub use schemars;
pub use serde::{Deserialize, Serialize};
pub use serde_json;

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_import() {
        // Verify exports are accessible
        // Actual functionality tests will be in integration tests
    }
}
