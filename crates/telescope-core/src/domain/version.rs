//! Semantic-ish version comparison for the GitHub release update check.

pub fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse().ok()).collect() };

    let latest_parts = parse(latest);
    let current_parts = parse(current);

    for i in 0..latest_parts.len().max(current_parts.len()) {
        let l = latest_parts.get(i).copied().unwrap_or(0);
        let c = current_parts.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::is_newer_version;

    #[test]
    fn newer_patch_and_major_versions_detected() {
        assert!(is_newer_version("0.3.1", "0.3.0"));
        assert!(is_newer_version("1.0.0", "0.9.9"));
        assert!(!is_newer_version("0.3.0", "0.3.1"));
        assert!(!is_newer_version("0.3.0", "0.3.0"));
    }

    #[test]
    fn shorter_versions_are_zero_padded() {
        assert!(!is_newer_version("1.0", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "1.0"));
        assert!(is_newer_version("1.1", "1.0.5"));
    }

    #[test]
    fn non_numeric_and_empty_parts_are_ignored() {
        // Non-numeric segments are skipped by the parser.
        assert!(!is_newer_version("abc", "1.0.0"));
        assert!(is_newer_version("1.0.1", "abc"));
        assert!(!is_newer_version("", ""));
        assert!(!is_newer_version("", "1.0.0"));
    }
}
