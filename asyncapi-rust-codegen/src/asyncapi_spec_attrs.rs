//! Utilities for parsing asyncapi spec-level attributes

use syn::{Attribute, LitStr, Path};

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
    pub mqtt: Option<MqttServerBindingsMeta>,
}

#[derive(Debug, Clone)]
pub struct LastWillMeta {
    pub topic: String,
    pub qos: u8,
    pub retain: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct MqttServerBindingsMeta {
    pub client_id: Option<String>,
    pub clean_session: Option<bool>,
    pub last_will: Option<LastWillMeta>,
    pub keep_alive: Option<u32>,
    pub session_expiry_interval: Option<u32>,
    #[allow(unused)]
    pub maximum_packet_size: Option<Path>,
    pub binding_version: Option<String>,
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
    pub default: Option<String>,
    pub enum_values: Vec<String>,
    pub examples: Vec<String>,
    pub location: Option<String>,
}

/// Operation metadata
#[derive(Debug, Clone)]
pub struct OperationMeta {
    pub name: String,
    pub action: String, // "send" or "receive"
    pub channel: String,
    #[allow(dead_code)] // Reserved for future use
    pub description: Option<String>,
    pub mqtt: Option<OperationMqttBindingsMeta>,
}

#[derive(Debug, Clone)]
pub struct OperationMqttBindingsMeta {
    pub qos: Option<u8>,
    pub retain: Option<bool>,
    pub message_expiry_interval: Option<u32>,
    pub binding_version: Option<String>,
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
    let mut mqtt = None;

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
        } else if nested.path.is_ident("mqtt") {
            if let Some(var) = extract_mqtt_server_bindings(&nested) {
                mqtt = Some(var);
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
        mqtt,
    })
}

fn extract_mqtt_server_bindings(
    nested: &syn::meta::ParseNestedMeta,
) -> Option<MqttServerBindingsMeta> {
    let mut client_id: Option<String> = None;
    let mut clean_session: Option<bool> = None;
    let mut keep_alive = None;
    let mut session_expiry_interval = None;
    let mut maximum_packet_size = None;
    let mut binding_version: Option<String> = None;

    let mut last_will: Option<LastWillMeta> = None;

    let _ = nested.parse_nested_meta(|inner| {
        if inner.path.is_ident("client_id") {
            let value = inner.value()?;
            let s: syn::LitStr = value.parse()?;
            client_id = Some(s.value());
        } else if inner.path.is_ident("clean_session") {
            let value = inner.value()?;
            let s: syn::LitBool = value.parse()?;
            clean_session = Some(s.value());
        } else if inner.path.is_ident("keep_alive") {
            let value = inner.value()?;
            let s: syn::LitInt = value.parse()?;
            keep_alive = Some(s.base10_parse()?);
        } else if inner.path.is_ident("session_expiry_interval") {
            let value = inner.value()?;
            let p: syn::LitInt = value.parse()?;
            session_expiry_interval = Some(p.base10_parse()?);
        } else if inner.path.is_ident("maximum_packet_size") {
            let value = inner.value()?;
            let p: Path = value.parse()?;
            maximum_packet_size = Some(p);
        } else if inner.path.is_ident("binding_version") {
            let value = inner.value()?;
            let s: syn::LitStr = value.parse()?;
            binding_version = Some(s.value());
        } else if inner.path.is_ident("last_will") {
            let content;
            syn::parenthesized!(content in inner.input);

            let mut topic: Option<String> = None;
            let mut qos: Option<u8> = None;
            let mut message: Option<String> = None;
            let mut retain: Option<bool> = None;

            while !content.is_empty() {
                let key: syn::Ident = content.parse()?;
                content.parse::<syn::Token![:]>()?;

                match key.to_string().as_str() {
                    "topic" => {
                        let v: syn::LitStr = content.parse()?;
                        topic = Some(v.value());
                    }
                    "qos" => {
                        let v: syn::LitInt = content.parse()?;
                        qos = Some(v.base10_parse()?);
                    }
                    "message" => {
                        let v: syn::LitStr = content.parse()?;
                        message = Some(v.value());
                    }
                    "retain" => {
                        let v: syn::LitBool = content.parse()?;
                        retain = Some(v.value());
                    }
                    _ => {}
                }

                let _ = content.parse::<syn::Token![,]>();
            }

            last_will = Some(LastWillMeta {
                topic: topic
                    .ok_or_else(|| syn::Error::new_spanned(&inner.path, "missing topic"))?,
                qos: qos.ok_or_else(|| syn::Error::new_spanned(&inner.path, "missing qos"))?,
                message: message
                    .ok_or_else(|| syn::Error::new_spanned(&inner.path, "missing message"))?,
                retain: retain
                    .ok_or_else(|| syn::Error::new_spanned(&inner.path, "missing retain"))?,
            });
        }

        Ok(())
    });

    Some(MqttServerBindingsMeta {
        client_id,
        clean_session,
        binding_version,
        last_will,
        keep_alive,
        maximum_packet_size,
        session_expiry_interval,
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
    let mut default = None;
    let mut enum_values = Vec::new();
    let mut examples = Vec::new();
    let mut location = None;

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
            let _ = inner.value()?;
            let content;
            syn::bracketed!(content in inner.input);
            let values: syn::punctuated::Punctuated<syn::LitStr, syn::Token![,]> =
                content.parse_terminated(|stream| stream.parse(), syn::Token![,])?;
            enum_values = values.iter().map(|lit| lit.value()).collect();
        } else if inner.path.is_ident("examples") {
            let _ = inner.value()?;
            let content;
            syn::bracketed!(content in inner.input);
            let values: syn::punctuated::Punctuated<syn::LitStr, syn::Token![,]> =
                content.parse_terminated(|stream| stream.parse(), syn::Token![,])?;
            examples = values.iter().map(|lit| lit.value()).collect();
        } else if inner.path.is_ident("location") {
            let value = inner.value()?;
            let s: syn::LitStr = value.parse()?;
            location = Some(s.value());
        }
        Ok(())
    });

    Some(ParameterMeta {
        name: name?,
        description,
        default,
        enum_values,
        examples,
        location,
    })
}

fn extract_mqtt_operation_bindings(
    nested: &syn::meta::ParseNestedMeta,
) -> Option<OperationMqttBindingsMeta> {
    let mut qos = None;
    let mut retain = None;
    let mut message_expiry_interval = None;
    let mut binding_version = None;

    let _ = nested.parse_nested_meta(|inner| {
        if inner.path.is_ident("qos") {
            let value = inner.value()?;
            let s: syn::LitInt = value.parse()?;
            qos = Some(s.base10_parse()?);
        } else if inner.path.is_ident("retain") {
            let value = inner.value()?;
            let s: syn::LitBool = value.parse()?;
            retain = Some(s.value());
        } else if inner.path.is_ident("message_expiry_interval") {
            let value = inner.value()?;
            let p: syn::LitInt = value.parse()?;
            message_expiry_interval = Some(p.base10_parse()?);
        } else if inner.path.is_ident("binding_version") {
            let value = inner.value()?;
            let s: LitStr = value.parse()?;
            binding_version = Some(s.value());
        }
        Ok(())
    });

    Some(OperationMqttBindingsMeta {
        qos,
        retain,
        message_expiry_interval,
        binding_version,
    })
}

/// Extract operation metadata from `#[asyncapi_operation(...)]` attribute
fn extract_operation(attr: &Attribute) -> Option<OperationMeta> {
    let mut name = None;
    let mut action = None;
    let mut channel = None;
    let mut description = None;
    let mut mqtt = None;

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
        } else if nested.path.is_ident("mqtt") {
            mqtt = extract_mqtt_operation_bindings(&nested);
        }
        Ok(())
    });

    // Require name, action, and channel
    Some(OperationMeta {
        name: name?,
        action: action?,
        channel: channel?,
        description,
        mqtt,
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
    fn test_extract_operation_with_mqtt_binding() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_operation(name = "sendMessage", action = "send", channel = "chat", mqtt(qos = 1, retain = true))]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.operations.len(), 1);
        assert_eq!(meta.operations[0].name, "sendMessage");
        assert_eq!(meta.operations[0].action, "send");
        assert_eq!(meta.operations[0].channel, "chat");

        assert!(meta.operations[0].mqtt.is_some());
        let mqtt = meta.operations[0].mqtt.clone().unwrap();
        assert_eq!(mqtt.qos, Some(1));
        assert!(mqtt.retain.unwrap());
        assert!(mqtt.binding_version.is_none());
        assert!(mqtt.message_expiry_interval.is_none());
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
    fn test_extract_server_with_mqtt_bindings() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_server(
                name = "staging",
                host = "staging.example.com",
                protocol = "wss",
                pathname = "/api/{version}/ws/{userId}",
                mqtt(
                    client_id = "abc",
                    last_will(
                        topic: "a",
                        qos: 0,
                        message: "B",
                        retain: true
                    )
                )
            )]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        let server = &meta.servers[0];

        let mqtt = &server.mqtt;

        assert!(mqtt.is_some());
        let mqtt = mqtt.clone().unwrap();
        assert_eq!(mqtt.client_id, Some("abc".to_string()));
        assert!(mqtt.last_will.is_some());
        let lw = mqtt.last_will.unwrap();
        assert_eq!(lw.topic, "a".to_string());
        assert_eq!(lw.qos, 0);
        assert_eq!(lw.message, "B".to_string());
        assert!(lw.retain)
    }

    #[test]
    fn test_extract_channel_with_parameters() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_channel(
                name = "rtMessaging",
                address = "/api/ws/{userId}",
                parameter(name = "userId", description = "User ID for this WebSocket connection", examples = ["42", "100"])
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
        assert_eq!(param.examples, vec!["42".to_string(), "100".to_string()]);
    }

    #[test]
    fn test_extract_channel_with_multiple_parameters() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_channel(
                name = "userChannel",
                address = "/api/{version}/ws/{userId}",
                parameter(name = "version", description = "API version", enum_values = ["v1", "v2"], default = "v2"),
                parameter(name = "userId", description = "User ID", examples = ["42"])
            )]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.channels.len(), 1);
        let channel = &meta.channels[0];
        assert_eq!(channel.parameters.len(), 2);

        let param0 = &channel.parameters[0];
        assert_eq!(param0.name, "version");
        assert_eq!(param0.enum_values, vec!["v1".to_string(), "v2".to_string()]);
        assert_eq!(param0.default, Some("v2".to_string()));

        let param1 = &channel.parameters[1];
        assert_eq!(param1.name, "userId");
        assert_eq!(param1.examples, vec!["42".to_string()]);
    }

    #[test]
    fn test_extract_channel_with_description() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_channel(
                name = "events",
                address = "/ws/events",
                description = "Real-time event stream"
            )]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.channels.len(), 1);
        let channel = &meta.channels[0];
        assert_eq!(channel.name, "events");
        assert_eq!(
            channel.description,
            Some("Real-time event stream".to_string())
        );
    }

    #[test]
    fn test_extract_operation_with_description() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi_operation(
                name = "sendMessage",
                action = "send",
                channel = "chat",
                description = "Send a chat message"
            )]
        }];

        let meta = extract_asyncapi_spec_meta(&attrs);
        assert_eq!(meta.operations.len(), 1);
        let op = &meta.operations[0];
        assert_eq!(op.name, "sendMessage");
        assert_eq!(op.description, Some("Send a chat message".to_string()));
    }
}
