//! Classifies pasted scan text as a local member list or a d-scan.

use super::text::js_trim;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanInputKind {
    Local,
    Dscan,
}

/// Splits a pasted local scan into trimmed, non-empty pilot names, one per
/// line. Shared by the pilot count and the cross-fade retain set.
pub fn split_pilot_names(text: &str) -> Vec<String> {
    text.split('\n')
        .map(js_trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn detect_scan_input_kind(text: &str) -> ScanInputKind {
    let lines: Vec<&str> = text
        .split('\n')
        .map(js_trim)
        .filter(|line| !line.is_empty())
        .collect();

    if lines.is_empty() {
        return ScanInputKind::Local;
    }

    let dscan_matches = lines.iter().filter(|line| is_dscan_line(line)).count();
    if dscan_matches > 0 && dscan_matches >= (lines.len() / 2).max(1) {
        ScanInputKind::Dscan
    } else {
        ScanInputKind::Local
    }
}

/// Equivalent of `/^\d+\t.+\t.+(?:\t.+)?$/`: a numeric type id, a tab, then
/// at least two non-empty tab-separated fields.
fn is_dscan_line(line: &str) -> bool {
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return false;
    }
    let Some(rest) = line[digits..].strip_prefix('\t') else {
        return false;
    };
    if rest.contains(['\r', '\n', '\u{2028}', '\u{2029}']) {
        return false;
    }
    rest.char_indices()
        .any(|(i, c)| c == '\t' && i > 0 && i + 1 < rest.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_local_scan() {
        assert_eq!(
            detect_scan_input_kind("Pilot One\nPilot Two\nPilot Three"),
            ScanInputKind::Local
        );
    }

    #[test]
    fn detects_dscan() {
        let input = "28356\tRifter\tPilot Ship\t100 km\n670\tCapsule\tPilot Pod\t50 km";
        assert_eq!(detect_scan_input_kind(input), ScanInputKind::Dscan);
    }

    #[test]
    fn detects_local_when_mostly_names() {
        let input = "Pilot One\nPilot Two\nPilot Three\nPilot Four\n12345\tShip\tType\t10km";
        assert_eq!(detect_scan_input_kind(input), ScanInputKind::Local);
    }

    #[test]
    fn detects_dscan_when_majority_tab_separated() {
        let input = "28356\tRifter\tShip\t100 km\n670\tCapsule\tPod\t50 km\n123\tAtron\tFrig\t200 km\nRandom Name";
        assert_eq!(detect_scan_input_kind(input), ScanInputKind::Dscan);
    }

    #[test]
    fn handles_empty_input() {
        assert_eq!(detect_scan_input_kind(""), ScanInputKind::Local);
    }

    #[test]
    fn handles_single_pilot_name() {
        assert_eq!(detect_scan_input_kind("Solo Pilot"), ScanInputKind::Local);
    }

    #[test]
    fn handles_crlf_line_endings() {
        let input = "28356\tRifter\tShip\t100 km\r\n670\tCapsule\tPod\t50 km\r\n";
        assert_eq!(detect_scan_input_kind(input), ScanInputKind::Dscan);
    }

    #[test]
    fn dscan_line_shape() {
        assert!(is_dscan_line("1\ta\tb"));
        assert!(is_dscan_line("1\ta\t\tb"));
        assert!(!is_dscan_line("1\ta\t"));
        assert!(!is_dscan_line("1\t\tb"));
        assert!(!is_dscan_line("x1\ta\tb"));
        assert!(!is_dscan_line("1 a\tb\tc"));
    }

    #[test]
    fn splits_one_pilot_per_line() {
        assert_eq!(
            split_pilot_names("Pilot One\nPilot Two"),
            vec!["Pilot One", "Pilot Two"]
        );
    }

    #[test]
    fn trims_and_drops_blank_lines() {
        assert_eq!(
            split_pilot_names("  Pilot One  \n\n   \nPilot Two\n"),
            vec!["Pilot One", "Pilot Two"]
        );
    }

    #[test]
    fn split_returns_empty_for_empty_input() {
        assert!(split_pilot_names("").is_empty());
        assert!(split_pilot_names("   \n \n").is_empty());
    }
}
