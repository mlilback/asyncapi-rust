//! Procedural macro implementation for asyncapi-rust
//!
//! This crate provides the proc macros used by asyncapi-rust:
//! - `#[derive(ToAsyncApiMessage)]` - Generate message metadata from Rust types
//! - `#[derive(AsyncApi)]` - Generate complete AsyncAPI spec

#![warn(clippy::all)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

mod serde_attrs;

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

    // Parse enum variants
    let messages = match &input.data {
        Data::Enum(data_enum) => {
            let mut message_names = Vec::new();

            for variant in &data_enum.variants {
                let variant_name = &variant.ident;

                // Check for serde(rename) attribute on variant
                let message_name = extract_serde_rename(&variant.attrs)
                    .unwrap_or_else(|| variant_name.to_string());

                message_names.push(message_name);
            }

            message_names
        }
        Data::Struct(_) => {
            // For structs, just use the type name
            vec![name.to_string()]
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(name, "ToAsyncApiMessage cannot be derived for unions")
                .to_compile_error()
                .into();
        }
    };

    let message_count = messages.len();
    let message_literals = messages.iter().map(|s| s.as_str());

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
                use schemars::schema::RootSchema;

                let message_names = Self::asyncapi_message_names();
                let schema = schema_for!(Self);

                // Convert schemars RootSchema to our Schema type
                let schema_json = serde_json::to_value(&schema)
                    .expect("Failed to serialize schema");

                let payload_schema: asyncapi_rust::Schema = serde_json::from_value(schema_json)
                    .expect("Failed to deserialize schema");

                // Create a Message for each message name
                message_names
                    .iter()
                    .map(|name| asyncapi_rust::Message {
                        name: Some(name.to_string()),
                        title: Some(name.to_string()),
                        summary: None,
                        description: None,
                        content_type: Some("application/json".to_string()),
                        payload: Some(payload_schema.clone()),
                    })
                    .collect()
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
/// )]
/// struct ChatApi;
/// ```
#[proc_macro_derive(AsyncApi, attributes(asyncapi))]
pub fn derive_asyncapi(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // TODO: Parse asyncapi attributes
    // TODO: Generate AsyncApiSpec

    // Placeholder implementation
    let expanded = quote! {
        impl #name {
            /// Generate the AsyncAPI specification
            pub fn asyncapi() -> String {
                format!("AsyncAPI spec for {}", stringify!(#name))
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
