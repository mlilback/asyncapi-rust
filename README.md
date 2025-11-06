# asyncapi-rust

> **⚠️ Early Development Notice**
>
> This project is in the early stages of development. Documentation, examples, and some links may be incomplete or broken. The API is not yet stable and may change significantly. Stay tuned for updates!

[![Crates.io](https://img.shields.io/crates/v/asyncapi-rust.svg)](https://crates.io/crates/asyncapi-rust)
[![Documentation](https://docs.rs/asyncapi-rust/badge.svg)](https://docs.rs/asyncapi-rust)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

**AsyncAPI 3.0 specification generation for Rust WebSockets and async protocols**

Generate AsyncAPI documentation directly from your Rust code using procedural macros. Similar to how `utoipa` generates OpenAPI specs for REST APIs, `asyncapi-rust` generates AsyncAPI specs for WebSocket and other async protocols.

## Features

- 🦀 **Code-first**: Generate specs from Rust types, not YAML
- ⚡ **Compile-time**: Zero runtime cost, all generation at build time
- 🔒 **Type-safe**: Compile errors if documentation drifts from code
- 🎯 **Familiar**: Follows patterns from [`utoipa`](https://crates.io/crates/utoipa), [`serde`](https://serde.rs), and [`clap`](https://crates.io/crates/clap)
- 🌐 **Framework agnostic**: Works with actix-ws, axum, or any serde-compatible types
- 📦 **Binary protocols**: Support for mixed text/binary WebSocket messages (Arrow IPC, Protobuf, etc.)

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
asyncapi-rust = "0.1"
serde = { version = "1.0", features = ["derive"] }
schemars = { version = "0.8", features = ["derive"] }
```

Define your WebSocket messages:

```rust
use asyncapi_rust::{schemars::JsonSchema, ToAsyncApiMessage};
use serde::{Deserialize, Serialize};

/// WebSocket messages for a chat application
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum ChatMessage {
    /// User joins a chat room
    #[serde(rename = "user.join")]
    UserJoin { username: String, room: String },

    /// Send a chat message
    #[serde(rename = "chat.message")]
    Chat { username: String, room: String, text: String },
}

fn main() {
    // Get message names
    let names = ChatMessage::asyncapi_message_names();
    println!("Messages: {:?}", names); // ["user.join", "chat.message"]

    // Generate messages with JSON schemas
    let messages = ChatMessage::asyncapi_messages();

    // Each message includes:
    // - name and title
    // - contentType: "application/json"
    // - payload: Full JSON Schema from schemars

    let json = serde_json::to_string_pretty(&messages).unwrap();
    println!("{}", json);
}
```

### Message Integration

Combine message types into complete specifications using `#[asyncapi_messages(...)]`:

```rust
use asyncapi_rust::{AsyncApi, ToAsyncApiMessage, schemars::JsonSchema};
use serde::{Deserialize, Serialize};

// Define your message types
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum ChatMessage {
    #[serde(rename = "user.join")]
    UserJoin { username: String, room: String },

    #[serde(rename = "chat.message")]
    Chat { username: String, text: String },
}

// Reference message types in your API spec
#[derive(AsyncApi)]
#[asyncapi(title = "Chat API", version = "1.0.0")]
#[asyncapi_messages(ChatMessage)]  // Automatically includes all messages
struct ChatApi;

fn main() {
    let spec = ChatApi::asyncapi_spec();
    // spec.components.messages now contains all ChatMessage variants
    // with full JSON schemas
}
```

The `#[asyncapi_messages(...)]` attribute automatically populates the `components/messages` section with:
- All message definitions from referenced types
- Complete JSON schemas generated from Rust types
- Message metadata (name, summary, description, content-type)

## Examples

See working examples in the `examples/` directory:

- **`simple.rs`** - Basic message types with schema generation
  ```bash
  cargo run --example simple
  ```

- **`chat_api.rs`** - Complete AsyncAPI 3.0 specification with server, channels, and operations
  ```bash
  cargo run --example chat_api
  ```

- **`message_integration.rs`** - Demonstrates automatic message integration with `#[asyncapi_messages(...)]`
  ```bash
  cargo run --example message_integration
  ```

## Motivation

Manually maintaining AsyncAPI specifications is error-prone and time-consuming:

- ❌ Type changes in Rust require manual YAML updates
- ❌ No compile-time validation of documentation accuracy
- ❌ Easy for docs to drift from implementation
- ❌ Repetitive work defining the same types twice

**asyncapi-rust solves this** by generating AsyncAPI specs directly from your Rust types, providing a single source of truth with compile-time guarantees.

## Comparison: Manual vs Generated

**Before (Manual YAML):**
```yaml
# asyncapi.yaml - must keep in sync manually!
components:
  messages:
    SendMessage:
      payload:
        type: object
        properties:
          type: { type: string, const: SendMessage }
          room: { type: string }
          text: { type: string }
```

**After (Generated from Rust):**
```rust
/// Send a chat message
#[derive(Serialize, Deserialize, ToAsyncApiMessage)]
#[serde(tag = "type", rename = "SendMessage")]
pub struct SendMessage {
    pub room: String,
    pub text: String,
}
// AsyncAPI YAML generated automatically at compile time!
```

## Supported Frameworks

- ✅ **actix-ws** - Full integration with actix-web WebSocket handlers
- ✅ **axum** - Integration with axum WebSocket routes
- 🔄 **Framework-agnostic** - Works with any serde-compatible message types

## Binary Protocol Support

Document binary WebSocket messages (Arrow IPC, Protobuf, MessagePack):

```rust
/// Binary data stream
#[derive(ToAsyncApiMessage)]
#[asyncapi(
    name = "BinaryData",
    content_type = "application/octet-stream",
    binary = true,
    description = "Binary data payload",
)]
pub struct BinaryData;
```

## Generating Specification Files

### Standalone Binary (Recommended)

Create a separate binary in your project to generate AsyncAPI specs:

```rust
// bin/generate-asyncapi.rs
use my_project::MyApi;
use asyncapi_rust::AsyncApi;

fn main() {
    let spec = MyApi::asyncapi_spec();
    let json = serde_json::to_string_pretty(&spec)
        .expect("Failed to serialize spec");

    std::fs::write("docs/asyncapi.json", json)
        .expect("Failed to write spec file");

    println!("✅ Generated docs/asyncapi.json");
}
```

Run with:
```bash
cargo run --bin generate-asyncapi
```

**Benefits:**
- Simple to implement and use
- Works with any build system
- Can commit generated spec to git for CI/CD
- Easy to integrate into workflows

### Including in Rustdoc

You can include the generated spec in your crate's documentation:

```rust
#[doc = include_str!("../docs/asyncapi.json")]
#[derive(AsyncApi)]
#[asyncapi(title = "My API", version = "1.0.0")]
struct MyApi;
```

This embeds the AsyncAPI specification directly in your rustdoc output, making it accessible alongside your Rust API documentation.

**Workflow:**
1. Generate the spec file: `cargo run --bin generate-asyncapi`
2. Build docs: `cargo doc`
3. The AsyncAPI spec will be visible in the rustdoc for `MyApi`

### Future: Cargo Plugin

A `cargo-asyncapi` plugin for automatic spec generation is planned for a future release. This would allow:

```bash
cargo asyncapi generate
cargo asyncapi serve  # Start AsyncAPI UI viewer
```

## Examples

The `examples/` directory contains working demonstrations:

- **`simple.rs`** - Basic message types with schema generation
- **`chat_api.rs`** - Complete AsyncAPI 3.0 specification
- **`asyncapi_derive.rs`** - Using `#[derive(AsyncApi)]` for specs
- **`generate_spec_file.rs`** - Generating specification files
- **`full_asyncapi_derive.rs`** - Complete spec with servers, channels, operations
- **`message_integration.rs`** - Automatic message integration with `#[asyncapi_messages(...)]`
- **`actix_websocket.rs`** - Real-world actix-web + actix-ws integration
- **`axum_websocket.rs`** - Real-world axum WebSocket integration
- **`framework_integration_guide.rs`** - Comprehensive framework integration guide

Run any example:
```bash
cargo run --example message_integration
cargo run --example actix_websocket
cargo run --example axum_websocket
```

## Documentation

- [API Documentation](https://docs.rs/asyncapi-rust)
- [User Guide](docs/guide.md)
- [Migration from Manual Specs](docs/migration.md)
- [Binary Protocol Support](docs/binary-protocols.md)

## Roadmap

- [x] Core macro implementation
- [x] actix-ws integration
- [x] axum integration
- [x] Binary message support
- [ ] Embedded AsyncAPI UI
- [ ] Additional framework support (tonic/gRPC, Rocket, Warp)
- [ ] Cargo plugin (`cargo-asyncapi`) for automated spec generation

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgments

Inspired by:
- [utoipa](https://github.com/juhaku/utoipa) - OpenAPI code generation for Rust
- [AsyncAPI Initiative](https://www.asyncapi.com/) - AsyncAPI specification

---

**Author:** Mark Lilback (mark@lilback.com)
**Repository:** https://github.com/mlilback/asyncapi-rust
