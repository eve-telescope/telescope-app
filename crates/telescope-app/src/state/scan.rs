use std::collections::HashSet;

use gpui_kit::{App, AppContext as _, Context, Entity, EventEmitter, Task};
use log::{error, warn};
use telescope_core::domain::lookup::LookupProgress;
use telescope_core::lookup::{PilotBatch, lookup_pilots};
use telescope_core::models::{DscanParseResult, PilotIntel, SdeStatus};
use telescope_core::view::pilot_accumulator::PilotAccumulator;
use telescope_core::view::scan_input::{ScanInputKind, detect_scan_input_kind, split_pilot_names};

use crate::runtime;
use crate::services::Services;
use crate::state::filters::FilterStore;
use crate::state::intel::IntelStore;

pub enum ScanEvent {
    /// A scan of this kind started; the main window switches to its tab.
    Started(ScanInputKind),
}

/// The current local and d-scan results, shared by the main window and the
/// overlay.
pub struct ScanStore {
    intel: Entity<IntelStore>,
    filters: Entity<FilterStore>,
    pilots: PilotAccumulator,
    loading: bool,
    progress: Option<LookupProgress>,
    dscan: Option<DscanParseResult>,
    dscan_loading: bool,
    dscan_error: Option<String>,
    lookup_task: Option<Task<()>>,
    dscan_task: Option<Task<()>>,
    sde_status: Option<SdeStatus>,
    sde_syncing: bool,
    sde_task: Option<Task<()>>,
}

impl EventEmitter<ScanEvent> for ScanStore {}

impl ScanStore {
    pub fn new(intel: Entity<IntelStore>, filters: Entity<FilterStore>) -> Self {
        Self {
            intel,
            filters,
            pilots: PilotAccumulator::new(),
            loading: false,
            progress: None,
            dscan: None,
            dscan_loading: false,
            dscan_error: None,
            lookup_task: None,
            dscan_task: None,
            sde_status: None,
            sde_syncing: false,
            sde_task: None,
        }
    }

    pub fn sde_status(&self) -> Option<&SdeStatus> {
        self.sde_status.as_ref()
    }

    pub fn sde_syncing(&self) -> bool {
        self.sde_syncing
    }

    /// Downloads or refreshes the SDE type index used to classify d-scans.
    pub fn ensure_sde(&mut self, cx: &mut Context<Self>) {
        if self.sde_syncing {
            return;
        }
        self.sde_syncing = true;
        cx.notify();
        let services = Services::get(cx);
        let sde = services.sde.clone();
        let dir = services.paths.data.clone();
        let ensure =
            runtime::spawn(async move { telescope_core::sde::ensure_sde_index(&dir, &sde).await });
        self.sde_task = Some(cx.spawn(async move |this, cx| {
            let result = ensure.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(status) => this.sde_status = Some(status),
                    Err(e) => {
                        warn!("SDE index update failed: {}", e);
                        if let Some(status) = &mut this.sde_status {
                            status.last_error = Some(e);
                        }
                    }
                }
                this.sde_syncing = false;
                cx.notify();
            });
        }));
    }

    pub fn pilots(&self) -> &[PilotIntel] {
        self.pilots.as_slice()
    }

    pub fn loading(&self) -> bool {
        self.loading
    }

    pub fn progress(&self) -> Option<LookupProgress> {
        self.progress
    }

    pub fn dscan(&self) -> Option<&DscanParseResult> {
        self.dscan.as_ref()
    }

    pub fn dscan_loading(&self) -> bool {
        self.dscan_loading
    }

    pub fn dscan_error(&self) -> Option<&str> {
        self.dscan_error.as_deref()
    }

    /// Scans `text` and shares it with the active intel network, like a
    /// paste from the input panel, the global hotkey or a share link.
    pub fn submit(&mut self, text: &str, cx: &mut Context<Self>) {
        let kind = self.load(text, cx);
        self.intel
            .update(cx, |intel, cx| intel.share_scan(kind, text, cx));
    }

    /// Scans `text` without sharing it (scan history entries).
    pub fn load(&mut self, text: &str, cx: &mut Context<Self>) -> ScanInputKind {
        let kind = detect_scan_input_kind(text);
        cx.emit(ScanEvent::Started(kind));
        match kind {
            ScanInputKind::Dscan => self.parse_dscan(text.to_string(), cx),
            ScanInputKind::Local => self.lookup(text.to_string(), cx),
        }
        kind
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.lookup_task = None;
        self.dscan_task = None;
        self.pilots.clear();
        self.loading = false;
        self.progress = None;
        self.dscan = None;
        self.dscan_loading = false;
        self.dscan_error = None;
        self.filters
            .update(cx, |filters, cx| filters.update(cx, |f| f.clear()));
        cx.notify();
    }

    fn lookup(&mut self, text: String, cx: &mut Context<Self>) {
        if text.trim().is_empty() {
            return;
        }

        // Keep rows for pilots that are also in the new scan, so a rescan
        // updates them in place instead of blanking the table.
        let names: HashSet<String> = split_pilot_names(&text)
            .into_iter()
            .map(|name| name.to_lowercase())
            .collect();
        self.pilots
            .retain_where(|p| names.contains(&p.character.name.to_lowercase()));
        self.loading = true;
        self.progress = None;
        self.intel.update(cx, |intel, cx| intel.clear_entries(cx));
        self.filters
            .update(cx, |filters, cx| filters.update(cx, |f| f.clear()));
        cx.notify();

        let cache = Services::get(cx).cache.clone();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<PilotBatch>();
        let lookup = runtime::spawn(async move {
            let result = lookup_pilots(&cache, &text, tx).await;
            if let Err(e) = cache.flush() {
                warn!("Failed to persist cache: {}", e);
            }
            result
        });

        self.lookup_task = Some(cx.spawn(async move |this, cx| {
            while let Some(batch) = rx.recv().await {
                let applied = this.update(cx, |this, cx| {
                    for result in batch.pilots {
                        this.pilots.upsert(result.pilot);
                    }
                    this.progress = Some(batch.progress);
                    cx.notify();
                });
                if applied.is_err() {
                    return;
                }
            }

            let result = lookup.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    // Batches can trail the return value, so merge the
                    // authoritative list; upserting by id makes overlap safe.
                    Ok(pilots) => {
                        for pilot in pilots {
                            this.pilots.upsert(pilot);
                        }
                        let ids = this.entity_ids();
                        this.intel
                            .update(cx, |intel, cx| intel.lookup_intel(ids, cx));
                    }
                    Err(e) => error!("Failed to lookup pilots: {}", e),
                }
                this.loading = false;
                this.progress = None;
                cx.notify();
            });
        }));
    }

    fn entity_ids(&self) -> Vec<i64> {
        let mut seen = HashSet::new();
        self.pilots
            .iter()
            .flat_map(|p| {
                [
                    Some(p.character.id),
                    p.character.corporation_id,
                    p.character.alliance_id,
                ]
            })
            .flatten()
            .filter(|id| *id != 0 && seen.insert(*id))
            .collect()
    }

    fn parse_dscan(&mut self, text: String, cx: &mut Context<Self>) {
        self.dscan_loading = true;
        self.dscan_error = None;
        cx.notify();

        let services = Services::get(cx);
        let sde = services.sde.clone();
        let dir = services.paths.data.clone();
        let parse = runtime::spawn(async move { sde.parse_dscan(&dir, text).await });
        self.dscan_task = Some(cx.spawn(async move |this, cx| {
            let result = parse.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(result) => this.dscan = Some(result),
                    Err(e) => {
                        this.dscan = None;
                        this.dscan_error = Some(e);
                    }
                }
                this.dscan_loading = false;
                cx.notify();
            });
        }));
    }
}

pub fn init(
    intel: Entity<IntelStore>,
    filters: Entity<FilterStore>,
    cx: &mut App,
) -> Entity<ScanStore> {
    cx.new(|cx| {
        let mut store = ScanStore::new(intel, filters);
        store.ensure_sde(cx);
        store
    })
}
