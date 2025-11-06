//! Procedural macro implementation for asyncapi-rust
//!
//! This crate provides the proc macros used by asyncapi-rust:
//! - `#[derive(ToAsyncApiMessage)]` - Generate message metadata from Rust types
//! - `#[derive(AsyncApi)]` - Generate complete AsyncAPI spec

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
#[proc_macro_derive(AsyncApi, attributes(asyncapi))]
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

    let expanded = quote! {
        impl #name {
            /// Generate the AsyncAPI specification
            ///
            /// Returns a basic AsyncApiSpec with Info section populated.
            /// Add servers, channels, and operations as needed.
            pub fn asyncapi_spec() -> asyncapi_rust::AsyncApiSpec {
                asyncapi_rust::AsyncApiSpec {
                    asyncapi: "3.0.0".to_string(),
                    info: asyncapi_rust::Info {
                        title: #title.to_string(),
                        version: #version.to_string(),
                        description: #description,
                    },
                    servers: None,
                    channels: None,
                    operations: None,
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
