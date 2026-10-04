//! Display labels for zKillboard-derived pilot flags.

use crate::models::PilotFlags;

pub const SUPER: &str = "SUPER";
pub const CAPITAL: &str = "CAPITAL";
pub const BLACK_OPS: &str = "BLACK OPS";
pub const RECON: &str = "RECON";
pub const CYNO: &str = "CYNO";
pub const SOLO: &str = "SOLO";

/// SUPER implies CAPITAL, so only the stronger label is shown.
pub fn flag_labels(flags: &PilotFlags) -> Vec<&'static str> {
    let mut labels = Vec::new();

    if flags.is_super {
        labels.push(SUPER);
    } else if flags.is_capital {
        labels.push(CAPITAL);
    }
    if flags.is_blops {
        labels.push(BLACK_OPS);
    }
    if flags.is_recon {
        labels.push(RECON);
    }
    if flags.is_cyno {
        labels.push(CYNO);
    }
    if flags.is_solo {
        labels.push(SOLO);
    }

    labels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_for_no_flags() {
        assert!(flag_labels(&PilotFlags::default()).is_empty());
    }

    #[test]
    fn super_excludes_capital() {
        let labels = flag_labels(&PilotFlags {
            is_super: true,
            is_capital: true,
            ..Default::default()
        });
        assert!(labels.contains(&SUPER));
        assert!(!labels.contains(&CAPITAL));
    }

    #[test]
    fn capital_when_not_super() {
        let labels = flag_labels(&PilotFlags {
            is_capital: true,
            ..Default::default()
        });
        assert!(labels.contains(&CAPITAL));
    }

    #[test]
    fn multiple_flags_in_order() {
        let labels = flag_labels(&PilotFlags {
            is_cyno: true,
            is_recon: true,
            is_solo: true,
            ..Default::default()
        });
        assert_eq!(labels, vec![RECON, CYNO, SOLO]);
    }

    #[test]
    fn black_ops() {
        let labels = flag_labels(&PilotFlags {
            is_blops: true,
            ..Default::default()
        });
        assert_eq!(labels, vec!["BLACK OPS"]);
    }
}
