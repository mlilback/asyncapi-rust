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
//!     channel = "chat",
//!     messages = [ChatMessage]
//! )]
//! #[asyncapi_operation(
//!     name = "receiveMessage",
//!     action = "receive",
//!     channel = "chat",
//!     messages = [ChatMessage, SystemMessage]
//! )]
//! #[asyncapi_messages(ChatMessage, SystemMessage)]
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
//! - `messages = [Type1, Type2, ...]` - Message types available for this operation (optional)
//!
//! When the `messages` parameter is specified on operations, those messages are automatically
//! added to the channel that the operation references. Operation messages reference the channel's
//! messages (e.g., `#/channels/{channel}/messages/{message}`), while channel messages reference
//! the components section (e.g., `#/components/messages/{message}`), following AsyncAPI 3.0 spec.
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
    let (messages, _is_enum) = match &input.data {
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

            (message_metas, true)
        }
        Data::Struct(_) => {
            // For structs, extract metadata from the struct itself
            let asyncapi_meta = extract_asyncapi_meta(&input.attrs);

            (
                vec![MessageMeta {
                    name: name.to_string(),
                    summary: asyncapi_meta.summary,
                    description: asyncapi_meta.description,
                    title: asyncapi_meta.title,
                    content_type: asyncapi_meta.content_type,
                    triggers_binary: asyncapi_meta.triggers_binary,
                }],
                false,
            )
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

                // Convert schemars RootSchema to JSON
                let schema_json = serde_json::to_value(&schema)
                    .expect("Failed to serialize schema");

                // For enums, extract individual variant schemas from oneOf
                let variant_schemas = if let Some(one_of_array) = schema_json.get("oneOf") {
                    if let Some(variants) = one_of_array.as_array() {
                        // Create a map of variant name to its schema with capacity
                        let mut variant_map = std::collections::HashMap::with_capacity(variants.len());

                        for variant in variants {
                            // Extract the const value from the type field
                            if let Some(properties) = variant.get("properties") {
                                if let Some(type_prop) = properties.get("type") {
                                    if let Some(const_val) = type_prop.get("const") {
                                        if let Some(variant_name) = const_val.as_str() {
                                            // Convert this variant to a Schema
                                            // Note: clone is necessary here because we need ownership
                                            // of the JSON value to deserialize it
                                            let variant_schema: asyncapi_rust::Schema =
                                                serde_json::from_value(variant.clone())
                                                    .unwrap_or_else(|e| panic!(
                                                        "Failed to deserialize schema for variant '{}': {}",
                                                        variant_name, e
                                                    ));
                                            variant_map.insert(variant_name.to_string(), variant_schema);
                                        }
                                    }
                                }
                            }
                        }

                        Some(variant_map)
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Create messages with metadata
                let message_names = vec![#(#message_names_for_gen),*];
                let message_titles = vec![#(#message_titles),*];
                let message_summaries = vec![#(#message_summaries),*];
                let message_descriptions = vec![#(#message_descriptions),*];
                let message_content_types = vec![#(#message_content_types),*];

                let mut messages = Vec::new();
                for i in 0..message_names.len() {
                    let msg_name = message_names[i];

                    // For enums, try to find the specific variant schema
                    let msg_payload = if let Some(ref variant_schemas) = variant_schemas {
                        // Try to get the specific variant schema for this message
                        variant_schemas.get(msg_name).cloned()
                    } else {
                        // For structs, deserialize and use the full schema
                        let payload_schema: asyncapi_rust::Schema = serde_json::from_value(schema_json.clone())
                            .expect("Failed to deserialize schema");
                        Some(payload_schema)
                    };

                    messages.push(asyncapi_rust::Message {
                        name: Some(msg_name.to_string()),
                        title: message_titles[i].clone(),
                        summary: message_summaries[i].clone(),
                        description: message_descriptions[i].clone(),
                        content_type: message_content_types[i].clone(),
                        payload: msg_payload,
                    });
                }

                messages
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
#[proc_macro_derive(
    AsyncApi,
    attributes(
        asyncapi,
        asyncapi_server,
        asyncapi_channel,
        asyncapi_operation,
        asyncapi_messages
    )
)]
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
                "AsyncApi requires a title attribute: #[asyncapi(title = \"...\")]",
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
                "AsyncApi requires a version attribute: #[asyncapi(version = \"...\")]",
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
            let pathname = if let Some(p) = &server.pathname {
                quote! { Some(#p.to_string()) }
            } else {
                quote! { None }
            };
            let desc = if let Some(d) = &server.description {
                quote! { Some(#d.to_string()) }
            } else {
                quote! { None }
            };

            // Generate server variables
            let variables = if server.variables.is_empty() {
                quote! { None }
            } else {
                let var_entries = server.variables.iter().map(|var| {
                    let var_name = &var.name;
                    let var_desc = if let Some(d) = &var.description {
                        quote! { Some(#d.to_string()) }
                    } else {
                        quote! { None }
                    };
                    let var_default = if let Some(d) = &var.default {
                        quote! { Some(#d.to_string()) }
                    } else {
                        quote! { None }
                    };
                    let var_enum = if var.enum_values.is_empty() {
                        quote! { None }
                    } else {
                        let enum_vals = &var.enum_values;
                        quote! { Some(vec![#(#enum_vals.to_string()),*]) }
                    };
                    let var_examples = if var.examples.is_empty() {
                        quote! { None }
                    } else {
                        let examples = &var.examples;
                        quote! { Some(vec![#(#examples.to_string()),*]) }
                    };

                    quote! {
                        server_variables.insert(
                            #var_name.to_string(),
                            asyncapi_rust::ServerVariable {
                                description: #var_desc,
                                default: #var_default,
                                enum_values: #var_enum,
                                examples: #var_examples,
                            }
                        );
                    }
                });

                quote! {
                    {
                        let mut server_variables = std::collections::HashMap::new();
                        #(#var_entries)*
                        Some(server_variables)
                    }
                }
            };

            quote! {
                servers.insert(
                    #name.to_string(),
                    asyncapi_rust::Server {
                        host: #host.to_string(),
                        protocol: #protocol.to_string(),
                        pathname: #pathname,
                        description: #desc,
                        variables: #variables,
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

            // Generate channel parameters
            let parameters = if channel.parameters.is_empty() {
                quote! { None }
            } else {
                let param_entries = channel.parameters.iter().map(|param| {
                    let param_name = &param.name;
                    let param_desc = if let Some(d) = &param.description {
                        quote! { Some(#d.to_string()) }
                    } else {
                        quote! { None }
                    };

                    // Build schema from schema_type and format
                    let schema = if let Some(schema_type) = &param.schema_type {
                        let format_field = if let Some(fmt) = &param.format {
                            quote! {
                                additional.insert("format".to_string(), serde_json::json!(#fmt));
                            }
                        } else {
                            quote! {}
                        };

                        quote! {
                            {
                                let mut additional = std::collections::HashMap::new();
                                #format_field
                                Some(asyncapi_rust::Schema::Object(Box::new(asyncapi_rust::SchemaObject {
                                    schema_type: Some(serde_json::json!(#schema_type)),
                                    properties: None,
                                    required: None,
                                    description: None,
                                    title: None,
                                    enum_values: None,
                                    const_value: None,
                                    items: None,
                                    additional_properties: None,
                                    one_of: None,
                                    any_of: None,
                                    all_of: None,
                                    additional,
                                })))
                            }
                        }
                    } else {
                        quote! { None }
                    };

                    quote! {
                        channel_parameters.insert(
                            #param_name.to_string(),
                            asyncapi_rust::Parameter {
                                description: #param_desc,
                                schema: #schema,
                            }
                        );
                    }
                });

                quote! {
                    {
                        let mut channel_parameters = std::collections::HashMap::new();
                        #(#param_entries)*
                        Some(channel_parameters)
                    }
                }
            };

            // Collect messages from all operations that reference this channel
            let channel_name_str = name.as_str();
            let operations_for_channel: Vec<_> = spec_meta.operations.iter()
                .filter(|op| op.channel == channel_name_str)
                .collect();

            let messages_field = if operations_for_channel.is_empty() ||
                                   operations_for_channel.iter().all(|op| op.messages.is_empty()) {
                quote! { None }
            } else {
                let message_calls: Vec<_> = operations_for_channel.iter()
                    .flat_map(|op| &op.messages)
                    .collect::<std::collections::HashSet<_>>() // Deduplicate
                    .into_iter()
                    .map(|type_name| {
                        quote! {
                            // Call asyncapi_message_names() for this type and add references
                            for msg_name in #type_name::asyncapi_message_names() {
                                channel_messages.insert(
                                    msg_name.to_string(),
                                    asyncapi_rust::MessageRef::Reference {
                                        reference: format!("#/components/messages/{}", msg_name),
                                    }
                                );
                            }
                        }
                    })
                    .collect();

                quote! {
                    {
                        let mut channel_messages = std::collections::HashMap::new();
                        #(#message_calls)*
                        Some(channel_messages)
                    }
                }
            };

            quote! {
                channels.insert(
                    #name.to_string(),
                    asyncapi_rust::Channel {
                        address: #address,
                        messages: #messages_field,
                        parameters: #parameters,
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
                    format!("Invalid action '{}', must be 'send' or 'receive'", action),
                )
                .to_compile_error();
            };

            // Generate messages references if any messages are specified
            let messages_field = if operation.messages.is_empty() {
                quote! { None }
            } else {
                let message_calls = operation.messages.iter().map(|type_name| {
                    quote! {
                        // Call asyncapi_message_names() for this type and add references to channel messages
                        for msg_name in #type_name::asyncapi_message_names() {
                            message_refs.push(asyncapi_rust::MessageRef::Reference {
                                reference: format!("#/channels/{}/messages/{}", #channel_ref, msg_name),
                            });
                        }
                    }
                });

                quote! {
                    {
                        let mut message_refs = Vec::new();
                        #(#message_calls)*
                        Some(message_refs)
                    }
                }
            };

            quote! {
                operations.insert(
                    #name.to_string(),
                    asyncapi_rust::Operation {
                        action: #action_enum,
                        channel: asyncapi_rust::ChannelRef {
                            reference: format!("#/channels/{}", #channel_ref),
                        },
                        messages: #messages_field,
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

    // Generate components with messages
    let components_code = if spec_meta.message_types.is_empty() {
        quote! { None }
    } else {
        let message_calls = spec_meta.message_types.iter().map(|type_name| {
            quote! {
                // Call asyncapi_messages() for this type and add to messages map
                for msg in #type_name::asyncapi_messages() {
                    if let Some(ref name) = msg.name {
                        messages.insert(name.clone(), msg.clone());
                    }
                }
            }
        });

        quote! {
            {
                let mut messages = std::collections::HashMap::new();
                #(#message_calls)*
                Some(asyncapi_rust::Components {
                    messages: if messages.is_empty() { None } else { Some(messages) },
                    schemas: None,
                })
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
                    components: #components_code,
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
