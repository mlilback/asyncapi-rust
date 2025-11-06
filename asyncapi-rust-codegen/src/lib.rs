//! Procedural macro implementation for asyncapi-rust
//!
//! This crate provides the procedural macros that power `asyncapi-rust`, enabling
//! compile-time generation of AsyncAPI 3.0 specifications from Rust code.
//!
//! ## Overview
//!
//! Two derive macros are provided:
//!
//! ### `#[derive(ToAsyncApiMessage)]`
//!
//! Generates message metadata and JSON schemas from Rust types (structs or enums).
//!
//! - Works with [`serde`](https://serde.rs) for serialization patterns
//! - Uses [`schemars`](https://docs.rs/schemars) for JSON Schema generation
//! - Supports `#[asyncapi(...)]` helper attributes for documentation
//! - Generates methods: `asyncapi_message_names()`, `asyncapi_messages()`, etc.
//!
//! **Example:**
//! ```rust,ignore
//! use asyncapi_rust::{ToAsyncApiMessage, schemars::JsonSchema};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
//! #[serde(tag = "type")]
//! pub enum ChatMessage {
//!     #[serde(rename = "user.join")]
//!     #[asyncapi(
//!         summary = "User joins",
//!         description = "Sent when a user enters a room"
//!     )]
//!     UserJoin { username: String, room: String },
//!
//!     #[serde(rename = "chat.message")]
//!     #[asyncapi(summary = "Chat message")]
//!     Chat { username: String, room: String, text: String },
//! }
//!
//! // Generated methods available:
//! let names = ChatMessage::asyncapi_message_names();
//! let messages = ChatMessage::asyncapi_messages(); // Requires JsonSchema
//! ```
//!
//! ### `#[derive(AsyncApi)]`
//!
//! Generates complete AsyncAPI 3.0 specifications with servers, channels, and operations.
//!
//! - Requires `title` and `version` attributes
//! - Supports optional `description` attribute
//! - Use `#[asyncapi_server(...)]` to define servers
//! - Use `#[asyncapi_channel(...)]` to define channels
//! - Use `#[asyncapi_operation(...)]` to define operations
//! - Can use multiple of each attribute type
//!
//! **Example:**
//! ```rust,ignore
//! use asyncapi_rust::AsyncApi;
//!
//! #[derive(AsyncApi)]
//! #[asyncapi(
//!     title = "Chat API",
//!     version = "1.0.0",
//!     description = "Real-time chat application"
//! )]
//! #[asyncapi_server(
//!     name = "production",
//!     host = "chat.example.com",
//!     protocol = "wss",
//!     description = "Production WebSocket server"
//! )]
//! #[asyncapi_channel(
//!     name = "chat",
//!     address = "/ws/chat"
//! )]
//! #[asyncapi_operation(
//!     name = "sendMessage",
//!     action = "send",
//!     channel = "chat"
//! )]
//! #[asyncapi_operation(
//!     name = "receiveMessage",
//!     action = "receive",
//!     channel = "chat"
//! )]
//! struct ChatApi;
//!
//! // Generated method:
//! let spec = ChatApi::asyncapi_spec();
//! ```
//!
//! ## Supported Attributes
//!
//! ### `#[asyncapi(...)]` on message types
//!
//! Helper attributes for documenting messages (used with `ToAsyncApiMessage`):
//!
//! - `summary = "..."` - Short summary of the message
//! - `description = "..."` - Detailed description
//! - `title = "..."` - Human-readable title (defaults to message name)
//! - `content_type = "..."` - Content type (defaults to "application/json")
//! - `triggers_binary` - Flag for binary messages (sets content_type to "application/octet-stream")
//!
//! ### `#[asyncapi(...)]` on API specs
//!
//! Required attributes for complete specifications (used with `AsyncApi`):
//!
//! - `title = "..."` - API title (required)
//! - `version = "..."` - API version (required)
//! - `description = "..."` - API description (optional)
//!
//! ### `#[asyncapi_server(...)]`
//!
//! Define server connection information:
//!
//! - `name = "..."` - Server identifier (required)
//! - `host = "..."` - Server host/URL (required)
//! - `protocol = "..."` - Protocol (e.g., "wss", "ws", "grpc") (required)
//! - `description = "..."` - Server description (optional)
//!
//! ### `#[asyncapi_channel(...)]`
//!
//! Define communication channels:
//!
//! - `name = "..."` - Channel identifier (required)
//! - `address = "..."` - Channel path/address (optional)
//!
//! ### `#[asyncapi_operation(...)]`
//!
//! Define send/receive operations:
//!
//! - `name = "..."` - Operation identifier (required)
//! - `action = "send"|"receive"` - Operation type (required)
//! - `channel = "..."` - Channel reference (required)
//!
//! ## Integration with serde
//!
//! The macros respect serde attributes for naming and structure:
//!
//! - `#[serde(rename = "...")]` - Use custom name in AsyncAPI spec
//! - `#[serde(tag = "...")]` - Tagged enum with discriminator field
//! - `#[serde(skip)]` - Exclude fields from schema
//! - `#[serde(skip_serializing_if = "...")]` - Optional fields
//!
//! ## Integration with schemars
//!
//! JSON schemas are generated automatically using schemars:
//!
//! - Requires `JsonSchema` derive on message types
//! - Generates complete JSON Schema from Rust type definitions
//! - Supports nested types, generics, and references
//! - Schemas include validation rules from type constraints
//!
//! ## Generated Code
//!
//! The macros generate implementations with these methods:
//!
//! **From `ToAsyncApiMessage`:**
//! - `asyncapi_message_names() -> Vec<&'static str>` - Get all message names
//! - `asyncapi_message_count() -> usize` - Number of messages
//! - `asyncapi_tag_field() -> Option<&'static str>` - Serde tag field if present
//! - `asyncapi_messages() -> Vec<Message>` - Generate messages with schemas
//!
//! **From `AsyncApi`:**
//! - `asyncapi_spec() -> AsyncApiSpec` - Generate complete specification
//!
//! ## Implementation Notes
//!
//! - All code generation happens at compile time (proc macros)
//! - Zero runtime cost - generates plain Rust code
//! - Compile errors if documentation drifts from code
//! - Type-safe - uses Rust's type system for validation

#![warn(clippy::all)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

mod asyncapi_attrs;
mod asyncapi_spec_attrs;
mod serde_attrs;

use asyncapi_attrs::extract_asyncapi_meta;
use asyncapi_spec_attrs::extract_asyncapi_spec_meta;
use serde_attrs::{extract_serde_rename, extract_serde_tag};

/// Derive macro for generating AsyncAPI message metadata
///
/// # Example
///
/// ```rust,ignore
/// use asyncapi_rust::ToAsyncApiMessage;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize, ToAsyncApiMessage)]
/// #[serde(tag = "type")]
/// pub enum Message {
///     #[serde(rename = "chat")]
///     Chat { room: String, text: String },
///     Echo { id: i64, text: String },
/// }
/// ```
#[proc_macro_derive(ToAsyncApiMessage, attributes(asyncapi))]
pub fn derive_to_asyncapi_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract serde tag attribute from enum
    let tag_field = extract_serde_tag(&input.attrs);

    // Struct to hold message metadata
    struct MessageMeta {
        name: String,
        summary: Option<String>,
        description: Option<String>,
        title: Option<String>,
        content_type: Option<String>,
        triggers_binary: bool,
    }

    // Parse enum variants or struct
    let messages = match &input.data {
        Data::Enum(data_enum) => {
            let mut message_metas = Vec::new();

            for variant in &data_enum.variants {
                let variant_name = &variant.ident;

                // Check for serde(rename) attribute on variant
                let message_name = extract_serde_rename(&variant.attrs)
                    .unwrap_or_else(|| variant_name.to_string());

                // Extract asyncapi metadata
                let asyncapi_meta = extract_asyncapi_meta(&variant.attrs);

                message_metas.push(MessageMeta {
                    name: message_name,
                    summary: asyncapi_meta.summary,
                    description: asyncapi_meta.description,
                    title: asyncapi_meta.title,
                    content_type: asyncapi_meta.content_type,
                    triggers_binary: asyncapi_meta.triggers_binary,
                });
            }

            message_metas
        }
        Data::Struct(_) => {
            // For structs, extract metadata from the struct itself
            let asyncapi_meta = extract_asyncapi_meta(&input.attrs);

            vec![MessageMeta {
                name: name.to_string(),
                summary: asyncapi_meta.summary,
                description: asyncapi_meta.description,
                title: asyncapi_meta.title,
                content_type: asyncapi_meta.content_type,
                triggers_binary: asyncapi_meta.triggers_binary,
            }]
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(name, "ToAsyncApiMessage cannot be derived for unions")
                .to_compile_error()
                .into();
        }
    };

    let message_count = messages.len();
    let message_literals = messages.iter().map(|m| m.name.as_str());

    // Prepare metadata for message generation
    let message_names_for_gen = messages.iter().map(|m| m.name.as_str());
    let message_titles = messages.iter().map(|m| {
        if let Some(ref title) = m.title {
            quote! { Some(#title.to_string()) }
        } else {
            let name = &m.name;
            quote! { Some(#name.to_string()) }
        }
    });
    let message_summaries = messages.iter().map(|m| {
        if let Some(ref summary) = m.summary {
            quote! { Some(#summary.to_string()) }
        } else {
            quote! { None }
        }
    });
    let message_descriptions = messages.iter().map(|m| {
        if let Some(ref desc) = m.description {
            quote! { Some(#desc.to_string()) }
        } else {
            quote! { None }
        }
    });
    let message_content_types = messages.iter().map(|m| {
        if let Some(ref ct) = m.content_type {
            quote! { Some(#ct.to_string()) }
        } else if m.triggers_binary {
            quote! { Some("application/octet-stream".to_string()) }
        } else {
            quote! { Some("application/json".to_string()) }
        }
    });

    let tag_info = if let Some(tag) = tag_field {
        quote! {
            Some(#tag)
        }
    } else {
        quote! { None }
    };

    let expanded = quote! {
        impl #name {
            /// Get AsyncAPI message names for this type
            pub fn asyncapi_message_names() -> Vec<&'static str> {
                vec![#(#message_literals),*]
            }

            /// Get the number of messages in this type
            pub fn asyncapi_message_count() -> usize {
                #message_count
            }

            /// Get the serde tag field name if this is a tagged enum
            pub fn asyncapi_tag_field() -> Option<&'static str> {
                #tag_info
            }

            /// Generate AsyncAPI Message objects with JSON schemas
            ///
            /// This method requires that the type implements `schemars::JsonSchema`.
            pub fn asyncapi_messages() -> Vec<asyncapi_rust::Message>
            where
                Self: schemars::JsonSchema,
            {
                use schemars::schema_for;

                let schema = schema_for!(Self);

                // Convert schemars RootSchema to our Schema type
                let schema_json = serde_json::to_value(&schema)
                    .expect("Failed to serialize schema");

                let payload_schema: asyncapi_rust::Schema = serde_json::from_value(schema_json)
                    .expect("Failed to deserialize schema");

                // Create messages with metadata
                vec![#(asyncapi_rust::Message {
                    name: Some(#message_names_for_gen.to_string()),
                    title: #message_titles,
                    summary: #message_summaries,
                    description: #message_descriptions,
                    content_type: #message_content_types,
                    payload: Some(payload_schema.clone()),
                }),*]
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for generating complete AsyncAPI specification
///
/// # Example
///
/// ```rust,ignore
/// use asyncapi_rust::AsyncApi;
///
/// #[derive(AsyncApi)]
/// #[asyncapi(
///     title = "Chat API",
///     version = "1.0.0",
///     description = "A real-time chat API"
/// )]
/// struct ChatApi;
/// ```
#[proc_macro_derive(AsyncApi, attributes(asyncapi, asyncapi_server, asyncapi_channel, asyncapi_operation))]
pub fn derive_asyncapi(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract asyncapi spec metadata
    let spec_meta = extract_asyncapi_spec_meta(&input.attrs);

    // Validate required fields
    let title = match spec_meta.title {
        Some(t) => t,
        None => {
            return syn::Error::new_spanned(
                name,
                "AsyncApi requires a title attribute: #[asyncapi(title = \"...\")]"
            )
            .to_compile_error()
            .into();
        }
    };

    let version = match spec_meta.version {
        Some(v) => v,
        None => {
            return syn::Error::new_spanned(
                name,
                "AsyncApi requires a version attribute: #[asyncapi(version = \"...\")]"
            )
            .to_compile_error()
            .into();
        }
    };

    let description = if let Some(desc) = spec_meta.description {
        quote! { Some(#desc.to_string()) }
    } else {
        quote! { None }
    };

    // Generate servers
    let servers_code = if spec_meta.servers.is_empty() {
        quote! { None }
    } else {
        let server_entries = spec_meta.servers.iter().map(|server| {
            let name = &server.name;
            let host = &server.host;
            let protocol = &server.protocol;
            let desc = if let Some(d) = &server.description {
                quote! { Some(#d.to_string()) }
            } else {
                quote! { None }
            };

            quote! {
                servers.insert(
                    #name.to_string(),
                    asyncapi_rust::Server {
                        host: #host.to_string(),
                        protocol: #protocol.to_string(),
                        description: #desc,
                    }
                );
            }
        });

        quote! {
            {
                let mut servers = std::collections::HashMap::new();
                #(#server_entries)*
                Some(servers)
            }
        }
    };

    // Generate channels
    let channels_code = if spec_meta.channels.is_empty() {
        quote! { None }
    } else {
        let channel_entries = spec_meta.channels.iter().map(|channel| {
            let name = &channel.name;
            let address = if let Some(addr) = &channel.address {
                quote! { Some(#addr.to_string()) }
            } else {
                quote! { None }
            };

            quote! {
                channels.insert(
                    #name.to_string(),
                    asyncapi_rust::Channel {
                        address: #address,
                        messages: None,
                    }
                );
            }
        });

        quote! {
            {
                let mut channels = std::collections::HashMap::new();
                #(#channel_entries)*
                Some(channels)
            }
        }
    };

    // Generate operations
    let operations_code = if spec_meta.operations.is_empty() {
        quote! { None }
    } else {
        let operation_entries = spec_meta.operations.iter().map(|operation| {
            let name = &operation.name;
            let channel_ref = &operation.channel;
            let action = &operation.action;

            // Convert action string to OperationAction enum
            let action_enum = if action == "send" {
                quote! { asyncapi_rust::OperationAction::Send }
            } else if action == "receive" {
                quote! { asyncapi_rust::OperationAction::Receive }
            } else {
                return syn::Error::new_spanned(
                    name,
                    format!("Invalid action '{}', must be 'send' or 'receive'", action)
                )
                .to_compile_error();
            };

            quote! {
                operations.insert(
                    #name.to_string(),
                    asyncapi_rust::Operation {
                        action: #action_enum,
                        channel: asyncapi_rust::ChannelRef {
                            reference: format!("#/channels/{}", #channel_ref),
                        },
                        messages: None,
                    }
                );
            }
        });

        quote! {
            {
                let mut operations = std::collections::HashMap::new();
                #(#operation_entries)*
                Some(operations)
            }
        }
    };

    let expanded = quote! {
        impl #name {
            /// Generate the AsyncAPI specification
            ///
            /// Returns an AsyncApiSpec with Info, Servers, Channels, and Operations
            /// sections populated from attributes.
            pub fn asyncapi_spec() -> asyncapi_rust::AsyncApiSpec {
                asyncapi_rust::AsyncApiSpec {
                    asyncapi: "3.0.0".to_string(),
                    info: asyncapi_rust::Info {
                        title: #title.to_string(),
                        version: #version.to_string(),
                        description: #description,
                    },
                    servers: #servers_code,
                    channels: #channels_code,
                    operations: #operations_code,
                    components: None,
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        // Macro expansion tests will go here
    }
}
