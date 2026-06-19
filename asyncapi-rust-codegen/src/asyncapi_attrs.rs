//! Utilities for parsing asyncapi attributes

use syn::Attribute;

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
}
