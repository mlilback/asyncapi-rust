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
        /// Stable message identity used in components.messages and asyncapi_message_names().
        /// Defaults to the Rust variant/type identifier; overridable via
        /// `#[asyncapi(message_name = "...")]`.
        name: String,
        /// Wire discriminant value from serde rename (used for payload schema lookup).
        /// May be an empty string when `#[serde(rename = "")]`; defaults to variant ident.
        discriminant: String,
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
                let variant_ident_str = variant_name.to_string();

                // Wire discriminant: serde rename if present (even if empty), else variant ident.
                let discriminant = extract_serde_rename(&variant.attrs)
                    .unwrap_or_else(|| variant_ident_str.clone());

                // Extract asyncapi metadata
                let asyncapi_meta = extract_asyncapi_meta(&variant.attrs);

                // Message identity: explicit message_name override, else variant ident.
                // We deliberately do NOT use the serde rename here — it may be empty,
                // non-unique across enums, or unsuitable as a code identifier.
                let message_name = asyncapi_meta
                    .message_name
                    .clone()
                    .unwrap_or_else(|| variant_ident_str.clone());

                message_metas.push(MessageMeta {
                    name: message_name,
                    discriminant,
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
            let struct_name = name.to_string();
            let message_name = asyncapi_meta
                .message_name
                .clone()
                .unwrap_or_else(|| struct_name.clone());

            vec![MessageMeta {
                name: message_name,
                discriminant: struct_name,
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
    // Wire discriminant for each variant — used to look up per-variant schemas at runtime.
    let message_discriminants = messages.iter().map(|m| m.discriminant.as_str());
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
        // const _: () scopes the helper so it doesn't leak into the user's namespace
        const _: () = {
            /// Rewrites schemars' `#/$defs/X` refs to `#/components/schemas/X` in-place.
            fn rewrite_defs_refs(value: &mut serde_json::Value) {
                match value {
                    serde_json::Value::Object(map) => {
                        if let Some(r) = map.get_mut("$ref") {
                            if let Some(s) = r.as_str() {
                                if let Some(name) = s.strip_prefix("#/$defs/") {
                                    *r = serde_json::Value::String(
                                        format!("#/components/schemas/{}", name)
                                    );
                                }
                            }
                        }
                        for v in map.values_mut() {
                            rewrite_defs_refs(v);
                        }
                    }
                    serde_json::Value::Array(arr) => {
                        for v in arr.iter_mut() {
                            rewrite_defs_refs(v);
                        }
                    }
                    _ => {}
                }
            }

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

                /// Return shared schema definitions for this type, keyed by name.
                ///
                /// These are the `$defs` that schemars generates for sub-types referenced
                /// by this type's variants. The `AsyncApi` derive collects them into
                /// `components.schemas` so message payloads can reference them via
                /// `#/components/schemas/X` instead of embedding them inline.
                pub fn asyncapi_schemas() -> asyncapi_rust::indexmap::IndexMap<String, asyncapi_rust::Schema>
                where
                    Self: schemars::JsonSchema,
                {
                    use schemars::schema_for;
                    let schema = schema_for!(Self);
                    let schema_json = serde_json::to_value(&schema)
                        .expect("Failed to serialize schema");

                    let mut result = asyncapi_rust::indexmap::IndexMap::new();
                    if let Some(defs) = schema_json.get("$defs").and_then(|v| v.as_object()) {
                        for (name, def_schema) in defs {
                            let mut def = def_schema.clone();
                            rewrite_defs_refs(&mut def);
                            if let Ok(s) = serde_json::from_value::<asyncapi_rust::Schema>(def) {
                                result.insert(name.clone(), s);
                            }
                        }
                    }
                    result
                }

                /// Generate AsyncAPI Message objects with JSON schemas.
                ///
                /// For internally-tagged enums each message carries only its own variant
                /// schema. `$ref`s within payloads point to `#/components/schemas/X`;
                /// the corresponding definitions are available via `asyncapi_schemas()`.
                pub fn asyncapi_messages() -> Vec<asyncapi_rust::Message>
                where
                    Self: schemars::JsonSchema,
                {
                    use schemars::schema_for;

                    let schema = schema_for!(Self);
                    let schema_json = serde_json::to_value(&schema)
                        .expect("Failed to serialize schema");

                    // Build a discriminant→schema map using the actual serde tag field name.
                    let tag_field = Self::asyncapi_tag_field();
                    let mut variant_schemas: asyncapi_rust::indexmap::IndexMap<String, serde_json::Value> =
                        asyncapi_rust::indexmap::IndexMap::new();
                    if let Some(tag) = tag_field {
                        if let Some(variants) = schema_json.get("oneOf").and_then(|v| v.as_array()) {
                            for variant in variants {
                                let discriminant = variant
                                    .get("properties")
                                    .and_then(|props| props.get(tag))
                                    .and_then(|tag_prop| {
                                        tag_prop.get("const").or_else(|| {
                                            tag_prop
                                                .get("enum")
                                                .and_then(|e| e.as_array())
                                                .and_then(|a| a.first())
                                        })
                                    })
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());

                                if let Some(name) = discriminant {
                                    let mut variant_schema = variant.clone();
                                    // Drop $defs — they live in components.schemas, not the payload.
                                    if let Some(obj) = variant_schema.as_object_mut() {
                                        obj.remove("$defs");
                                    }
                                    rewrite_defs_refs(&mut variant_schema);
                                    variant_schemas.insert(name, variant_schema);
                                }
                            }
                        }
                    }

                    // Metadata arrays are baked in at compile time; schemas resolved at runtime.
                    let names: &[&str] = &[#(#message_names_for_gen),*];
                    // Discriminants are the serde rename values — used to look up per-variant
                    // schemas. Separate from names so empty renames and cross-enum collisions
                    // don't affect message identity.
                    let discriminants: &[&str] = &[#(#message_discriminants),*];
                    let titles: &[Option<String>] = &[#(#message_titles),*];
                    let summaries: &[Option<String>] = &[#(#message_summaries),*];
                    let descriptions: &[Option<String>] = &[#(#message_descriptions),*];
                    let content_types: &[Option<String>] = &[#(#message_content_types),*];

                    let mut messages = Vec::with_capacity(names.len());
                    for i in 0..names.len() {
                        let msg_name = names[i];
                        let discriminant = discriminants[i];
                        let payload = if let Some(v) = variant_schemas.get(discriminant) {
                            serde_json::from_value(v.clone()).ok()
                        } else {
                            // Structs, untagged enums, or variants not in the map:
                            // remove $defs and rewrite refs in the full schema.
                            let mut fallback = schema_json.clone();
                            if let Some(obj) = fallback.as_object_mut() {
                                obj.remove("$defs");
                            }
                            rewrite_defs_refs(&mut fallback);
                            serde_json::from_value(fallback).ok()
                        };
                        messages.push(asyncapi_rust::Message {
                            name: Some(msg_name.to_string()),
                            title: titles[i].clone(),
                            summary: summaries[i].clone(),
                            description: descriptions[i].clone(),
                            content_type: content_types[i].clone(),
                            payload,
                        });
                    }
                    messages
                }
            }
        };
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
                        let mut server_variables = asyncapi_rust::indexmap::IndexMap::new();
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
                let mut servers = asyncapi_rust::indexmap::IndexMap::new();
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
                    let param_default = if let Some(d) = &param.default {
                        quote! { Some(#d.to_string()) }
                    } else {
                        quote! { None }
                    };
                    let param_enum = if param.enum_values.is_empty() {
                        quote! { None }
                    } else {
                        let vals = &param.enum_values;
                        quote! { Some(vec![#(#vals.to_string()),*]) }
                    };
                    let param_examples = if param.examples.is_empty() {
                        quote! { None }
                    } else {
                        let vals = &param.examples;
                        quote! { Some(vec![#(#vals.to_string()),*]) }
                    };
                    let param_location = if let Some(l) = &param.location {
                        quote! { Some(#l.to_string()) }
                    } else {
                        quote! { None }
                    };

                    quote! {
                        channel_parameters.insert(
                            #param_name.to_string(),
                            asyncapi_rust::Parameter {
                                description: #param_desc,
                                default: #param_default,
                                enum_values: #param_enum,
                                examples: #param_examples,
                                location: #param_location,
                            }
                        );
                    }
                });

                quote! {
                    {
                        let mut channel_parameters = asyncapi_rust::indexmap::IndexMap::new();
                        #(#param_entries)*
                        Some(channel_parameters)
                    }
                }
            };

            quote! {
                channels.insert(
                    #name.to_string(),
                    asyncapi_rust::Channel {
                        address: #address,
                        messages: None,
                        parameters: #parameters,
                    }
                );
            }
        });

        quote! {
            {
                let mut channels = asyncapi_rust::indexmap::IndexMap::new();
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
                let mut operations = asyncapi_rust::indexmap::IndexMap::new();
                #(#operation_entries)*
                Some(operations)
            }
        }
    };

    // Generate components with messages and hoisted shared schemas
    let components_code = if spec_meta.message_types.is_empty() {
        quote! { None }
    } else {
        let type_calls = spec_meta.message_types.iter().map(|type_name| {
            quote! {
                for msg in #type_name::asyncapi_messages() {
                    if let Some(ref name) = msg.name {
                        if messages.contains_key(name.as_str()) {
                            panic!(
                                "asyncapi-rust: message name collision for '{}' from {}. \
                                 Use #[asyncapi(message_name = \"...\")] on one variant to disambiguate.",
                                name,
                                stringify!(#type_name)
                            );
                        }
                        messages.insert(name.clone(), msg.clone());
                    }
                }
                // Hoist shared $defs into components.schemas (first writer wins on name collision)
                for (name, schema) in #type_name::asyncapi_schemas() {
                    schemas.entry(name).or_insert(schema);
                }
            }
        });

        quote! {
            {
                let mut messages = asyncapi_rust::indexmap::IndexMap::new();
                let mut schemas = asyncapi_rust::indexmap::IndexMap::new();
                #(#type_calls)*
                Some(asyncapi_rust::Components {
                    messages: if messages.is_empty() { None } else { Some(messages) },
                    schemas: if schemas.is_empty() { None } else { Some(schemas) },
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
