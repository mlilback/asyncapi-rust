//! Utilities for parsing asyncapi spec-level attributes

use syn::Attribute;

/// AsyncAPI spec metadata extracted from attributes
#[derive(Debug, Default, Clone)]
pub struct AsyncApiSpecMeta {
    pub title: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
}

/// Extract asyncapi spec metadata from `#[asyncapi(...)]` attributes
pub fn extract_asyncapi_spec_meta(attrs: &[Attribute]) -> AsyncApiSpecMeta {
    let mut meta = AsyncApiSpecMeta::default();

    for attr in attrs {
        if !attr.path().is_ident("asyncapi") {
            continue;
        }

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
    }

    meta
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
}
