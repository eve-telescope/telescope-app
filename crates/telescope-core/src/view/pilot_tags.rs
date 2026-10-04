//! Tag chips shown on a pilot row: zKillboard flags first, then intel
//! annotation tags, deduplicated by text.

use std::collections::HashSet;

use super::annotations::{AnnotationIndex, ResolvedAnnotation, resolve_pilot_annotations};
use super::flags::{self, flag_labels};
use crate::models::PilotIntel;

/// Fallback chip colors for tags without an explicit color.
pub const DEFAULT_TAG_COLOR: &str = "#94A3B8";
pub const DEFAULT_TAG_TEXT_COLOR: &str = "#CBD5E1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PilotTag {
    /// Stable element id: `flag:{label}` or `{scope}:{annotation id}:{tag}`.
    pub key: String,
    pub text: String,
    pub color: Option<String>,
}

pub fn flag_color(label: &str) -> Option<&'static str> {
    match label {
        flags::SUPER => Some("#F43F5E"),
        flags::CAPITAL => Some("#F59E0B"),
        flags::BLACK_OPS => Some("#6366F1"),
        flags::RECON => Some("#14B8A6"),
        flags::CYNO => Some("#A855F7"),
        flags::SOLO => Some("#38BDF8"),
        _ => None,
    }
}

pub fn pilot_tags(pilot: &PilotIntel, resolved: &[ResolvedAnnotation<'_>]) -> Vec<PilotTag> {
    let mut tags = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();

    for flag in flag_labels(&pilot.flags) {
        seen.insert(flag);
        tags.push(PilotTag {
            key: format!("flag:{flag}"),
            text: flag.to_string(),
            color: flag_color(flag).map(str::to_string),
        });
    }

    for matched in resolved {
        for tag in &matched.annotation.tags {
            if seen.insert(tag.as_str()) {
                tags.push(PilotTag {
                    key: format!("{}:{tag}", matched.key),
                    text: tag.clone(),
                    color: matched.annotation.color.clone(),
                });
            }
        }
    }

    tags
}

pub fn pilot_tags_with_index(pilot: &PilotIntel, index: &AnnotationIndex) -> Vec<PilotTag> {
    pilot_tags(pilot, &resolve_pilot_annotations(pilot, index))
}

pub fn pilot_tag_strings(pilot: &PilotIntel, resolved: &[ResolvedAnnotation<'_>]) -> Vec<String> {
    pilot_tags(pilot, resolved)
        .into_iter()
        .map(|t| t.text)
        .collect()
}

pub fn pilot_tag_strings_with_index(pilot: &PilotIntel, index: &AnnotationIndex) -> Vec<String> {
    pilot_tag_strings(pilot, &resolve_pilot_annotations(pilot, index))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::annotations::{Annotation, EntityType};
    use crate::view::test_support::{pilot, with_flags};

    fn annotation(tags: &[&str], color: &str) -> Annotation {
        Annotation {
            id: 1,
            network_id: 1,
            network_name: "Test".into(),
            target_type: EntityType::Character,
            target_id: 1,
            target_name: "Test Pilot".into(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            note: None,
            color: Some(color.into()),
            created_by: None,
        }
    }

    fn resolved(a: &Annotation) -> ResolvedAnnotation<'_> {
        ResolvedAnnotation {
            key: "character:1".into(),
            scope: EntityType::Character,
            annotation: a,
        }
    }

    fn texts(tags: &[PilotTag]) -> Vec<&str> {
        tags.iter().map(|t| t.text.as_str()).collect()
    }

    #[test]
    fn empty_for_no_flags_and_no_annotations() {
        assert!(pilot_tags(&pilot(1, "Test Pilot"), &[]).is_empty());
    }

    #[test]
    fn returns_flag_tags_with_colors() {
        let p = with_flags(pilot(1, "p"), |f| {
            f.is_cyno = true;
            f.is_recon = true;
        });
        let tags = pilot_tags(&p, &[]);
        assert_eq!(texts(&tags), vec!["RECON", "CYNO"]);
        assert_eq!(tags[0].key, "flag:RECON");
        assert_eq!(tags[0].color.as_deref(), Some("#14B8A6"));
    }

    #[test]
    fn includes_super_flag() {
        let p = with_flags(pilot(1, "p"), |f| f.is_super = true);
        assert!(texts(&pilot_tags(&p, &[])).contains(&"SUPER"));
    }

    #[test]
    fn includes_annotation_tags() {
        let a = annotation(&["HOSTILE", "SPY"], "#FF3B3B");
        let tags = pilot_tags(&pilot(1, "p"), &[resolved(&a)]);
        assert_eq!(texts(&tags), vec!["HOSTILE", "SPY"]);
        assert_eq!(tags[0].key, "character:1:HOSTILE");
        assert_eq!(tags[1].color.as_deref(), Some("#FF3B3B"));
    }

    #[test]
    fn dedupes_flags_and_annotations() {
        let a = annotation(&["CYNO"], "#A855F7");
        let p = with_flags(pilot(1, "p"), |f| f.is_cyno = true);
        let tags = pilot_tags(&p, &[resolved(&a)]);
        assert_eq!(tags.iter().filter(|t| t.text == "CYNO").count(), 1);
        assert_eq!(tags[0].key, "flag:CYNO");
    }

    #[test]
    fn tag_strings() {
        let p = with_flags(pilot(1, "p"), |f| {
            f.is_capital = true;
            f.is_solo = true;
        });
        assert_eq!(pilot_tag_strings(&p, &[]), vec!["CAPITAL", "SOLO"]);
        assert_eq!(
            pilot_tag_strings_with_index(&p, &AnnotationIndex::new()),
            vec!["CAPITAL", "SOLO"]
        );
    }
}
