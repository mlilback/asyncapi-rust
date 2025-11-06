//! Runtime data structures for AsyncAPI 3.0 specifications
//!
//! This crate provides Rust types that represent [AsyncAPI 3.0](https://www.asyncapi.com/docs/reference/specification/v3.0.0)
//! specification objects. These types are used by the proc macros to generate
//! specifications at compile time and can also be constructed manually.
//!
//! ## Overview
//!
//! The main types mirror the AsyncAPI 3.0 specification structure:
//!
//! - [`AsyncApiSpec`] - Root specification object
//! - [`Info`] - General API information
//! - [`Server`] - Server connection details
//! - [`Channel`] - Communication channels
//! - [`Operation`] - Send/receive operations
//! - [`Message`] - Message definitions
//! - [`Schema`] - JSON Schema definitions
//! - [`Components`] - Reusable components
//!
//! ## Serialization
//!
//! All types implement [`serde::Serialize`] and [`serde::Deserialize`] for JSON
//! serialization, following the AsyncAPI 3.0 specification's JSON Schema.
//!
//! ## Example
//!
//! ```rust
//! use asyncapi_rust_models::*;
//! use std::collections::HashMap;
//!
//! // Create a simple AsyncAPI specification
//! let spec = AsyncApiSpec {
//!     asyncapi: "3.0.0".to_string(),
//!     info: Info {
//!         title: "My API".to_string(),
//!         version: "1.0.0".to_string(),
//!         description: Some("A simple API".to_string()),
//!     },
//!     servers: None,
//!     channels: None,
//!     operations: None,
//!     components: None,
//! };
//!
//! // Serialize to JSON
//! let json = serde_json::to_string_pretty(&spec).unwrap();
//! ```

#![deny(missing_docs)]
#![warn(clippy::all)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AsyncAPI 3.0 Specification
///
/// Root document object representing a complete AsyncAPI specification.
///
/// This is the top-level object that contains all information about an API,
/// including servers, channels, operations, and reusable components.
///
/// # Example
///
/// ```rust
/// use asyncapi_rust_models::*;
///
/// let spec = AsyncApiSpec {
///     asyncapi: "3.0.0".to_string(),
///     info: Info {
///         title: "My WebSocket API".to_string(),
///         version: "1.0.0".to_string(),
///         description: Some("Real-time messaging API".to_string()),
///     },
///     servers: None,
///     channels: None,
///     operations: None,
///     components: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncApiSpec {
    /// AsyncAPI version (e.g., "3.0.0")
    pub asyncapi: String,

    /// General information about the API
    pub info: Info,

    /// Server connection details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<HashMap<String, Server>>,

    /// Available channels (communication paths)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<HashMap<String, Channel>>,

    /// Operations (send/receive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operations: Option<HashMap<String, Operation>>,

    /// Reusable components (messages, schemas, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
}

/// API information object
///
/// Contains general metadata about the API such as title, version, and description.
/// This information is displayed in documentation tools and helps users understand
/// the purpose and version of the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    /// API title
    ///
    /// A human-readable name for the API (e.g., "Chat WebSocket API")
    pub title: String,

    /// API version
    ///
    /// The version of the API (e.g., "1.0.0"). Should follow semantic versioning.
    pub version: String,

    /// API description
    ///
    /// A longer description of the API's purpose and functionality (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Server connection information
///
/// Defines connection details for a server that hosts the API. Multiple servers
/// can be defined to support different environments (production, staging, development).
///
/// # Example
///
/// ```rust
/// use asyncapi_rust_models::Server;
///
/// let server = Server {
///     host: "chat.example.com:443".to_string(),
///     protocol: "wss".to_string(),
///     description: Some("Production WebSocket server".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    /// Server URL or host
    ///
    /// The hostname or URL where the server is hosted. May include port number.
    /// Examples: "localhost:8080", "api.example.com", "ws.example.com:443"
    pub host: String,

    /// Protocol (e.g., "wss", "ws", "grpc")
    ///
    /// The protocol used to communicate with the server.
    /// Common values: "ws" (WebSocket), "wss" (WebSocket Secure), "grpc", "mqtt"
    pub protocol: String,

    /// Server description
    ///
    /// An optional human-readable description of the server's purpose or environment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Communication channel
///
/// Represents a communication path through which messages are exchanged.
/// Channels define where messages are sent and received (e.g., WebSocket endpoints,
/// message queue topics, gRPC methods).
///
/// # Example
///
/// ```rust
/// use asyncapi_rust_models::Channel;
///
/// let channel = Channel {
///     address: Some("/ws/chat".to_string()),
///     messages: None, // Messages are typically added via operations
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    /// Channel address/path
    ///
    /// The location where this channel is available. For WebSocket, this is typically
    /// the WebSocket path (e.g., "/ws/chat"). For other protocols, this could be a
    /// topic name, queue name, or method path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Messages available on this channel
    ///
    /// A map of message identifiers to message definitions or references.
    /// Messages define the structure of data that flows through this channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<HashMap<String, MessageRef>>,
}

/// Reference to a message definition
///
/// Messages can be defined either inline or as references to reusable components.
/// This enum supports both patterns, following the AsyncAPI 3.0 specification.
///
/// # Example
///
/// ```rust
/// use asyncapi_rust_models::{MessageRef, Message};
///
/// // Reference to a component message
/// let ref_msg = MessageRef::Reference {
///     reference: "#/components/messages/ChatMessage".to_string(),
/// };
///
/// // Inline message definition
/// let inline_msg = MessageRef::Inline(Box::new(Message {
///     name: Some("ChatMessage".to_string()),
///     title: Some("Chat Message".to_string()),
///     summary: Some("A chat message".to_string()),
///     description: None,
///     content_type: Some("application/json".to_string()),
///     payload: None,
/// }));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageRef {
    /// Reference to component message
    ///
    /// Points to a reusable message definition in the components section.
    /// Format: "#/components/messages/{messageName}"
    Reference {
        /// $ref path
        #[serde(rename = "$ref")]
        reference: String,
    },
    /// Inline message definition
    ///
    /// Embeds the message definition directly rather than referencing a component
    Inline(Box<Message>),
}

/// Message definition
///
/// Represents a message that can be sent or received through a channel.
/// Messages describe the structure, content type, and documentation for data
/// exchanged in asynchronous communication.
///
/// # Example
///
/// ```rust
/// use asyncapi_rust_models::{Message, Schema, SchemaObject};
/// use std::collections::HashMap;
///
/// let message = Message {
///     name: Some("ChatMessage".to_string()),
///     title: Some("Chat Message".to_string()),
///     summary: Some("A message in a chat room".to_string()),
///     description: Some("Sent when a user posts a message".to_string()),
///     content_type: Some("application/json".to_string()),
///     payload: Some(Schema::Object(Box::new(SchemaObject {
///         schema_type: Some("object".to_string()),
///         properties: None,
///         required: None,
///         description: Some("Chat message payload".to_string()),
///         title: None,
///         enum_values: None,
///         const_value: None,
///         items: None,
///         additional_properties: None,
///         one_of: None,
///         any_of: None,
///         all_of: None,
///         additional: HashMap::new(),
///     }))),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message name
    ///
    /// A machine-readable identifier for the message (e.g., "ChatMessage", "user.join")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Message title
    ///
    /// A human-readable title for the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Message summary
    ///
    /// A short summary of what the message is for
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Message description
    ///
    /// A detailed description of the message's purpose and usage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Content type (e.g., "application/json")
    ///
    /// The MIME type of the message payload. Common values:
    /// - "application/json" (default for text messages)
    /// - "application/octet-stream" (binary data)
    /// - "application/x-protobuf" (Protocol Buffers)
    /// - "application/x-msgpack" (MessagePack)
    #[serde(rename = "contentType", skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// Message payload schema
    ///
    /// JSON Schema defining the structure of the message payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Schema>,
}

/// Operation (send or receive)
///
/// Defines an action that can be performed on a channel. Operations describe
/// whether an application sends or receives messages through a specific channel.
///
/// # Example
///
/// ```rust
/// use asyncapi_rust_models::{Operation, OperationAction, ChannelRef};
///
/// let operation = Operation {
///     action: OperationAction::Send,
///     channel: ChannelRef {
///         reference: "#/channels/chat".to_string(),
///     },
///     messages: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation action (send or receive)
    ///
    /// Specifies whether the application sends or receives messages
    pub action: OperationAction,

    /// Channel reference
    ///
    /// Points to the channel where this operation takes place
    pub channel: ChannelRef,

    /// Messages for this operation
    ///
    /// Optional list of messages that can be used with this operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<MessageRef>>,
}

/// Operation action type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationAction {
    /// Send message
    Send,
    /// Receive message
    Receive,
}

/// Reference to a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelRef {
    /// $ref path
    #[serde(rename = "$ref")]
    pub reference: String,
}

/// Reusable components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    /// Message definitions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<HashMap<String, Message>>,

    /// Schema definitions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<HashMap<String, Schema>>,
}

/// JSON Schema object
///
/// Flexible representation that can hold any valid JSON Schema. This type supports
/// both schema references (using `$ref`) and complete inline schema definitions.
///
/// Schemas define the structure and validation rules for message payloads,
/// following the JSON Schema specification.
///
/// # Example
///
/// ## Reference Schema
///
/// ```rust
/// use asyncapi_rust_models::Schema;
///
/// let schema = Schema::Reference {
///     reference: "#/components/schemas/ChatMessage".to_string(),
/// };
/// ```
///
/// ## Object Schema
///
/// ```rust
/// use asyncapi_rust_models::{Schema, SchemaObject};
/// use std::collections::HashMap;
///
/// let schema = Schema::Object(Box::new(SchemaObject {
///     schema_type: Some("object".to_string()),
///     properties: None,
///     required: Some(vec!["username".to_string(), "room".to_string()]),
///     description: Some("A chat message".to_string()),
///     title: Some("ChatMessage".to_string()),
///     enum_values: None,
///     const_value: None,
///     items: None,
///     additional_properties: None,
///     one_of: None,
///     any_of: None,
///     all_of: None,
///     additional: HashMap::new(),
/// }));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Schema {
    /// Reference to another schema ($ref)
    ///
    /// Points to a reusable schema definition in the components section.
    /// Format: "#/components/schemas/{schemaName}"
    Reference {
        /// $ref path
        #[serde(rename = "$ref")]
        reference: String,
    },
    /// Full schema object (boxed to reduce enum size)
    ///
    /// Contains a complete JSON Schema definition with all properties inline
    Object(Box<SchemaObject>),
}

/// Schema object with all JSON Schema properties
///
/// Complete representation of a JSON Schema with support for all standard properties.
/// This struct provides fine-grained control over schema definitions for message payloads.
///
/// # Example
///
/// ```rust
/// use asyncapi_rust_models::{Schema, SchemaObject};
/// use std::collections::HashMap;
///
/// // String property schema
/// let username_schema = Schema::Object(Box::new(SchemaObject {
///     schema_type: Some("string".to_string()),
///     properties: None,
///     required: None,
///     description: Some("User's display name".to_string()),
///     title: None,
///     enum_values: None,
///     const_value: None,
///     items: None,
///     additional_properties: None,
///     one_of: None,
///     any_of: None,
///     all_of: None,
///     additional: HashMap::new(),
/// }));
///
/// // Object schema with properties
/// let mut properties = HashMap::new();
/// properties.insert("username".to_string(), Box::new(username_schema));
///
/// let message_schema = SchemaObject {
///     schema_type: Some("object".to_string()),
///     properties: Some(properties),
///     required: Some(vec!["username".to_string()]),
///     description: Some("A chat message".to_string()),
///     title: Some("ChatMessage".to_string()),
///     enum_values: None,
///     const_value: None,
///     items: None,
///     additional_properties: None,
///     one_of: None,
///     any_of: None,
///     all_of: None,
///     additional: HashMap::new(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaObject {
    /// Schema type
    ///
    /// The JSON Schema type: "object", "array", "string", "number", "integer", "boolean", "null"
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,

    /// Properties (for object type)
    ///
    /// Map of property names to their schemas when schema_type is "object"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Box<Schema>>>,

    /// Required properties
    ///
    /// List of property names that must be present (for object types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    /// Description
    ///
    /// Human-readable description of what this schema represents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Title
    ///
    /// A short title for the schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Enum values
    ///
    /// List of allowed values (for enum types)
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<serde_json::Value>>,

    /// Const value
    ///
    /// A single constant value that this schema must match
    #[serde(rename = "const", skip_serializing_if = "Option::is_none")]
    pub const_value: Option<serde_json::Value>,

    /// Items schema (for array type)
    ///
    /// Schema for array elements when schema_type is "array"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,

    /// Additional properties
    ///
    /// Schema for additional properties not explicitly defined (for object types)
    #[serde(
        rename = "additionalProperties",
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_properties: Option<Box<Schema>>,

    /// OneOf schemas
    ///
    /// Value must match exactly one of these schemas (XOR logic)
    #[serde(rename = "oneOf", skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<Schema>>,

    /// AnyOf schemas
    ///
    /// Value must match at least one of these schemas (OR logic)
    #[serde(rename = "anyOf", skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<Schema>>,

    /// AllOf schemas
    ///
    /// Value must match all of these schemas (AND logic)
    #[serde(rename = "allOf", skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<Schema>>,

    /// Additional fields that may be present in the schema
    ///
    /// Captures any additional JSON Schema properties not explicitly defined above
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

impl Default for AsyncApiSpec {
    fn default() -> Self {
        Self {
            asyncapi: "3.0.0".to_string(),
            info: Info {
                title: "API".to_string(),
                version: "1.0.0".to_string(),
                description: None,
            },
            servers: None,
            channels: None,
            operations: None,
            components: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_serialization() {
        let spec = AsyncApiSpec::default();
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("asyncapi"));
        assert!(json.contains("3.0.0"));
    }

    #[test]
    fn test_spec_deserialization() {
        let json = r#"{
            "asyncapi": "3.0.0",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            }
        }"#;
        let spec: AsyncApiSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.asyncapi, "3.0.0");
        assert_eq!(spec.info.title, "Test API");
    }
}
