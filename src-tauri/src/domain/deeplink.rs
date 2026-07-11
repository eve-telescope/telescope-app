//! Pure telescope:// deep-link parsing. Dispatch (emitting events, applying
//! tokens, parking share codes) lives in `crate::deep_link`.

const DEEP_LINK_SCHEME: &str = "telescope";

#[derive(Debug, PartialEq, Eq)]
pub enum DeepLink {
    Auth { token: String },
    Share { code: String },
}

/// Parses a telescope:// deep link.
///
/// Tolerant of the variants browsers and URL builders produce: trailing
/// slashes (`telescope://auth/?token=X`), extra query parameters,
/// scheme/host casing, and percent-encoded tokens (decoded by the URL
/// machinery, never by hand).
pub fn parse(raw: &str) -> Option<DeepLink> {
    let parsed = url::Url::parse(raw).ok()?;
    if !parsed.scheme().eq_ignore_ascii_case(DEEP_LINK_SCHEME) {
        return None;
    }

    match parsed.host_str()?.to_ascii_lowercase().as_str() {
        // telescope://auth?token=XXX
        "auth" => parsed
            .query_pairs()
            .find(|(key, _)| key == "token")
            .map(|(_, value)| value.into_owned())
            .filter(|token| !token.is_empty())
            .map(|token| DeepLink::Auth { token }),

        // telescope://s/{code} — the alphanumeric gate keeps the code safe
        // to interpolate into the share API path.
        "s" => {
            let code = parsed.path().trim_matches('/');
            if !code.is_empty() && code.chars().all(|c| c.is_ascii_alphanumeric()) {
                Some(DeepLink::Share {
                    code: code.to_string(),
                })
            } else {
                None
            }
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_auth_link() {
        assert_eq!(
            parse("telescope://auth?token=abc123"),
            Some(DeepLink::Auth {
                token: "abc123".into()
            })
        );
    }

    #[test]
    fn parses_auth_link_with_trailing_slash() {
        assert_eq!(
            parse("telescope://auth/?token=abc123"),
            Some(DeepLink::Auth {
                token: "abc123".into()
            })
        );
    }

    #[test]
    fn extracts_token_among_extra_query_params() {
        assert_eq!(
            parse("telescope://auth?token=abc123&source=web&foo=bar"),
            Some(DeepLink::Auth {
                token: "abc123".into()
            })
        );
    }

    #[test]
    fn percent_decodes_the_token() {
        assert_eq!(
            parse("telescope://auth?token=abc%7C123%3D%3D"),
            Some(DeepLink::Auth {
                token: "abc|123==".into()
            })
        );
    }

    #[test]
    fn scheme_and_host_are_case_insensitive() {
        assert_eq!(
            parse("TELESCOPE://AUTH?token=abc"),
            Some(DeepLink::Auth {
                token: "abc".into()
            })
        );
    }

    #[test]
    fn rejects_missing_or_empty_token() {
        assert_eq!(parse("telescope://auth"), None);
        assert_eq!(parse("telescope://auth?token="), None);
        assert_eq!(parse("telescope://auth?other=x"), None);
    }

    #[test]
    fn parses_canonical_share_link() {
        assert_eq!(
            parse("telescope://s/Xy12Za"),
            Some(DeepLink::Share {
                code: "Xy12Za".into()
            })
        );
    }

    #[test]
    fn parses_share_link_with_trailing_slash() {
        assert_eq!(
            parse("telescope://s/Xy12Za/"),
            Some(DeepLink::Share {
                code: "Xy12Za".into()
            })
        );
    }

    #[test]
    fn rejects_share_codes_with_unexpected_characters() {
        assert_eq!(parse("telescope://s/code%20name"), None);
        assert_eq!(parse("telescope://s/"), None);
    }

    #[test]
    fn collapses_dot_segments_before_validation() {
        // ".." is resolved by URL parsing; the alphanumeric gate on the
        // result is what keeps the code safe.
        assert_eq!(
            parse("telescope://s/../etc"),
            Some(DeepLink::Share { code: "etc".into() })
        );
    }

    #[test]
    fn rejects_other_schemes_and_hosts() {
        assert_eq!(parse("https://auth?token=abc"), None);
        assert_eq!(parse("spyglass://auth?token=abc"), None);
        assert_eq!(parse(""), None);
        assert_eq!(parse("not a url at all"), None);
        assert_eq!(parse("telescope://unknown?token=abc"), None);
        assert_eq!(parse("telescope://"), None);
    }
}
