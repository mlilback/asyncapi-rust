//! Utilities for parsing asyncapi spec-level attributes

use syn::{Attribute, Path};

/// AsyncAPI spec metadata extracted from attributes
#[derive(Debug, Default, Clone)]
pub struct AsyncApiSpecMeta {
    pub title: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub servers: Vec<ServerMeta>,
    pub channels: Vec<ChannelMeta>,
    pub operations: Vec<OperationMeta>,
    pub message_types: Vec<Path>,
}

/// Server metadata
#[derive(Debug, Clone)]
pub struct ServerMeta {
    pub name: String,
    pub host: String,
    pub protocol: String,
    pub pathname: Option<String>,
    pub description: Option<String>,
    pub variables: Vec<ServerVariableMeta>,
}

/// Server variable metadata
#[derive(Debug, Clone)]
pub struct ServerVariableMeta {
    pub name: String,
    pub description: Option<String>,
    pub default: Option<String>,
    pub enum_values: Vec<String>,
    pub examples: Vec<String>,
}

/// Channel metadata
#[derive(Debug, Clone)]
pub struct ChannelMeta {
    pub name: String,
    pub address: Option<String>,
    #[allow(dead_code)] // Reserved for future use
    pub description: Option<String>,
    pub parameters: Vec<ParameterMeta>,
}

/// Channel parameter metadata
#[derive(Debug, Clone)]
pub struct ParameterMeta {
    pub name: String,
    pub description: Option<String>,
    pub schema_type: Option<String>,
    pub format: Option<String>,
}

/// Operation metadata
#[derive(Debug, Clone)]
pub struct OperationMeta {
    pub name: String,
    pub action: String, // "send" or "receive"
    pub channel: String,
    #[allow(dead_code)] // Reserved for future use
    pub description: Option<String>,
    pub messages: Vec<Path>,
}

/// Extract asyncapi spec metadata from `#[asyncapi(...)]` attributes
pub fn extract_asyncapi_spec_meta(attrs: &[Attribute]) -> AsyncApiSpecMeta {
    let mut meta = AsyncApiSpecMeta::default();

    for attr in attrs {
        if attr.path().is_ident("asyncapi") {
            // Parse main asyncapi attributes
            let _ = attr.parse_nested_meta(|nested| {
                if nested.path.is_ident("title") {
                    let value = nested.value()?;
                    let s: syn::LitStr = value.parse()?;
                    meta.title = Some(s.value());
                } else if nested.path.is_ident("version") {
                    let value = nested.value()?;
                    let s: syn::LitStr = value.parse()?;
                    meta.version = Some(s.value());
                } else if nested.path.is_ident("description") {
                    let value = nested.value()?;
                    let s: syn::LitStr = value.parse()?;
                    meta.description = Some(s.value());
                }
                Ok(())
            });
        } else if attr.path().is_ident("asyncapi_server") {
            // Parse server attributes
            if let Some(server) = extract_server(attr) {
                meta.servers.push(server);
            }
        } else if attr.path().is_ident("asyncapi_channel") {
            // Parse channel attributes
            if let Some(channel) = extract_channel(attr) {
                meta.channels.push(channel);
            }
        } else if attr.path().is_ident("asyncapi_operation") {
            // Parse operation attributes
            if let Some(operation) = extract_operation(attr) {
                meta.operations.push(operation);
            }
        } else if attr.path().is_ident("asyncapi_messages") {
            // Parse message type references
            if let Ok(types) = extract_message_types(attr) {
                meta.message_types.extend(types);
            }
        }
    }

    meta
}

/// Extract message type paths from `#[asyncapi_messages(...)]` attribute
fn extract_message_types(attr: &Attribute) -> syn::Result<Vec<Path>> {
    use syn::Token;
    use syn::punctuated::Punctuated;

    // Parse comma-separated list of type paths (e.g., super::messages::Operation, MyType)
    let types = attr.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)?;
    Ok(types.into_iter().collect())
}

/// Extract server metadata from `#[asyncapi_server(...)]` attribute
fn extract_server(attr: &Attribute) -> Option<ServerMeta> {
    let mut name = None;
    let mut host = None;
    let mut protocol = None;
    let mut pathname = None;
    let mut description = None;
    let mut variables = Vec::new();

    let _ = attr.parse_nested_meta(|nested| {
        if nested.path.is_ident("name") {
            let value = nested.value()?;
            let s: syn::LitStr = value.parse()?;
            name = Some(s.value());
        } else if nested.path.is_ident("host") {
            let value = nested.value()?;
            let s: syn::LitStr = value.parse()?;
            host = Some(s.value());
        } else if nested.path.is_ident("protocol") {
            let value = nested.value()?;
            let s: syn::LitStr = value.parse()?;
            protocol = Some(s.value());
        } else if nested.path.is_ident("pathname") {
            let value = nested.value()?;
            let s: syn::LitStr = value.parse()?;
            pathname = Some(s.value());
        } else if nested.path.is_ident("description") {
            let value = nested.value()?;
            let s: syn::LitStr = value.parse()?;
            description = Some(s.value());
        } else if nested.path.is_ident("variable") {
            // Parse nested variable(...) attribute
            if let Some(var) = extract_server_variable(&nested) {
                variables.push(var);
            }
        }
        Ok(())
    });

    // Require name, host, and protocol
    Some(ServerMeta {
        name: name?,
        host: host?,
        protocol: protocol?,
        pathname,
        description,
        variables,
    })
}

/// Extract server variable from nested meta (called from within parse_nested_meta)
fn extract_server_variable(nested: &syn::meta::ParseNestedMeta) -> Option<ServerVariableMeta> {
    let mut name = None;
    let mut description = None;
    let mut default = None;
    let mut enum_values = Vec::new();
    let mut examples = Vec::new();

    let _ = nested.parse_nested_meta(|inner| {
        if inner.path.is_ident("name") {
            let value = inner.value()?;
            let s: syn::LitStr = value.parse()?;
            name = Some(s.value());
        } else if inner.path.is_ident("description") {
            let value = inner.value()?;
            let s: syn::LitStr = value.parse()?;
            description = Some(s.value());
        } else if inner.path.is_ident("default") {
            let value = inner.value()?;
            let s: syn::LitStr = value.parse()?;
            default = Some(s.value());
        } else if inner.path.is_ident("enum_values") {
            // Parse array of strings: enum_values = ["val1", "val2"]
            let _ = inner.value()?; // Consume the equals sign
            let content;
            syn::bracketed!(content in inner.input);
            let values: syn::punctuated::Punctuated<syn::LitStr, syn::Token![,]> =
                content.parse_terminated(|stream| stream.parse(), syn::Token![,])?;
            enum_values = values.iter().map(|lit| lit.value()).collect();
        } else if inner.path.is_ident("examples") {
            // Parse array of strings: examples = ["val1", "val2"]
            let _ = inner.value()?; // Consume the equals sign
            let content;
            syn::bracketed!(content in inner.input);
            let values: syn::punctuated::Punctuated<syn::LitStr, syn::Token![,]> =
                content.parse_terminated(|stream| stream.parse(), syn::Token![,])?;
            examples = values.iter().map(|lit| lit.value()).collect();
        }
        Ok(())
    });

    Some(ServerVariableMeta {
        name: name?,
        description,
        default,
        enum_values,
        examples,
    })
}

/// Extract channel metadata from `#[asyncapi_channel(...)]` attribute
fn extract_channel(attr: &Attribute) -> Option<ChannelMeta> {
    let mut name = None;
    let mut address = None;
    let mut description = None;
    let mut parameters = Vec::new();

    let _ = attr.parse_nested_meta(|nested| {
        if nested.path.is_ident("name") {
            let value = nested.value()?;
            let s: syn::LitStr = value.parse()?;
            name = Some(s.value());
        } else if nested.path.is_ident("address") {
            let value = nested.value()?;
            let s: syn::LitStr = value.parse()?;
            address = Some(s.value());
        } else if nested.path.is_ident("description") {
            let value = nested.value()?;
            let s: syn::LitStr = value.parse()?;
            description = Some(s.value());
        } else if nested.path.is_ident("parameter") {
            // Parse nested parameter(...) attribute
            if let Some(param) = extract_channel_parameter(&nested) {
                parameters.push(param);
            }
        }
        Ok(())
    });

    // Require name
    Some(ChannelMeta {
        name: name?,
        address,
        description,
        parameters,
    })
}

/// Extract channel parameter from nested meta (called from within parse_nested_meta)
fn extract_channel_parameter(nested: &syn::meta::ParseNestedMeta) -> Option<ParameterMeta> {
    let mut name = None;
    let mut description = None;
    let mut schema_type = None;
    let mut format = None;

    let _ = nested.parse_nested_meta(|inner| {
        if inner.path.is_ident("name") {
            let value = inner.value()?;
            let s: syn::LitStr = value.parse()?;
            name = Some(s.value());
        } else if inner.path.is_ident("description") {
            let value = inner.value()?;
            let s: syn::LitStr = value.parse()?;
            description = Some(s.value());
        } else if inner.path.is_ident("schema_type") {
            let value = inner.value()?;
            let s: syn::LitStr = value.parse()?;
            schema_type = Some(s.value());
        } else if inner.path.is_ident("format") {
            let value = inner.value()?;
            let s: syn::LitStr = value.parse()?;
            format = Some(s.value());
        }
        Ok(())
    });

    Some(ParameterMeta {
        name: name?,
        description,
        schema_type,
        format,
    })
}

/// Extract operation metadata from `#[asyncapi_operation(...)]` attribute
fn extract_operation(attr: &Attribute) -> Option<OperationMeta> {
    use syn::Token;
    use syn::punctuated::Punctuated;

    let mut name = None;
    let mut action = None;
    let mut channel = None;
    let mut description = None;
    let mut messages = Vec::new();

    let _ = attr.parse_nested_meta(|nested| {
        if nested.path.is_ident("name") {
            let value = nested.value()?;
            let s: syn::LitStr = value.parse()?;
            name = Some(s.value());
        } else if nested.path.is_ident("action") {
            let value = nested.value()?;
            let s: syn::LitStr = value.parse()?;
            action = Some(s.value());
        } else if nested.path.is_ident("channel") {
            let value = nested.value()?;
            let s: syn::LitStr = value.parse()?;
            channel = Some(s.value());
        } else if nested.path.is_ident("description") {
            let value = nested.value()?;
            let s: syn::LitStr = value.parse()?;
            description = Some(s.value());
        } else if nested.path.is_ident("messages") {
            // Parse array of type paths: messages = [Type1, Type2, ...]
            let _ = nested.value()?; // Parse the equals sign and prepare for value parsing
            let content;
            syn::bracketed!(content in nested.input);
            let types: Punctuated<Path, Token![,]> =
                content.parse_terminated(|stream| stream.parse(), Token![,])?;
            messages = types.into_iter().collect();
        }
        Ok(())
    });

    // Require name, action, and channel
    Some(OperationMeta {
        name: name?,
        action: action?,
        channel: channel?,
        description,
        messages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_extract_title_and_version() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi(title = "Chat API", version = "1.0.0")]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.title, Some("Chat API".to_string()));
        assert_eq!(meta.version, Some("1.0.0".to_string()));
        assert_eq!(meta.description, None);
    }

    #[test]
    fn test_extract_with_description() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi(
                title = "My API",
                version = "2.0.0",
                description = "A great API"
            )]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.title, Some("My API".to_string()));
        assert_eq!(meta.version, Some("2.0.0".to_string()));
        assert_eq!(meta.description, Some("A great API".to_string()));
    }

    #[test]
    fn test_extract_none() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[derive(Debug)]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.title, None);
        assert_eq!(meta.version, None);
        assert_eq!(meta.description, None);
    }

    #[test]
    fn test_extract_server() {
        let attrs: Vec<Attribute> = vec![
            parse_quote! { #[asyncapi(title = "API", version = "1.0.0")] },
            parse_quote! { #[asyncapi_server(name = "production", host = "api.example.com", protocol = "wss")] },
        ];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.servers.len(), 1);
        assert_eq!(meta.servers[0].name, "production");
        assert_eq!(meta.servers[0].host, "api.example.com");
        assert_eq!(meta.servers[0].protocol, "wss");
        assert_eq!(meta.servers[0].description, None);
    }

    #[test]
    fn test_extract_server_with_description() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_server(
                name = "dev",
                host = "localhost:8080",
                protocol = "ws",
                description = "Development server"
            )]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.servers.len(), 1);
        assert_eq!(
            meta.servers[0].description,
            Some("Development server".to_string())
        );
    }

    #[test]
    fn test_extract_channel() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_channel(name = "chat", address = "/ws/chat")]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.channels.len(), 1);
        assert_eq!(meta.channels[0].name, "chat");
        assert_eq!(meta.channels[0].address, Some("/ws/chat".to_string()));
    }

    #[test]
    fn test_extract_operation() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_operation(name = "sendMessage", action = "send", channel = "chat")]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.operations.len(), 1);
        assert_eq!(meta.operations[0].name, "sendMessage");
        assert_eq!(meta.operations[0].action, "send");
        assert_eq!(meta.operations[0].channel, "chat");
    }

    #[test]
    fn test_extract_multiple_components() {
        let attrs: Vec<Attribute> = vec![
            parse_quote! { #[asyncapi(title = "Chat API", version = "1.0.0")] },
            parse_quote! { #[asyncapi_server(name = "prod", host = "api.example.com", protocol = "wss")] },
            parse_quote! { #[asyncapi_channel(name = "chat", address = "/ws/chat")] },
            parse_quote! { #[asyncapi_operation(name = "send", action = "send", channel = "chat")] },
            parse_quote! { #[asyncapi_operation(name = "receive", action = "receive", channel = "chat")] },
        ];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.title, Some("Chat API".to_string()));
        assert_eq!(meta.servers.len(), 1);
        assert_eq!(meta.channels.len(), 1);
        assert_eq!(meta.operations.len(), 2);
    }

    #[test]
    fn test_extract_message_types() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_messages(ChatMessage, UserMessage, SystemMessage)]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.message_types.len(), 3);
        let path0 = &meta.message_types[0];
        let path1 = &meta.message_types[1];
        let path2 = &meta.message_types[2];
        assert_eq!(quote!(#path0).to_string(), "ChatMessage");
        assert_eq!(quote!(#path1).to_string(), "UserMessage");
        assert_eq!(quote!(#path2).to_string(), "SystemMessage");
    }

    #[test]
    fn test_extract_single_message_type() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_messages(ChatMessage)]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.message_types.len(), 1);
        let path0 = &meta.message_types[0];
        assert_eq!(quote!(#path0).to_string(), "ChatMessage");
    }

    #[test]
    fn test_extract_message_types_with_module_paths() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_messages(super::messages::Operation, crate::OperationResponse)]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.message_types.len(), 2);
        let path0 = &meta.message_types[0];
        let path1 = &meta.message_types[1];
        assert_eq!(quote!(#path0).to_string(), "super :: messages :: Operation");
        assert_eq!(quote!(#path1).to_string(), "crate :: OperationResponse");
    }

    #[test]
    fn test_extract_server_with_variables() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_server(
                name = "production",
                host = "api.enlightenhq.com",
                protocol = "wss",
                pathname = "/api/ws/{userId}",
                variable(name = "userId", description = "Authenticated user ID", examples = ["12", "13"])
            )]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.servers.len(), 1);
        let server = &meta.servers[0];
        assert_eq!(server.name, "production");
        assert_eq!(server.host, "api.enlightenhq.com");
        assert_eq!(server.protocol, "wss");
        assert_eq!(server.pathname, Some("/api/ws/{userId}".to_string()));

        assert_eq!(server.variables.len(), 1);
        let var = &server.variables[0];
        assert_eq!(var.name, "userId");
        assert_eq!(var.description, Some("Authenticated user ID".to_string()));
        assert_eq!(var.examples, vec!["12".to_string(), "13".to_string()]);
    }

    #[test]
    fn test_extract_server_with_multiple_variables() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_server(
                name = "staging",
                host = "staging.example.com",
                protocol = "wss",
                pathname = "/api/{version}/ws/{userId}",
                variable(name = "version", description = "API version", enum_values = ["v1", "v2"], default = "v2"),
                variable(name = "userId", description = "User ID", examples = ["12", "13"])
            )]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.servers.len(), 1);
        let server = &meta.servers[0];
        assert_eq!(server.variables.len(), 2);

        let var0 = &server.variables[0];
        assert_eq!(var0.name, "version");
        assert_eq!(var0.enum_values, vec!["v1".to_string(), "v2".to_string()]);
        assert_eq!(var0.default, Some("v2".to_string()));

        let var1 = &server.variables[1];
        assert_eq!(var1.name, "userId");
        assert_eq!(var1.examples, vec!["12".to_string(), "13".to_string()]);
    }

    #[test]
    fn test_extract_channel_with_parameters() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_channel(
                name = "rtMessaging",
                address = "/api/ws/{userId}",
                parameter(name = "userId", description = "User ID for this WebSocket connection", schema_type = "integer", format = "int64")
            )]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.channels.len(), 1);
        let channel = &meta.channels[0];
        assert_eq!(channel.name, "rtMessaging");
        assert_eq!(channel.address, Some("/api/ws/{userId}".to_string()));

        assert_eq!(channel.parameters.len(), 1);
        let param = &channel.parameters[0];
        assert_eq!(param.name, "userId");
        assert_eq!(
            param.description,
            Some("User ID for this WebSocket connection".to_string())
        );
        assert_eq!(param.schema_type, Some("integer".to_string()));
        assert_eq!(param.format, Some("int64".to_string()));
    }

    #[test]
    fn test_extract_channel_with_multiple_parameters() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_channel(
                name = "userChannel",
                address = "/api/{version}/ws/{userId}",
                parameter(name = "version", description = "API version", schema_type = "string"),
                parameter(name = "userId", description = "User ID", schema_type = "integer", format = "int64")
            )]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.channels.len(), 1);
        let channel = &meta.channels[0];
        assert_eq!(channel.parameters.len(), 2);

        let param0 = &channel.parameters[0];
        assert_eq!(param0.name, "version");
        assert_eq!(param0.schema_type, Some("string".to_string()));
        assert_eq!(param0.format, None);

        let param1 = &channel.parameters[1];
        assert_eq!(param1.name, "userId");
        assert_eq!(param1.schema_type, Some("integer".to_string()));
        assert_eq!(param1.format, Some("int64".to_string()));
    }

    #[test]
    fn test_extract_operation_with_messages() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_operation(name = "sendMessage", action = "send", channel = "chat", messages = [ChatMessage])]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.operations.len(), 1);
        assert_eq!(meta.operations[0].name, "sendMessage");
        assert_eq!(meta.operations[0].action, "send");
        assert_eq!(meta.operations[0].channel, "chat");
        assert_eq!(meta.operations[0].messages.len(), 1);
        let path0 = &meta.operations[0].messages[0];
        assert_eq!(quote!(#path0).to_string(), "ChatMessage");
    }

    #[test]
    fn test_extract_operation_with_multiple_messages() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_operation(name = "sendMessage", action = "send", channel = "chat", messages = [ChatMessage, SystemMessage])]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.operations.len(), 1);
        assert_eq!(meta.operations[0].messages.len(), 2);
        let path0 = &meta.operations[0].messages[0];
        let path1 = &meta.operations[0].messages[1];
        assert_eq!(quote!(#path0).to_string(), "ChatMessage");
        assert_eq!(quote!(#path1).to_string(), "SystemMessage");
    }

    #[test]
    fn test_extract_operation_with_module_path_messages() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_operation(name = "sendMessage", action = "send", channel = "chat", messages = [super::messages::ChatMessage, crate::SystemMessage])]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.operations.len(), 1);
        assert_eq!(meta.operations[0].messages.len(), 2);
        let path0 = &meta.operations[0].messages[0];
        let path1 = &meta.operations[0].messages[1];
        assert_eq!(
            quote!(#path0).to_string(),
            "super :: messages :: ChatMessage"
        );
        assert_eq!(quote!(#path1).to_string(), "crate :: SystemMessage");
    }
}
