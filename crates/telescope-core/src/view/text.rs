//! String helpers that mirror JavaScript semantics the frontend relied on.

use std::cmp::Ordering;

/// Characters stripped by `String.prototype.trim` and matched by `\s`.
pub(crate) fn is_js_whitespace(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{FEFF}'
}

pub(crate) fn js_trim(s: &str) -> &str {
    s.trim_matches(is_js_whitespace)
}

/// Case-insensitive comparison, like `Intl.Collator` with
/// `sensitivity: 'base'` for the strings EVE names consist of.
pub(crate) fn compare_base(a: &str, b: &str) -> Ordering {
    a.chars()
        .flat_map(char::to_lowercase)
        .cmp(b.chars().flat_map(char::to_lowercase))
}

/// Approximates `localeCompare`: case-insensitive first, then lowercase
/// before uppercase so the order is total and deterministic.
pub(crate) fn compare_locale(a: &str, b: &str) -> Ordering {
    compare_base(a, b).then_with(|| {
        let flip = |c: char| {
            if c.is_lowercase() {
                c.to_uppercase().next().unwrap_or(c)
            } else {
                c.to_lowercase().next().unwrap_or(c)
            }
        };
        a.chars().map(flip).cmp(b.chars().map(flip))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_trim_strips_bom_and_whitespace() {
        assert_eq!(js_trim("\u{FEFF}  name \t"), "name");
    }

    #[test]
    fn compare_base_ignores_case() {
        assert_eq!(compare_base("alice", "ALICE"), Ordering::Equal);
        assert_eq!(compare_base("alice", "Bob"), Ordering::Less);
    }

    #[test]
    fn compare_locale_puts_lowercase_first() {
        assert_eq!(compare_locale("a", "A"), Ordering::Less);
        assert_eq!(compare_locale("B", "a"), Ordering::Greater);
        assert_eq!(compare_locale("Atron", "Drake"), Ordering::Less);
    }
}
