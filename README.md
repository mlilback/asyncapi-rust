# asyncapi-rust

[![Crates.io](https://img.shields.io/crates/v/asyncapi-rust.svg)](https://crates.io/crates/asyncapi-rust)
[![Documentation](https://docs.rs/asyncapi-rust/badge.svg)](https://docs.rs/asyncapi-rust)
[![codecov](https://codecov.io/gh/mlilback/asyncapi-rust/graph/badge.svg)](https://codecov.io/gh/mlilback/asyncapi-rust)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

**AsyncAPI 3.0 specification generation for Rust WebSockets and async protocols**

Generate AsyncAPI documentation directly from your Rust code using procedural macros. Similar to how `utoipa` generates OpenAPI specs for REST APIs, `asyncapi-rust` generates AsyncAPI specs for WebSocket and other async protocols.

## Table of Contents

- [Features](#features)
- [Migrating from 0.3.x](#migrating-from-03x)
- [Migrating from 0.2.x](#migrating-from-02x)
- [Quick Start](#quick-start)
  - [Message Integration](#message-integration)
  - [Server Variables and Channel Parameters](#server-variables-and-channel-parameters)
  - [Message Naming and Disambiguation](#message-naming-and-disambiguation)
- [Examples](#examples)
- [Motivation](#motivation)
- [Comparison: Manual vs Generated](#comparison-manual-vs-generated)
- [Supported Frameworks](#supported-frameworks)
- [Binary Protocol Support](#binary-protocol-support)
- [DateTime Support (Chrono)](#datetime-support-chrono)
- [Generating Specification Files](#generating-specification-files)
- [Documentation](#documentation)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [Statement on AI/LLM Usage](#statement-on-aillm-usage)
- [Acknowledgments](#acknowledgments)

## Features

- 🦀 **Code-first**: Generate specs from Rust types, not YAML
- ⚡ **Compile-time**: Zero runtime cost, all generation at build time
- 🔒 **Type-safe**: Compile errors if documentation drifts from code
- 🎯 **Familiar**: Follows patterns from [`utoipa`](https://crates.io/crates/utoipa), [`serde`](https://serde.rs), and [`clap`](https://crates.io/crates/clap)
- 🌐 **Framework agnostic**: Works with actix-ws, axum, or any serde-compatible types
- 📦 **Binary protocols**: Support for mixed text/binary WebSocket messages (Arrow IPC, Protobuf, etc.)

## Migrating from 0.3.x

**Breaking change in 0.4.0:** all map-typed fields on model structs (`servers`, `channels`, `operations`, `components.messages`, `components.schemas`, `properties`, etc.) changed from `std::collections::HashMap` to `indexmap::IndexMap`. This makes generated spec output byte-stable across builds — previously `HashMap`'s random iteration order caused churn in downstream TypeScript codegen and other consumers even when the API hadn't changed.

If you construct model structs directly (e.g. in tests or a custom spec builder), replace `HashMap::new()` with `IndexMap::new()`. `indexmap` is re-exported from `asyncapi_rust` so you don't need to add it to your own `Cargo.toml`:

```rust
use asyncapi_rust::indexmap::IndexMap;

let mut servers = IndexMap::new();
servers.insert("production".to_string(), my_server);
```

Code that only _reads_ from these fields (iterating, `.get()`, `.contains_key()`) compiles unchanged — `IndexMap` has the same API as `HashMap` for read operations.

## Migrating from 0.2.x

**Breaking change in 0.3.0:** message names in `components.messages` and `asyncapi_message_names()` now derive from the **Rust variant identifier**, not the serde rename string.

| Before (0.2.x) | After (0.3.0) |
|----------------|---------------|
| `messages.get("user.join")` | `messages.get("UserJoin")` |
| `asyncapi_message_names()` → `["user.join", …]` | `asyncapi_message_names()` → `["UserJoin", …]` |

The serde rename string remains the wire discriminant inside the payload schema — wire format is unchanged.

Other 0.3.0 additions:
- `#[asyncapi(message_name = "CustomName")]` per-variant override for disambiguation
- Runtime collision detection: two enums sharing a variant identifier in the same document panic with a clear error instead of silently overwriting
- `Schema::Any` handles `serde_json::Value` fields without panicking
- AsyncAPI 3.0–compliant `Parameter` object (removed `schema`, added `default`, `enum_values`, `examples`, `location`)
- Shared `$defs` are hoisted to `components.schemas`; message payloads reference them via `$ref: "#/components/schemas/X"`

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
asyncapi-rust = "0.4"
serde = { version = "1.0", features = ["derive"] }
schemars = { version = "1.1", features = ["derive"] }

# Optional: for chrono datetime support in schemas
chrono = { version = "0.4", features = ["serde"] }
schemars = { version = "1.1", features = ["derive", "chrono04"] }
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
    // Get message names — returns Rust variant identifiers, not serde rename strings
    let names = ChatMessage::asyncapi_message_names();
    println!("Messages: {:?}", names); // ["UserJoin", "Chat"]

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

### Server Variables and Channel Parameters

Define dynamic server paths and channel parameters for WebSocket connections:

```rust
use asyncapi_rust::AsyncApi;

#[derive(AsyncApi)]
#[asyncapi(title = "User WebSocket API", version = "1.0.0")]
#[asyncapi_server(
    name = "production",
    host = "api.enlightenhq.com",
    protocol = "wss",
    pathname = "/api/ws/{userId}",
    variable(
        name = "userId",
        description = "Authenticated user ID",
        examples = ["12", "13"]
    )
)]
#[asyncapi_channel(
    name = "rtMessaging",
    address = "/api/ws/{userId}",
    parameter(
        name = "userId",
        description = "User ID for this WebSocket connection",
        examples = ["42", "100"]
    )
)]
struct UserApi;
```

**Server variables** define placeholders in server URLs with:
- `name`: Variable name (required)
- `description`: Human-readable description
- `examples`: Example values for documentation
- `default`: Default value if not provided
- `enum_values`: Restricted set of allowed values

**Channel parameters** define path parameters with:
- `name`: Parameter name (required)
- `description`: Human-readable description
- `default`: Default value if not provided
- `enum_values`: Restricted set of allowed values (e.g., `["v1", "v2"]`)
- `examples`: Example values for documentation (e.g., `["42", "100"]`)
- `location`: Runtime expression for the parameter's location

### Message Naming and Disambiguation

By default, message names in `components.messages` and `asyncapi_message_names()` are the **Rust variant identifiers** (`UserJoin`, `Chat`), not the serde rename strings (`"user.join"`, `"chat.message"`). The serde rename is preserved as the wire discriminant inside the payload schema.

If two `ToAsyncApiMessage` enums in the same AsyncAPI document share a variant identifier, the runtime detects the collision and panics with a clear message. Use `#[asyncapi(message_name = "…")]` to disambiguate:

```rust
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "message")]
pub enum Operation {
    #[serde(rename = "get-info")]
    GetInfo { project_id: i64 },
}

#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "message")]
pub enum OperationResponse {
    // Same wire discriminant as Operation::GetInfo, but a distinct message name
    #[serde(rename = "get-info")]
    #[asyncapi(message_name = "GetInfoResponse")]
    GetInfo { id: i64, label: String },
}
```

Both messages appear in `components.messages` under distinct keys (`GetInfo` and `GetInfoResponse`), with `"get-info"` as the wire value in both payload schemas.

`#[asyncapi(message_name = "…")]` attributes:
- `message_name = "CustomName"`: Override the default (variant ident) for a single variant
- An empty serde rename (`#[serde(rename = "")]`) automatically falls back to the variant identifier — no override needed

## Examples

See working examples in the `examples/` directory:

- **`simple.rs`** - Basic message types with schema generation
- **`chat_api.rs`** - Complete AsyncAPI 3.0 specification with server, channels, and operations
- **`message_integration.rs`** - Automatic message integration with `#[asyncapi_messages(...)]`
- **`server_variables.rs`** - Server variables and channel parameters for dynamic paths
- **`asyncapi_derive.rs`** - Using `#[derive(AsyncApi)]` for specs
- **`full_asyncapi_derive.rs`** - Complete spec with servers, channels, operations
- **`generate_spec_file.rs`** - Generating specification files
- **`actix_websocket.rs`** - Real-world actix-web + actix-ws integration
- **`axum_websocket.rs`** - Real-world axum WebSocket integration
- **`framework_integration_guide.rs`** - Comprehensive framework integration guide

Run any example:
```bash
cargo run --example simple
cargo run --example message_integration
cargo run --example server_variables
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
    content_type = "application/octet-stream",
    triggers_binary,
    description = "Raw binary data payload",
)]
pub struct BinaryData;
```

## DateTime Support (Chrono)

asyncapi-rust uses `schemars 1.1` with full support for `chrono` datetime types:

```rust
use asyncapi_rust::{schemars::JsonSchema, ToAsyncApiMessage};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum TimestampedMessage {
    /// Event with timestamp
    Event {
        timestamp: DateTime<Utc>,     // RFC3339 format
        created_at: NaiveDateTime,    // ISO8601 without timezone
        message: String,
    },
}
```

**Cargo.toml configuration:**
```toml
[dependencies]
asyncapi-rust = "0.4"
chrono = { version = "0.4", features = ["serde"] }
schemars = { version = "1.1", features = ["derive", "chrono04"] }
```

The `chrono04` feature in schemars enables proper JSON schema generation for chrono datetime types. Without this feature, you would need to use `#[schemars(skip)]` and lose schema information for datetime fields.

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
- [x] ~98% test coverage (measured by cargo-tarpaulin)

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Statement on AI/LLM Usage

There has been a lot of discussion in the Rust community about usage of AI and LLMs. This project has been implemented with the assistance of Claude Code, but it is *not* vibe-coded. In a few years, using AI Tools will be common practice and these arguments will seem as quaint as those made decades ago against the use of IDEs. A human has designed this project and reviewed all code.

## Acknowledgments

Inspired by:
- [utoipa](https://github.com/juhaku/utoipa) - OpenAPI code generation for Rust
- [AsyncAPI Initiative](https://www.asyncapi.com/) - AsyncAPI specification

---

**Author:** Mark Lilback (mark@lilback.com)
**Repository:** https://github.com/mlilback/asyncapi-rust
