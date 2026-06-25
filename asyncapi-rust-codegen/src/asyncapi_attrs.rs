//! Utilities for parsing asyncapi attributes

use syn::{Attribute, Meta, Path};

#[derive(Clone, Debug)]
pub enum ResponseTopic {
    Reference(Path),
    Uri(String),
}

impl Default for ResponseTopic {
    fn default() -> Self {
        Self::Uri(String::default())
    }
}

#[derive(Clone, Debug, Default)]
pub struct MqttMessageBinding {
    pub payload_format_indicator: Option<u8>,
    pub correlation_data: Option<Path>,
    pub content_type: Option<String>,
    pub response_topic: Option<ResponseTopic>,
    pub binding_version: Option<String>,
}

/// AsyncAPI metadata extracted from attributes
#[derive(Debug, Default, Clone)]
pub struct AsyncApiMeta {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub title: Option<String>,
    pub content_type: Option<String>,
    pub triggers_binary: bool,
    /// Override the message name used in `components.messages` and `asyncapi_message_names()`.
    /// When absent the Rust variant/type identifier is used.
    pub message_name: Option<String>,
    pub mqtt: Option<MqttMessageBinding>,
}

fn parse_response_topic(meta: Meta) -> Option<ResponseTopic> {
    match meta {
        Meta::Path(path) => Some(ResponseTopic::Reference(path)),

        Meta::NameValue(nv) => {
            match nv.value {
                syn::Expr::Lit(expr_lit) => {
                    match expr_lit.lit {
                        syn::Lit::Str(s) => Some(ResponseTopic::Uri(s.value())),
                        _ => None, // invalid literal → ignore
                    }
                }
                _ => None,
            }
        }

        _ => None,
    }
}

/// Extract asyncapi metadata from `#[asyncapi(...)]` attributes
pub fn extract_asyncapi_meta(attrs: &[Attribute]) -> AsyncApiMeta {
    let mut meta = AsyncApiMeta::default();

    for attr in attrs {
        if !attr.path().is_ident("asyncapi") {
            continue;
        }

        let _ = attr.parse_nested_meta(|nested| {
            if nested.path.is_ident("summary") {
                let value = nested.value()?;
                let s: syn::LitStr = value.parse()?;
                meta.summary = Some(s.value());
            } else if nested.path.is_ident("description") {
                let value = nested.value()?;
                let s: syn::LitStr = value.parse()?;
                meta.description = Some(s.value());
            } else if nested.path.is_ident("title") {
                let value = nested.value()?;
                let s: syn::LitStr = value.parse()?;
                meta.title = Some(s.value());
            } else if nested.path.is_ident("content_type") {
                let value = nested.value()?;
                let s: syn::LitStr = value.parse()?;
                meta.content_type = Some(s.value());
            } else if nested.path.is_ident("triggers_binary") {
                // Flag attribute (no value)
                meta.triggers_binary = true;
            } else if nested.path.is_ident("message_name") {
                let value = nested.value()?;
                let s: syn::LitStr = value.parse()?;
                meta.message_name = Some(s.value());
            } else if nested.path.is_ident("mqtt") {
                let mut binding = MqttMessageBinding::default();

                let value = nested.value()?;
                let mqtt_meta: syn::MetaList = value.parse()?;

                mqtt_meta.parse_nested_meta(|m| {
                    if m.path.is_ident("payload_format_indicator") {
                        let v = m.value()?;
                        let lit: syn::LitInt = v.parse()?;
                        binding.payload_format_indicator = Some(lit.base10_parse::<u8>()?);
                    } else if m.path.is_ident("correlation_data") {
                        let v = m.value()?;
                        let s: syn::Path = v.parse()?;

                        binding.correlation_data = Some(s);
                    } else if m.path.is_ident("content_type") {
                        let v = m.value()?;
                        let s: syn::LitStr = v.parse()?;
                        binding.content_type = Some(s.value());
                    } else if m.path.is_ident("response_topic") {
                        let v = m.value()?;
                        let meta_inner: syn::Meta = v.parse()?;

                        binding.response_topic = parse_response_topic(meta_inner);
                    } else if m.path.is_ident("binding_version") {
                        let v = m.value()?;
                        let s: syn::LitStr = v.parse()?;
                        binding.binding_version = Some(s.value());
                    }

                    Ok(())
                })?;

                meta.mqtt = Some(binding);
            }
            Ok(())
        });
    }

    meta
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_extract_summary() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi(summary = "Send a message")]
        }];

        let meta = extract_asyncapi_meta(&attrs);
        assert_eq!(meta.summary, Some("Send a message".to_string()));
        assert_eq!(meta.description, None);
    }

    #[test]
    fn test_extract_multiple() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi(summary = "Send message", description = "Sends a chat message to a room")]
        }];

        let meta = extract_asyncapi_meta(&attrs);
        assert_eq!(meta.summary, Some("Send message".to_string()));
        assert_eq!(
            meta.description,
            Some("Sends a chat message to a room".to_string())
        );
    }

    #[test]
    fn test_extract_content_type() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi(content_type = "application/octet-stream")]
        }];

        let meta = extract_asyncapi_meta(&attrs);
        assert_eq!(
            meta.content_type,
            Some("application/octet-stream".to_string())
        );
    }

    #[test]
    fn test_extract_none() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[derive(Debug)]
        }];

        let meta = extract_asyncapi_meta(&attrs);
        assert_eq!(meta.summary, None);
        assert_eq!(meta.description, None);
    }

    #[test]
    fn test_extract_triggers_binary() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi(triggers_binary)]
        }];

        let meta = extract_asyncapi_meta(&attrs);
        assert!(meta.triggers_binary);
        assert_eq!(meta.content_type, None);
    }

    #[test]
    fn test_extract_title() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi(title = "My Message")]
        }];

        let meta = extract_asyncapi_meta(&attrs);
        assert_eq!(meta.title, Some("My Message".to_string()));
    }

    #[test]
    fn test_extract_message_name() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[asyncapi(message_name = "CurrentEditorResponse")]
        }];

        let meta = extract_asyncapi_meta(&attrs);
        assert_eq!(meta.message_name, Some("CurrentEditorResponse".to_string()));
    }
}
