//! Utilities for parsing asyncapi spec-level attributes

use syn::Attribute;

/// AsyncAPI spec metadata extracted from attributes
#[derive(Debug, Default, Clone)]
pub struct AsyncApiSpecMeta {
    pub title: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub servers: Vec<ServerMeta>,
    pub channels: Vec<ChannelMeta>,
    pub operations: Vec<OperationMeta>,
}

/// Server metadata
#[derive(Debug, Clone)]
pub struct ServerMeta {
    pub name: String,
    pub host: String,
    pub protocol: String,
    pub description: Option<String>,
}

/// Channel metadata
#[derive(Debug, Clone)]
pub struct ChannelMeta {
    pub name: String,
    pub address: Option<String>,
    #[allow(dead_code)] // Reserved for future use
    pub description: Option<String>,
}

/// Operation metadata
#[derive(Debug, Clone)]
pub struct OperationMeta {
    pub name: String,
    pub action: String, // "send" or "receive"
    pub channel: String,
    #[allow(dead_code)] // Reserved for future use
    pub description: Option<String>,
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
        }
    }

    meta
}

/// Extract server metadata from `#[asyncapi_server(...)]` attribute
fn extract_server(attr: &Attribute) -> Option<ServerMeta> {
    let mut name = None;
    let mut host = None;
    let mut protocol = None;
    let mut description = None;

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
        } else if nested.path.is_ident("description") {
            let value = nested.value()?;
            let s: syn::LitStr = value.parse()?;
            description = Some(s.value());
        }
        Ok(())
    });

    // Require name, host, and protocol
    Some(ServerMeta {
        name: name?,
        host: host?,
        protocol: protocol?,
        description,
    })
}

/// Extract channel metadata from `#[asyncapi_channel(...)]` attribute
fn extract_channel(attr: &Attribute) -> Option<ChannelMeta> {
    let mut name = None;
    let mut address = None;
    let mut description = None;

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
        }
        Ok(())
    });

    // Require name
    Some(ChannelMeta {
        name: name?,
        address,
        description,
    })
}

/// Extract operation metadata from `#[asyncapi_operation(...)]` attribute
fn extract_operation(attr: &Attribute) -> Option<OperationMeta> {
    let mut name = None;
    let mut action = None;
    let mut channel = None;
    let mut description = None;

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
        }
        Ok(())
    });

    // Require name, action, and channel
    Some(OperationMeta {
        name: name?,
        action: action?,
        channel: channel?,
        description,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(meta.servers[0].description, Some("Development server".to_string()));
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
}
