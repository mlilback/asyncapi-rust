//! # asyncapi-rust
//!
//! Generate AsyncAPI 3.0 specifications from Rust code using procedural macros.
//!
//! Similar to how `utoipa` generates OpenAPI specs for REST APIs, `asyncapi-rust`
//! generates AsyncAPI specs for WebSocket and other async protocols.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use asyncapi_rust::{AsyncApi, ToAsyncApiMessage};
//! use serde::{Deserialize, Serialize};
//!
//! /// WebSocket messages
//! #[derive(Serialize, Deserialize, ToAsyncApiMessage)]
//! #[serde(tag = "type")]
//! pub enum Message {
//!     /// Send a chat message
//!     Chat { room: String, text: String },
//! }
//!
//! /// Generate the AsyncAPI spec
//! #[derive(AsyncApi)]
//! #[asyncapi(
//!     title = "Chat API",
//!     version = "1.0.0",
//! )]
//! struct ChatApi;
//!
//! fn main() {
//!     let spec = ChatApi::asyncapi();
//!     println!("{}", serde_json::to_string_pretty(&spec).unwrap());
//! }
//! ```
//!
//! ## Features
//!
//! - **Code-first**: Generate specs from Rust types
//! - **Compile-time**: Zero runtime cost
//! - **Type-safe**: Errors if docs drift from code
//! - **Framework agnostic**: Works with actix-ws, axum, or any serde types
//! - **Binary protocols**: Support for mixed text/binary messages

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
