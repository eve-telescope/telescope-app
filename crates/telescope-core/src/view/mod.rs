//! Pure view-layer logic ported from the Vue frontend: formatting, sorting,
//! filtering, counting and tag derivation for pilot lists, d-scan summaries
//! and network dialogs. Nothing here knows about the UI toolkit.

pub mod annotations;
pub mod dscan_view;
pub mod flags;
pub mod format;
pub mod network;
pub mod pilot_accumulator;
pub mod pilot_counts;
pub mod pilot_filters;
pub mod pilot_sort;
pub mod pilot_tags;
pub mod scan_input;
pub mod shortcut;
mod text;

#[cfg(test)]
mod test_support;
