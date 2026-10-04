//! Intel annotations: network entries viewed as tagged notes on a character,
//! corporation or alliance, plus the index used to resolve them per pilot.

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use super::text::{is_js_whitespace, js_trim};
use crate::models::{IntelEntry, IntelEntryDetail, PilotIntel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Character,
    Corporation,
    Alliance,
}

impl EntityType {
    pub const ALL: [EntityType; 3] = [Self::Character, Self::Corporation, Self::Alliance];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Character => "character",
            Self::Corporation => "corporation",
            Self::Alliance => "alliance",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "character" => Some(Self::Character),
            "corporation" => Some(Self::Corporation),
            "alliance" => Some(Self::Alliance),
            _ => None,
        }
    }
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationAuthor {
    pub id: i64,
    pub character_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    pub id: i64,
    pub network_id: i64,
    pub network_name: String,
    pub target_type: EntityType,
    pub target_id: i64,
    pub target_name: String,
    pub tags: Vec<String>,
    pub note: Option<String>,
    pub color: Option<String>,
    pub created_by: Option<AnnotationAuthor>,
}

impl Annotation {
    /// `None` when the entry's entity type is not one of the known scopes.
    pub fn from_intel_entry(entry: &IntelEntry) -> Option<Self> {
        let tags = parse_annotation_tags(entry.label.as_deref().unwrap_or(""));
        Some(Self {
            id: entry.id,
            network_id: entry.intel_network_id,
            network_name: entry.network_name.clone(),
            target_type: EntityType::parse(&entry.entity_type)?,
            target_id: entry.entity_id,
            target_name: entry.entity_name.clone(),
            color: annotation_color(&tags, entry.color.as_deref()).map(str::to_string),
            tags,
            note: entry.notes.clone(),
            created_by: None,
        })
    }

    /// Entry details carry no network of their own, so the caller supplies it.
    pub fn from_entry_detail(
        entry: &IntelEntryDetail,
        network_id: i64,
        network_name: &str,
    ) -> Option<Self> {
        let tags = parse_annotation_tags(entry.label.as_deref().unwrap_or(""));
        Some(Self {
            id: entry.id,
            network_id,
            network_name: network_name.to_string(),
            target_type: EntityType::parse(&entry.entity_type)?,
            target_id: entry.entity_id,
            target_name: entry.entity_name.clone(),
            color: annotation_color(&tags, entry.color.as_deref()).map(str::to_string),
            tags,
            note: entry.notes.clone(),
            created_by: entry.added_by.as_ref().map(|a| AnnotationAuthor {
                id: a.id,
                character_name: a.character_name.clone(),
            }),
        })
    }
}

pub struct PresetTag {
    pub tag: &'static str,
    pub color: &'static str,
}

pub const PRESET_ANNOTATION_TAGS: [PresetTag; 6] = [
    PresetTag {
        tag: "HOSTILE",
        color: "#FF3B3B",
    },
    PresetTag {
        tag: "SCOUT",
        color: "#00D4FF",
    },
    PresetTag {
        tag: "SPY",
        color: "#8B5CF6",
    },
    PresetTag {
        tag: "FRIENDLY",
        color: "#00FF88",
    },
    PresetTag {
        tag: "ALLY",
        color: "#34D399",
    },
    PresetTag {
        tag: "NEUTRAL",
        color: "#FFD93D",
    },
];

/// Badge color for annotations without a preset or custom color.
pub const DEFAULT_ANNOTATION_COLOR: &str = "#8b8b96";

pub const TAG_DELIMITER: &str = " | ";

pub fn normalize_annotation_tag(tag: &str) -> String {
    let mut out = String::with_capacity(tag.len());
    let mut in_space = false;
    for c in js_trim(tag).chars() {
        if is_js_whitespace(c) {
            if !in_space {
                out.push(' ');
                in_space = true;
            }
        } else {
            out.extend(c.to_uppercase());
            in_space = false;
        }
    }
    out
}

pub fn parse_annotation_tags(label: &str) -> Vec<String> {
    label
        .split('|')
        .map(normalize_annotation_tag)
        .filter(|t| !t.is_empty())
        .collect()
}

/// Normalizes, drops empties and dedupes (keeping first occurrence).
pub fn serialize_annotation_tags<S: AsRef<str>>(tags: &[S]) -> String {
    let mut unique: Vec<String> = Vec::with_capacity(tags.len());
    for tag in tags {
        let normalized = normalize_annotation_tag(tag.as_ref());
        if !normalized.is_empty() && !unique.contains(&normalized) {
            unique.push(normalized);
        }
    }
    unique.join(TAG_DELIMITER)
}

pub fn preset_annotation_color(tag: &str) -> Option<&'static str> {
    let normalized = normalize_annotation_tag(tag);
    PRESET_ANNOTATION_TAGS
        .iter()
        .find(|p| p.tag == normalized)
        .map(|p| p.color)
}

/// First preset color among the tags, else the fallback.
pub fn annotation_color<'a, S: AsRef<str>>(
    tags: &[S],
    fallback: Option<&'a str>,
) -> Option<&'a str> {
    tags.iter()
        .find_map(|t| preset_annotation_color(t.as_ref()))
        .or(fallback)
}

/// Color stored with a new or updated entry.
pub fn annotation_save_color<S: AsRef<str>>(tags: &[S]) -> &'static str {
    annotation_color(tags, None).unwrap_or(DEFAULT_ANNOTATION_COLOR)
}

pub fn target_key(target_type: EntityType, target_id: i64) -> String {
    format!("{target_type}:{target_id}")
}

pub fn format_scope(scope: EntityType) -> &'static str {
    match scope {
        EntityType::Character => "CHAR",
        EntityType::Corporation => "CORP",
        EntityType::Alliance => "ALLY",
    }
}

pub fn annotations_from_entries(entries: &[IntelEntry]) -> Vec<Annotation> {
    entries
        .iter()
        .filter_map(Annotation::from_intel_entry)
        .collect()
}

/// Annotations grouped by target, in entry order within each target.
pub type AnnotationIndex = HashMap<(EntityType, i64), Vec<Annotation>>;

pub fn annotations_by_target_key(
    annotations: impl IntoIterator<Item = Annotation>,
) -> AnnotationIndex {
    let mut index = AnnotationIndex::new();
    for annotation in annotations {
        index
            .entry((annotation.target_type, annotation.target_id))
            .or_default()
            .push(annotation);
    }
    index
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAnnotation<'a> {
    /// `"{scope}:{annotation id}"`, stable across renders.
    pub key: String,
    pub scope: EntityType,
    pub annotation: &'a Annotation,
}

/// Character annotations first, then corporation, then alliance. Missing or
/// zero ids are skipped.
pub fn get_annotations_for_pilot(
    character_id: i64,
    corporation_id: Option<i64>,
    alliance_id: Option<i64>,
    index: &AnnotationIndex,
) -> Vec<ResolvedAnnotation<'_>> {
    let targets = [
        (EntityType::Character, Some(character_id)),
        (EntityType::Corporation, corporation_id),
        (EntityType::Alliance, alliance_id),
    ];

    let mut matches = Vec::new();
    for (scope, id) in targets {
        let Some(id) = id.filter(|&id| id != 0) else {
            continue;
        };
        for annotation in index.get(&(scope, id)).into_iter().flatten() {
            matches.push(ResolvedAnnotation {
                key: format!("{scope}:{}", annotation.id),
                scope,
                annotation,
            });
        }
    }
    matches
}

pub fn resolve_pilot_annotations<'a>(
    pilot: &PilotIntel,
    index: &'a AnnotationIndex,
) -> Vec<ResolvedAnnotation<'a>> {
    get_annotations_for_pilot(
        pilot.character.id,
        pilot.character.corporation_id,
        pilot.character.alliance_id,
        index,
    )
}

/// Something that can be annotated: a character, corporation or alliance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub entity_type: EntityType,
    pub id: i64,
    pub name: String,
}

impl Target {
    /// The pilot, then their corporation and alliance when known.
    pub fn for_pilot(pilot: &PilotIntel) -> Vec<Target> {
        let c = &pilot.character;
        let mut targets = vec![Target {
            entity_type: EntityType::Character,
            id: c.id,
            name: c.name.clone(),
        }];
        if let (Some(id), Some(name)) =
            (c.corporation_id.filter(|id| *id != 0), &c.corporation_name)
        {
            targets.push(Target {
                entity_type: EntityType::Corporation,
                id,
                name: name.clone(),
            });
        }
        if let (Some(id), Some(name)) = (c.alliance_id.filter(|id| *id != 0), &c.alliance_name) {
            targets.push(Target {
                entity_type: EntityType::Alliance,
                id,
                name: name.clone(),
            });
        }
        targets
    }

    pub fn of(annotation: &Annotation) -> Target {
        Target {
            entity_type: annotation.target_type,
            id: annotation.target_id,
            name: annotation.target_name.clone(),
        }
    }
}

/// The annotation `network_id` holds for `target`, if any.
pub fn find_annotation<'a>(
    index: &'a AnnotationIndex,
    network_id: i64,
    target: &Target,
) -> Option<&'a Annotation> {
    index
        .get(&(target.entity_type, target.id))?
        .iter()
        .find(|a| a.network_id == network_id)
}

/// Non-preset tags used on `network_id`, alphabetically, each with the color
/// of the first annotation carrying it.
pub fn custom_tags(index: &AnnotationIndex, network_id: i64) -> Vec<(String, String)> {
    let mut tags: Vec<(String, String)> = Vec::new();
    for annotation in index
        .values()
        .flatten()
        .filter(|a| a.network_id == network_id)
    {
        for tag in &annotation.tags {
            let preset = PRESET_ANNOTATION_TAGS.iter().any(|p| p.tag == tag);
            if !preset && !tags.iter().any(|(t, _)| t == tag) {
                let color = annotation
                    .color
                    .clone()
                    .unwrap_or_else(|| DEFAULT_ANNOTATION_COLOR.into());
                tags.push((tag.clone(), color));
            }
        }
    }
    tags.sort();
    tags
}

/// What toggling one tag on a target's annotation should do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagToggle {
    Create(Vec<String>),
    Update {
        entry_id: i64,
        tags: Vec<String>,
    },
    /// The last tag went away; the annotation is removed even with a note,
    /// matching the original quick toggle.
    Remove {
        entry_id: i64,
    },
}

pub fn toggle_tag(existing: Option<&Annotation>, tag: &str) -> TagToggle {
    let tag = normalize_annotation_tag(tag);
    let Some(existing) = existing else {
        return TagToggle::Create(vec![tag]);
    };
    let mut tags = existing.tags.clone();
    match tags.iter().position(|t| *t == tag) {
        Some(i) => {
            tags.remove(i);
        }
        None => tags.push(tag),
    }
    if tags.is_empty() {
        TagToggle::Remove {
            entry_id: existing.id,
        }
    } else {
        TagToggle::Update {
            entry_id: existing.id,
            tags,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AddedBy;
    use crate::view::test_support::affiliated_pilot;

    fn entry(id: i64, entity_type: &str, entity_id: i64, label: Option<&str>) -> IntelEntry {
        IntelEntry {
            id,
            intel_network_id: 1,
            network_name: "Test Network".into(),
            entity_type: entity_type.into(),
            entity_id,
            entity_name: "Bad Guy".into(),
            color: Some("#FF3B3B".into()),
            label: label.map(str::to_string),
            notes: None,
        }
    }

    fn annotation_with_tags(tags: &[&str]) -> Annotation {
        Annotation::from_intel_entry(&entry(10, "character", 42, Some(&tags.join(" | ")))).unwrap()
    }

    #[test]
    fn normalize_uppercases_and_trims() {
        assert_eq!(normalize_annotation_tag("  hostile  "), "HOSTILE");
    }

    #[test]
    fn normalize_collapses_whitespace() {
        assert_eq!(normalize_annotation_tag("black  ops"), "BLACK OPS");
        assert_eq!(normalize_annotation_tag("black \t\n ops"), "BLACK OPS");
    }

    #[test]
    fn normalize_handles_empty() {
        assert_eq!(normalize_annotation_tag(""), "");
    }

    #[test]
    fn parse_splits_by_pipe_and_normalizes() {
        assert_eq!(
            parse_annotation_tags("HOSTILE | SPY"),
            vec!["HOSTILE", "SPY"]
        );
        assert_eq!(
            parse_annotation_tags("hostile | spy"),
            vec!["HOSTILE", "SPY"]
        );
    }

    #[test]
    fn parse_filters_empty_segments() {
        assert_eq!(
            parse_annotation_tags("HOSTILE | | SPY"),
            vec!["HOSTILE", "SPY"]
        );
        assert!(parse_annotation_tags("").is_empty());
    }

    #[test]
    fn serialize_joins_and_dedupes() {
        assert_eq!(
            serialize_annotation_tags(&["HOSTILE", "SPY"]),
            "HOSTILE | SPY"
        );
        assert_eq!(
            serialize_annotation_tags(&["HOSTILE", "hostile"]),
            "HOSTILE"
        );
        assert_eq!(serialize_annotation_tags::<&str>(&[]), "");
    }

    #[test]
    fn preset_color_lookup() {
        assert_eq!(preset_annotation_color("HOSTILE"), Some("#FF3B3B"));
        assert_eq!(preset_annotation_color("CUSTOM"), None);
        assert_eq!(preset_annotation_color("hostile"), Some("#FF3B3B"));
    }

    #[test]
    fn annotation_color_prefers_first_preset() {
        assert_eq!(
            annotation_color(&["CUSTOM", "HOSTILE"], None),
            Some("#FF3B3B")
        );
        assert_eq!(
            annotation_color(&["CUSTOM"], Some("#000000")),
            Some("#000000")
        );
        assert_eq!(annotation_color(&["CUSTOM"], None), None);
        assert_eq!(annotation_save_color(&["CUSTOM"]), DEFAULT_ANNOTATION_COLOR);
    }

    #[test]
    fn has_expected_presets() {
        let tags: Vec<_> = PRESET_ANNOTATION_TAGS.iter().map(|p| p.tag).collect();
        for t in ["HOSTILE", "FRIENDLY", "SPY", "NEUTRAL"] {
            assert!(tags.contains(&t));
        }
    }

    #[test]
    fn target_key_and_scope_format() {
        assert_eq!(
            target_key(EntityType::Corporation, 98000001),
            "corporation:98000001"
        );
        assert_eq!(format_scope(EntityType::Character), "CHAR");
        assert_eq!(format_scope(EntityType::Corporation), "CORP");
        assert_eq!(format_scope(EntityType::Alliance), "ALLY");
    }

    #[test]
    fn maps_entries_to_annotations() {
        let a = Annotation::from_intel_entry(&entry(9010, "character", 1, Some("HOSTILE | SPY")))
            .unwrap();
        assert_eq!(a.tags, vec!["HOSTILE", "SPY"]);
        assert_eq!(a.network_id, 1);
        assert_eq!(a.network_name, "Test Network");
        assert_eq!(a.color.as_deref(), Some("#FF3B3B"));
        assert!(a.created_by.is_none());
    }

    #[test]
    fn preset_color_overrides_stored_color() {
        let mut e = entry(1, "character", 1, Some("custom | friendly"));
        e.color = Some("#123456".into());
        let a = Annotation::from_intel_entry(&e).unwrap();
        assert_eq!(a.color.as_deref(), Some("#00FF88"));
    }

    #[test]
    fn handles_null_label() {
        let mut e = entry(9011, "character", 1, None);
        e.notes = Some("Just a note".into());
        let a = Annotation::from_intel_entry(&e).unwrap();
        assert!(a.tags.is_empty());
        assert_eq!(a.note.as_deref(), Some("Just a note"));
    }

    #[test]
    fn unknown_entity_type_is_skipped() {
        assert!(Annotation::from_intel_entry(&entry(1, "faction", 1, None)).is_none());
    }

    #[test]
    fn maps_entry_detail_with_author() {
        let detail = IntelEntryDetail {
            id: 7,
            entity_type: "alliance".into(),
            entity_id: 99,
            entity_name: "Goons".into(),
            color: None,
            label: Some("spy".into()),
            notes: None,
            added_by: Some(AddedBy {
                id: 3,
                character_name: "Scout".into(),
            }),
        };
        let a = Annotation::from_entry_detail(&detail, 5, "Net").unwrap();
        assert_eq!(a.network_id, 5);
        assert_eq!(a.network_name, "Net");
        assert_eq!(a.target_type, EntityType::Alliance);
        assert_eq!(a.color.as_deref(), Some("#8B5CF6"));
        assert_eq!(
            a.created_by,
            Some(AnnotationAuthor {
                id: 3,
                character_name: "Scout".into()
            })
        );
    }

    #[test]
    fn indexes_annotations_by_target() {
        let index = annotations_by_target_key(annotations_from_entries(&[
            entry(9020, "corporation", 98000001, Some("HOSTILE")),
            entry(9021, "corporation", 98000001, Some("SPY")),
        ]));
        let list = &index[&(EntityType::Corporation, 98000001)];
        assert_eq!(
            list.iter().map(|a| a.id).collect::<Vec<_>>(),
            vec![9020, 9021]
        );
    }

    #[test]
    fn resolves_character_and_corporation_annotations() {
        let index = annotations_by_target_key(annotations_from_entries(&[
            entry(9030, "character", 55555, Some("SPY")),
            entry(9031, "corporation", 98000001, Some("HOSTILE")),
            entry(9032, "alliance", 77, Some("ALLY")),
        ]));

        let results = get_annotations_for_pilot(55555, Some(98000001), None, &index);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].scope, EntityType::Character);
        assert_eq!(results[0].key, "character:9030");
        assert_eq!(results[1].scope, EntityType::Corporation);
        assert_eq!(results[1].key, "corporation:9031");
    }

    #[test]
    fn resolves_nothing_when_no_annotations_match() {
        let index =
            annotations_by_target_key(annotations_from_entries(&[entry(1, "character", 1, None)]));
        assert!(get_annotations_for_pilot(99999, Some(99998), Some(99997), &index).is_empty());
    }

    #[test]
    fn skips_zero_ids() {
        let index = annotations_by_target_key(annotations_from_entries(&[entry(
            1,
            "character",
            0,
            Some("X"),
        )]));
        assert!(get_annotations_for_pilot(0, None, None, &index).is_empty());
    }

    #[test]
    fn resolves_annotations_for_pilot() {
        let index = annotations_by_target_key(annotations_from_entries(&[entry(
            9040,
            "character",
            12345,
            Some("HOSTILE"),
        )]));
        let pilot = affiliated_pilot(12345, "Test Pilot");
        let results = resolve_pilot_annotations(&pilot, &index);
        assert!(
            results
                .iter()
                .any(|r| r.annotation.tags.contains(&"HOSTILE".to_string()))
        );
    }

    #[test]
    fn toggle_tag_creates_updates_and_removes() {
        assert_eq!(
            toggle_tag(None, "hostile"),
            TagToggle::Create(vec!["HOSTILE".into()])
        );

        let a = annotation_with_tags(&["HOSTILE", "SPY"]);
        assert_eq!(
            toggle_tag(Some(&a), "SPY"),
            TagToggle::Update {
                entry_id: a.id,
                tags: vec!["HOSTILE".into()]
            }
        );
        assert_eq!(
            toggle_tag(Some(&a), "Scout"),
            TagToggle::Update {
                entry_id: a.id,
                tags: vec!["HOSTILE".into(), "SPY".into(), "SCOUT".into()]
            }
        );

        let single = annotation_with_tags(&["HOSTILE"]);
        assert_eq!(
            toggle_tag(Some(&single), "HOSTILE"),
            TagToggle::Remove {
                entry_id: single.id
            }
        );
    }

    #[test]
    fn find_and_custom_tags_are_scoped_to_the_network() {
        let mut ours = annotation_with_tags(&["HOSTILE", "BAITER"]);
        ours.network_id = 1;
        ours.color = Some("#123456".into());
        let mut theirs = annotation_with_tags(&["OTHER"]);
        theirs.network_id = 2;
        theirs.id += 1;
        let target = Target::of(&ours);
        let index = annotations_by_target_key([ours.clone(), theirs]);

        assert_eq!(find_annotation(&index, 1, &target), Some(&ours));
        assert_eq!(find_annotation(&index, 3, &target), None);
        assert_eq!(
            custom_tags(&index, 1),
            vec![("BAITER".to_string(), "#123456".to_string())]
        );
    }

    #[test]
    fn targets_for_pilot_skip_missing_affiliations() {
        let pilot = affiliated_pilot(7, "Pilot");
        let types: Vec<_> = Target::for_pilot(&pilot)
            .iter()
            .map(|t| t.entity_type)
            .collect();
        assert_eq!(
            types,
            [
                EntityType::Character,
                EntityType::Corporation,
                EntityType::Alliance
            ]
        );
        let solo = crate::view::test_support::pilot(8, "Solo");
        assert_eq!(Target::for_pilot(&solo).len(), 1);
    }
}
