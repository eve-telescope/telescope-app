//! Per-category counts behind the filter chips.

use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use super::pilot_tags::PilotTag;
use crate::models::PilotIntel;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ThreatCounts {
    pub extreme: usize,
    pub high: usize,
    pub moderate: usize,
    pub low: usize,
    pub minimal: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagCount {
    pub tag: String,
    pub color: Option<String>,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupCount {
    pub ticker: String,
    pub name: String,
    pub count: usize,
}

/// Unknown and unrecognized threat levels are not counted.
pub fn threat_counts<'a>(pilots: impl IntoIterator<Item = &'a PilotIntel>) -> ThreatCounts {
    let mut counts = ThreatCounts::default();
    for pilot in pilots {
        match pilot.threat_level.to_lowercase().as_str() {
            "extreme" => counts.extreme += 1,
            "high" => counts.high += 1,
            "moderate" => counts.moderate += 1,
            "low" => counts.low += 1,
            "minimal" => counts.minimal += 1,
            _ => {}
        }
    }
    counts
}

/// Pilots per tag, each pilot counted once per tag. The color comes from the
/// first pilot seen with the tag. Sorted by count descending; ties keep
/// first-seen order.
pub fn tag_counts<'a>(
    pilots: impl IntoIterator<Item = &'a PilotIntel>,
    mut tags_of: impl FnMut(&PilotIntel) -> Vec<PilotTag>,
) -> Vec<TagCount> {
    let mut counts: Vec<TagCount> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();

    for pilot in pilots {
        let mut seen = HashSet::new();
        for tag in tags_of(pilot) {
            if !seen.insert(tag.text.clone()) {
                continue;
            }
            match index.get(&tag.text) {
                Some(&i) => counts[i].count += 1,
                None => {
                    index.insert(tag.text.clone(), counts.len());
                    counts.push(TagCount {
                        tag: tag.text,
                        color: tag.color,
                        count: 1,
                    });
                }
            }
        }
    }

    counts.sort_by_key(|c| Reverse(c.count));
    counts
}

fn group_counts<'a>(
    pilots: impl IntoIterator<Item = &'a PilotIntel>,
    group_of: impl Fn(&PilotIntel) -> (Option<&String>, Option<&String>),
) -> Vec<GroupCount> {
    let mut counts: Vec<GroupCount> = Vec::new();
    let mut index: HashMap<&str, usize> = HashMap::new();

    for pilot in pilots {
        let (ticker, name) = group_of(pilot);
        let Some(ticker) = ticker.filter(|t| !t.is_empty()) else {
            continue;
        };
        match index.get(ticker.as_str()) {
            Some(&i) => counts[i].count += 1,
            None => {
                index.insert(ticker, counts.len());
                counts.push(GroupCount {
                    ticker: ticker.clone(),
                    name: name.filter(|n| !n.is_empty()).unwrap_or(ticker).clone(),
                    count: 1,
                });
            }
        }
    }

    counts.sort_by_key(|c| Reverse(c.count));
    counts
}

/// Pilots per corporation ticker; the name falls back to the ticker.
pub fn corp_counts<'a>(pilots: impl IntoIterator<Item = &'a PilotIntel>) -> Vec<GroupCount> {
    group_counts(pilots, |p| {
        (
            p.character.corporation_ticker.as_ref(),
            p.character.corporation_name.as_ref(),
        )
    })
}

/// Pilots per alliance ticker; the name falls back to the ticker.
pub fn alliance_counts<'a>(pilots: impl IntoIterator<Item = &'a PilotIntel>) -> Vec<GroupCount> {
    group_counts(pilots, |p| {
        (
            p.character.alliance_ticker.as_ref(),
            p.character.alliance_name.as_ref(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::annotations::{Annotation, EntityType, ResolvedAnnotation};
    use crate::view::pilot_tags::pilot_tags;
    use crate::view::test_support::{affiliated_pilot, with_flags, with_threat};

    fn p(id: i64) -> PilotIntel {
        affiliated_pilot(id, "Pilot")
    }

    fn flag_tags(p: &PilotIntel) -> Vec<PilotTag> {
        pilot_tags(p, &[])
    }

    fn annotation(tags: &[&str], color: &str) -> Annotation {
        Annotation {
            id: 1,
            network_id: 1,
            network_name: "Net".into(),
            target_type: EntityType::Character,
            target_id: 1,
            target_name: "Pilot".into(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            note: None,
            color: Some(color.into()),
            created_by: None,
        }
    }

    fn with_annotation<'a>(a: &'a Annotation) -> impl FnMut(&PilotIntel) -> Vec<PilotTag> + 'a {
        move |p| {
            pilot_tags(
                p,
                &[ResolvedAnnotation {
                    key: "character:1".into(),
                    scope: EntityType::Character,
                    annotation: a,
                }],
            )
        }
    }

    #[test]
    fn counts_threat_levels() {
        let pilots = [
            with_threat(p(1), "extreme"),
            with_threat(p(2), "EXTREME"),
            with_threat(p(3), "low"),
            with_threat(p(4), "unknown"),
        ];
        let counts = threat_counts(&pilots);
        assert_eq!(counts.extreme, 2);
        assert_eq!(counts.low, 1);
        assert_eq!(counts.high, 0);
    }

    #[test]
    fn counts_flag_tags() {
        let pilots = [
            with_flags(p(1), |f| f.is_cyno = true),
            with_flags(p(2), |f| {
                f.is_cyno = true;
                f.is_recon = true;
            }),
            p(3),
        ];
        let counts = tag_counts(&pilots, flag_tags);
        assert_eq!(counts[0].tag, "CYNO");
        assert_eq!(counts[0].count, 2);
        assert_eq!(counts[0].color.as_deref(), Some("#A855F7"));
        assert_eq!(counts[1].tag, "RECON");
        assert_eq!(counts[1].count, 1);
    }

    #[test]
    fn includes_annotation_tags() {
        let a = annotation(&["HOSTILE"], "#FF3B3B");
        let counts = tag_counts(&[p(1)], with_annotation(&a));
        assert_eq!(
            counts,
            vec![TagCount {
                tag: "HOSTILE".into(),
                color: Some("#FF3B3B".into()),
                count: 1
            }]
        );
    }

    #[test]
    fn counts_each_pilot_once_per_tag() {
        let a = annotation(&["CYNO"], "#A855F7");
        let pilots = [with_flags(p(1), |f| f.is_cyno = true)];
        let counts = tag_counts(&pilots, with_annotation(&a));
        assert_eq!(counts.len(), 1);
        assert_eq!(counts[0].count, 1);
    }

    #[test]
    fn tag_ties_keep_first_seen_order() {
        let pilots = [
            with_flags(p(1), |f| f.is_solo = true),
            with_flags(p(2), |f| f.is_recon = true),
            with_flags(p(3), |f| f.is_recon = true),
        ];
        let tags: Vec<_> = tag_counts(&pilots, flag_tags)
            .into_iter()
            .map(|t| t.tag)
            .collect();
        assert_eq!(tags, vec!["RECON", "SOLO"]);
    }

    #[test]
    fn counts_corporations_by_ticker() {
        let mut pilots = [p(1), p(2), p(3)];
        for (pilot, (name, ticker)) in
            pilots
                .iter_mut()
                .zip([("Corp A", "CRPA"), ("Corp A", "CRPA"), ("Corp B", "CRPB")])
        {
            pilot.character.corporation_name = Some(name.into());
            pilot.character.corporation_ticker = Some(ticker.into());
        }
        let counts = corp_counts(&pilots);
        assert_eq!(counts.len(), 2);
        assert_eq!(counts[0].ticker, "CRPA");
        assert_eq!(counts[0].name, "Corp A");
        assert_eq!(counts[0].count, 2);
    }

    #[test]
    fn counts_alliances_by_ticker() {
        let mut pilots = [p(1), p(2), p(3)];
        for (pilot, ticker) in pilots.iter_mut().zip(["ALLX", "ALLX", "ALLY"]) {
            pilot.character.alliance_ticker = Some(ticker.into());
        }
        assert_eq!(alliance_counts(&pilots).len(), 2);
    }

    #[test]
    fn skips_missing_tickers_and_falls_back_to_ticker_name() {
        let mut no_alliance = p(1);
        no_alliance.character.alliance_ticker = None;
        let mut nameless = p(2);
        nameless.character.alliance_name = None;
        let counts = alliance_counts(&[no_alliance, nameless]);
        assert_eq!(counts.len(), 1);
        assert_eq!(counts[0].name, "TSTA");
    }
}
