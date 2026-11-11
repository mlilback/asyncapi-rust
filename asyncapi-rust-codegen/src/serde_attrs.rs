//! Utilities for parsing serde attributes

use syn::Attribute;
use syn::meta::ParseNestedMeta;

/// Consume the value of a serde key we don't read so parsing reaches the following keys
fn skip_value(meta: &ParseNestedMeta) -> syn::Result<()> {
    if meta.input.peek(syn::Token![=]) {
        meta.value()?.parse::<syn::Expr>()?;
    } else if meta.input.peek(syn::token::Paren) {
        meta.parse_nested_meta(|nested| skip_value(&nested))?;
    }
    Ok(())
}

/// Extract the value from `#[serde(rename = "...")]`
pub fn extract_serde_rename(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        let mut rename_value = None;

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let value = meta.value()?;
                let s: syn::LitStr = value.parse()?;
                rename_value = Some(s.value());
            } else {
                skip_value(&meta)?;
            }
            Ok(())
        });

        if rename_value.is_some() {
            return rename_value;
        }
    }
    None
}

/// Extract the value from `#[serde(rename_all = "...")]`
pub fn extract_serde_rename_all(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        let mut rule = None;

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                let value = meta.value()?;
                let s: syn::LitStr = value.parse()?;
                rule = Some(s.value());
            } else {
                skip_value(&meta)?;
            }
            Ok(())
        });

        if rule.is_some() {
            return rule;
        }
    }
    None
}

/// Apply a serde `rename_all` rule to a variant identifier, matching serde's own rules
pub fn apply_rename_all(rule: &str, variant: &str) -> Option<String> {
    let snake = || {
        let mut out = String::new();
        for (i, ch) in variant.char_indices() {
            if i > 0 && ch.is_uppercase() {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        }
        out
    };

    Some(match rule {
        "lowercase" => variant.to_ascii_lowercase(),
        "UPPERCASE" => variant.to_ascii_uppercase(),
        "PascalCase" => variant.to_string(),
        "camelCase" => variant[..1].to_ascii_lowercase() + &variant[1..],
        "snake_case" => snake(),
        "SCREAMING_SNAKE_CASE" => snake().to_ascii_uppercase(),
        "kebab-case" => snake().replace('_', "-"),
        "SCREAMING-KEBAB-CASE" => snake().to_ascii_uppercase().replace('_', "-"),
        _ => return None,
    })
}

/// Extract the value from `#[serde(tag = "...")]`
pub fn extract_serde_tag(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        let mut tag_value = None;

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("tag") {
                let value = meta.value()?;
                let s: syn::LitStr = value.parse()?;
                tag_value = Some(s.value());
            } else {
                skip_value(&meta)?;
            }
            Ok(())
        });

        if tag_value.is_some() {
            return tag_value;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_extract_serde_rename() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[serde(rename = "custom_name")]
        }];

        assert_eq!(
            extract_serde_rename(&attrs),
            Some("custom_name".to_string())
        );
    }

    #[test]
    fn test_extract_serde_rename_none() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[derive(Debug)]
        }];

        assert_eq!(extract_serde_rename(&attrs), None);
    }

    #[test]
    fn test_extract_serde_tag() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[serde(tag = "type")]
        }];

        assert_eq!(extract_serde_tag(&attrs), Some("type".to_string()));
    }

    #[test]
    fn test_extract_serde_tag_none() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[serde(rename = "foo")]
        }];

        assert_eq!(extract_serde_tag(&attrs), None);
    }

    #[test]
    fn test_extract_serde_tag_after_other_keys() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[serde(rename_all = "snake_case", tag = "type")]
        }];

        assert_eq!(extract_serde_tag(&attrs), Some("type".to_string()));
    }

    #[test]
    fn test_extract_serde_rename_all_after_other_keys() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[serde(tag = "type", rename_all = "snake_case")]
        }];

        assert_eq!(
            extract_serde_rename_all(&attrs),
            Some("snake_case".to_string())
        );
    }

    #[test]
    fn test_apply_rename_all() {
        let cases = [
            ("lowercase", "nexttrack"),
            ("UPPERCASE", "NEXTTRACK"),
            ("PascalCase", "NextTrack"),
            ("camelCase", "nextTrack"),
            ("snake_case", "next_track"),
            ("SCREAMING_SNAKE_CASE", "NEXT_TRACK"),
            ("kebab-case", "next-track"),
            ("SCREAMING-KEBAB-CASE", "NEXT-TRACK"),
        ];

        for (rule, expected) in cases {
            assert_eq!(
                apply_rename_all(rule, "NextTrack").as_deref(),
                Some(expected)
            );
        }
        assert_eq!(apply_rename_all("unknown", "NextTrack"), None);
    }
}
