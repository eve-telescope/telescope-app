//! Pure reducer for `IntelState`: every mutation the intel commands perform
//! is an `IntelAction`. Commands apply `reduce()` under the state lock and
//! then persist/emit exactly as before — the reducer never does I/O.

use crate::intel_state::IntelState;
use crate::models::{IntelEntry, IntelNetwork, NetworkDetail};

#[derive(Debug, Clone)]
pub enum IntelAction {
    SetBaseUrl(String),
    SetToken(String),
    /// Reset everything except the configured base URL.
    Logout,
    SetNetworks(Vec<IntelNetwork>),
    AddNetwork(IntelNetwork),
    /// Also clears `selected_network` if it was the removed one.
    RemoveNetwork(i64),
    SelectNetwork(NetworkDetail),
    ClearSelected,
    SetEntries(Vec<IntelEntry>),
    /// Replace the entry with the same id, or append if absent.
    UpsertEntry(IntelEntry),
    RemoveEntry(i64),
    SetActiveNetworkIds(Vec<i64>),
}

pub fn reduce(mut state: IntelState, action: IntelAction) -> IntelState {
    match action {
        IntelAction::SetBaseUrl(url) => state.api_base_url = url,
        IntelAction::SetToken(token) => state.api_token = Some(token),
        IntelAction::Logout => {
            state = IntelState {
                api_base_url: state.api_base_url,
                ..IntelState::default()
            };
        }
        IntelAction::SetNetworks(networks) => state.networks = networks,
        IntelAction::AddNetwork(network) => state.networks.push(network),
        IntelAction::RemoveNetwork(network_id) => {
            state.networks.retain(|n| n.id != network_id);
            if state.selected_network.as_ref().map(|n| n.id) == Some(network_id) {
                state.selected_network = None;
            }
        }
        IntelAction::SelectNetwork(detail) => state.selected_network = Some(detail),
        IntelAction::ClearSelected => state.selected_network = None,
        IntelAction::SetEntries(entries) => state.entries = entries,
        IntelAction::UpsertEntry(entry) => upsert_entry(&mut state.entries, entry),
        IntelAction::RemoveEntry(entry_id) => state.entries.retain(|e| e.id != entry_id),
        IntelAction::SetActiveNetworkIds(ids) => state.active_network_ids = ids,
    }
    state
}

fn upsert_entry(entries: &mut Vec<IntelEntry>, entry: IntelEntry) {
    if let Some(idx) = entries.iter().position(|e| e.id == entry.id) {
        entries[idx] = entry;
    } else {
        entries.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn network(id: i64, name: &str) -> IntelNetwork {
        IntelNetwork {
            id,
            name: name.to_string(),
            slug: name.to_lowercase(),
            entries_count: None,
        }
    }

    fn detail(id: i64) -> NetworkDetail {
        NetworkDetail {
            id,
            name: format!("Network {}", id),
            slug: format!("network-{}", id),
            entries: Vec::new(),
            accesses: Vec::new(),
        }
    }

    fn entry(id: i64, name: &str) -> IntelEntry {
        IntelEntry {
            id,
            intel_network_id: 1,
            network_name: "Net".to_string(),
            entity_type: "character".to_string(),
            entity_id: 1000 + id,
            entity_name: name.to_string(),
            color: None,
            label: None,
            notes: None,
        }
    }

    fn populated_state() -> IntelState {
        IntelState {
            api_base_url: "https://example.com".to_string(),
            api_token: Some("tok".to_string()),
            networks: vec![network(1, "Alpha"), network(2, "Beta")],
            entries: vec![entry(10, "Pilot A"), entry(11, "Pilot B")],
            selected_network: Some(detail(1)),
            active_network_ids: vec![1, 2],
        }
    }

    #[test]
    fn set_base_url_only_touches_base_url() {
        let state = reduce(
            populated_state(),
            IntelAction::SetBaseUrl("https://other.example".into()),
        );
        assert_eq!(state.api_base_url, "https://other.example");
        assert_eq!(state.api_token.as_deref(), Some("tok"));
        assert_eq!(state.networks.len(), 2);
    }

    #[test]
    fn set_token_stores_token() {
        let state = reduce(IntelState::default(), IntelAction::SetToken("abc".into()));
        assert_eq!(state.api_token.as_deref(), Some("abc"));
    }

    #[test]
    fn set_token_replaces_existing_token() {
        let state = reduce(populated_state(), IntelAction::SetToken("new".into()));
        assert_eq!(state.api_token.as_deref(), Some("new"));
        // Everything else is untouched (network refresh is a separate effect).
        assert_eq!(state.networks.len(), 2);
        assert!(state.selected_network.is_some());
    }

    #[test]
    fn logout_resets_everything_but_preserves_base_url() {
        let state = reduce(populated_state(), IntelAction::Logout);
        assert_eq!(state.api_base_url, "https://example.com");
        assert_eq!(state.api_token, None);
        assert!(state.networks.is_empty());
        assert!(state.entries.is_empty());
        assert!(state.selected_network.is_none());
        assert!(state.active_network_ids.is_empty());
    }

    #[test]
    fn set_networks_replaces_the_list() {
        let state = reduce(
            populated_state(),
            IntelAction::SetNetworks(vec![network(9, "Gamma")]),
        );
        assert_eq!(state.networks.len(), 1);
        assert_eq!(state.networks[0].id, 9);
    }

    #[test]
    fn add_network_appends() {
        let state = reduce(
            populated_state(),
            IntelAction::AddNetwork(network(3, "New")),
        );
        assert_eq!(state.networks.len(), 3);
        assert_eq!(state.networks[2].id, 3);
    }

    #[test]
    fn remove_network_drops_it_and_clears_matching_selection() {
        let state = reduce(populated_state(), IntelAction::RemoveNetwork(1));
        assert_eq!(state.networks.len(), 1);
        assert_eq!(state.networks[0].id, 2);
        assert!(state.selected_network.is_none());
    }

    #[test]
    fn remove_network_keeps_unrelated_selection() {
        let state = reduce(populated_state(), IntelAction::RemoveNetwork(2));
        assert_eq!(state.networks.len(), 1);
        assert_eq!(state.selected_network.as_ref().map(|n| n.id), Some(1));
    }

    #[test]
    fn remove_nonexistent_network_is_a_no_op() {
        let state = reduce(populated_state(), IntelAction::RemoveNetwork(999));
        assert_eq!(state.networks.len(), 2);
        assert!(state.selected_network.is_some());
    }

    #[test]
    fn select_and_clear_selected_network() {
        let state = reduce(populated_state(), IntelAction::SelectNetwork(detail(2)));
        assert_eq!(state.selected_network.as_ref().map(|n| n.id), Some(2));

        let state = reduce(state, IntelAction::ClearSelected);
        assert!(state.selected_network.is_none());
    }

    #[test]
    fn set_entries_replaces_the_list() {
        let state = reduce(
            populated_state(),
            IntelAction::SetEntries(vec![entry(99, "Only")]),
        );
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.entries[0].id, 99);
    }

    #[test]
    fn upsert_appends_a_new_entry() {
        let state = reduce(
            populated_state(),
            IntelAction::UpsertEntry(entry(12, "New")),
        );
        assert_eq!(state.entries.len(), 3);
        assert_eq!(state.entries[2].id, 12);
    }

    #[test]
    fn upsert_replaces_an_existing_entry_in_place() {
        let mut updated = entry(10, "Pilot A Renamed");
        updated.notes = Some("now with notes".to_string());
        let state = reduce(populated_state(), IntelAction::UpsertEntry(updated));
        assert_eq!(state.entries.len(), 2);
        assert_eq!(state.entries[0].id, 10);
        assert_eq!(state.entries[0].entity_name, "Pilot A Renamed");
        assert_eq!(state.entries[0].notes.as_deref(), Some("now with notes"));
        // Position preserved: the other entry is still second.
        assert_eq!(state.entries[1].id, 11);
    }

    #[test]
    fn remove_entry_drops_only_that_entry() {
        let state = reduce(populated_state(), IntelAction::RemoveEntry(10));
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.entries[0].id, 11);
    }

    #[test]
    fn remove_nonexistent_entry_is_a_no_op() {
        let state = reduce(populated_state(), IntelAction::RemoveEntry(999));
        assert_eq!(state.entries.len(), 2);
    }

    #[test]
    fn set_active_network_ids_replaces_the_list() {
        let state = reduce(populated_state(), IntelAction::SetActiveNetworkIds(vec![7]));
        assert_eq!(state.active_network_ids, vec![7]);
    }
}
