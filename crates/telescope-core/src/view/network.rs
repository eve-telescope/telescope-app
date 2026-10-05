//! Helpers for the intel network dialogs: access list, permission labels
//! and the annotation form.

use super::annotations::{TAG_DELIMITER, normalize_annotation_tag, parse_annotation_tags};
use super::format::{alliance_logo_url, character_portrait_url, corporation_logo_url};
use crate::models::NetworkAccess;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionLevel {
    Viewer,
    Member,
    Manager,
}

impl PermissionLevel {
    pub const ALL: [PermissionLevel; 3] = [Self::Viewer, Self::Member, Self::Manager];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Viewer => "viewer",
            Self::Member => "member",
            Self::Manager => "manager",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.as_str() == s)
    }
}

pub fn describe_permission(level: PermissionLevel) -> &'static str {
    match level {
        PermissionLevel::Viewer => "Can view intel entries",
        PermissionLevel::Member => "Can view and add/edit entries",
        PermissionLevel::Manager => "Full control including access management",
    }
}

/// Portrait or logo URL for an entity. Accepts both the plain scope names
/// and backend model class names (`App\Models\User`, `...\Corporation`,
/// `...\Alliance`). `None` for anything else.
pub fn portrait_url(entity_type: &str, id: i64, size: u32) -> Option<String> {
    if entity_type == "character" || entity_type.contains("User") {
        Some(character_portrait_url(id, size))
    } else if entity_type == "corporation" || entity_type.contains("Corporation") {
        Some(corporation_logo_url(id, size))
    } else if entity_type == "alliance" || entity_type.contains("Alliance") {
        Some(alliance_logo_url(id, size))
    } else {
        None
    }
}

/// Owner accesses can never be revoked.
pub fn can_remove_access(access: &NetworkAccess) -> bool {
    !access.is_owner
}

/// Badge label for an access row: "owner" trumps the permission level.
pub fn access_permission_label(access: &NetworkAccess) -> &str {
    if access.is_owner {
        "owner"
    } else {
        &access.permission
    }
}

/// Toggles a tag inside a raw `|`-separated tags input, keeping the order
/// of the remaining tags. The result is normalized.
pub fn toggle_annotation_tag(tags_input: &str, tag: &str) -> String {
    let mut tags = parse_annotation_tags(tags_input);
    let normalized = normalize_annotation_tag(tag);
    match tags.iter().position(|t| *t == normalized) {
        Some(i) => {
            tags.remove(i);
        }
        None => tags.push(normalized),
    }
    tags.join(TAG_DELIMITER)
}

/// No tags and no (non-empty) note means the form carries nothing.
pub fn is_annotation_form_empty<S: AsRef<str>>(tags: &[S], note: Option<&str>) -> bool {
    tags.is_empty() && note.is_none_or(str::is_empty)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationSaveAction {
    Create,
    Update,
    Delete,
    /// A fresh, empty form: the dialog just closes.
    Nothing,
}

/// Clearing an existing entry deletes it; otherwise edit updates and a new
/// form creates.
pub fn resolve_annotation_save_action(
    editing_id: Option<i64>,
    is_empty: bool,
) -> AnnotationSaveAction {
    match (editing_id.is_some(), is_empty) {
        (true, true) => AnnotationSaveAction::Delete,
        (true, false) => AnnotationSaveAction::Update,
        (false, true) => AnnotationSaveAction::Nothing,
        (false, false) => AnnotationSaveAction::Create,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn access(is_owner: bool, permission: &str) -> NetworkAccess {
        NetworkAccess {
            id: 1,
            accessible_type: "character".into(),
            accessible_id: 1,
            permission: permission.into(),
            is_owner,
            expires_at: None,
            entity: None,
        }
    }

    #[test]
    fn portrait_urls_by_scope() {
        assert_eq!(
            portrait_url("character", 42, 32).unwrap(),
            "https://images.evetech.net/characters/42/portrait?size=32"
        );
        assert_eq!(
            portrait_url("corporation", 7, 32).unwrap(),
            "https://images.evetech.net/corporations/7/logo?size=32"
        );
        assert_eq!(
            portrait_url("alliance", 9, 64).unwrap(),
            "https://images.evetech.net/alliances/9/logo?size=64"
        );
    }

    #[test]
    fn portrait_urls_for_backend_type_names() {
        assert!(
            portrait_url("App\\Models\\User", 1, 32)
                .unwrap()
                .contains("/characters/")
        );
        assert!(
            portrait_url("App\\Models\\Corporation", 1, 32)
                .unwrap()
                .contains("/corporations/")
        );
        assert!(
            portrait_url("App\\Models\\Alliance", 1, 32)
                .unwrap()
                .contains("/alliances/")
        );
    }

    #[test]
    fn portrait_url_unknown_type() {
        assert_eq!(portrait_url("unknown", 1, 32), None);
    }

    #[test]
    fn owner_access_cannot_be_removed() {
        assert!(can_remove_access(&access(false, "viewer")));
        assert!(!can_remove_access(&access(true, "manager")));
    }

    #[test]
    fn permission_labels() {
        assert_eq!(access_permission_label(&access(true, "manager")), "owner");
        assert_eq!(access_permission_label(&access(false, "viewer")), "viewer");
        assert_eq!(access_permission_label(&access(false, "member")), "member");
    }

    #[test]
    fn describes_permissions() {
        assert_eq!(
            describe_permission(PermissionLevel::Viewer),
            "Can view intel entries"
        );
        assert_eq!(
            describe_permission(PermissionLevel::Member),
            "Can view and add/edit entries"
        );
        assert_eq!(
            describe_permission(PermissionLevel::Manager),
            "Full control including access management"
        );
        assert_eq!(
            PermissionLevel::parse("member"),
            Some(PermissionLevel::Member)
        );
        assert_eq!(PermissionLevel::parse("owner"), None);
    }

    #[test]
    fn toggle_tag_adds_and_removes() {
        assert_eq!(toggle_annotation_tag("", "HOSTILE"), "HOSTILE");
        assert_eq!(toggle_annotation_tag("HOSTILE", "SCOUT"), "HOSTILE | SCOUT");
        assert_eq!(toggle_annotation_tag("HOSTILE | SCOUT", "HOSTILE"), "SCOUT");
    }

    #[test]
    fn toggle_tag_normalizes_and_keeps_order() {
        assert_eq!(toggle_annotation_tag("HOSTILE", "  hostile "), "");
        assert_eq!(toggle_annotation_tag("a | b | c", "B"), "A | C");
    }

    #[test]
    fn annotation_form_emptiness() {
        assert!(is_annotation_form_empty::<&str>(&[], None));
        assert!(is_annotation_form_empty::<&str>(&[], Some("")));
        assert!(!is_annotation_form_empty(&["HOSTILE"], None));
        assert!(!is_annotation_form_empty::<&str>(&[], Some("seen in Jita")));
    }

    #[test]
    fn save_actions() {
        assert_eq!(
            resolve_annotation_save_action(Some(5), true),
            AnnotationSaveAction::Delete
        );
        assert_eq!(
            resolve_annotation_save_action(Some(5), false),
            AnnotationSaveAction::Update
        );
        assert_eq!(
            resolve_annotation_save_action(None, false),
            AnnotationSaveAction::Create
        );
        assert_eq!(
            resolve_annotation_save_action(None, true),
            AnnotationSaveAction::Nothing
        );
    }
}
