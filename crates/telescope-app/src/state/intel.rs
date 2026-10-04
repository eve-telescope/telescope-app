use std::future::Future;
use std::sync::Arc;

use gpui_kit::{App, AppContext as _, AsyncApp, Context, Entity, EventEmitter, Task, WeakEntity};
use log::{error, info, warn};
use telescope_core::intel_service::{AccessInput, EntryInput, IntelService};
use telescope_core::intel_state::IntelState;
use telescope_core::models::PilotIntel;
use telescope_core::models::{
    IntelNetwork, NetworkDetail, NetworkScan, PaginatedScans, SearchResult,
};
use telescope_core::realtime::{self, RealtimeEvent, RealtimeHandle};
use telescope_core::view::annotations::{
    Annotation, AnnotationIndex, EntityType, ResolvedAnnotation, TagToggle, Target,
    annotation_save_color, annotations_by_target_key, annotations_from_entries, custom_tags,
    find_annotation, resolve_pilot_annotations, serialize_annotation_tags, toggle_tag,
};
use telescope_core::view::scan_input::ScanInputKind;

use crate::runtime;
use crate::services::Services;

pub enum IntelEvent {
    /// A scan was shared to the active network (by anyone).
    ScanShared(NetworkScan),
    Error(String),
}

/// UI mirror of [`IntelService`] state plus the realtime subscription for the
/// active network.
pub struct IntelStore {
    service: Arc<IntelService>,
    state: IntelState,
    annotations: AnnotationIndex,
    realtime: Option<(String, i64, RealtimeHandle)>,
    _watch: Task<()>,
    _realtime_events: Option<Task<()>>,
}

impl EventEmitter<IntelEvent> for IntelStore {}

impl IntelStore {
    fn new(cx: &mut Context<Self>) -> Self {
        let service = Services::get(cx).intel.clone();
        let mut changes = service.subscribe();
        let state = changes.borrow_and_update().clone();
        let watch = cx.spawn(async move |this, cx| {
            while changes.changed().await.is_ok() {
                let state = changes.borrow_and_update().clone();
                if this
                    .update(cx, |this, cx| this.set_state(state, cx))
                    .is_err()
                {
                    return;
                }
            }
        });

        let mut store = Self {
            annotations: annotations_by_target_key(annotations_from_entries(&state.entries)),
            service,
            state,
            realtime: None,
            _watch: watch,
            _realtime_events: None,
        };
        store.sync_realtime(cx);
        if store.is_authenticated() {
            store.fetch_networks(cx);
        }
        store
    }

    fn set_state(&mut self, state: IntelState, cx: &mut Context<Self>) {
        if state.entries.len() != self.state.entries.len()
            || state
                .entries
                .iter()
                .zip(&self.state.entries)
                .any(|(a, b)| a.id != b.id || a.label != b.label || a.notes != b.notes)
        {
            self.annotations = annotations_by_target_key(annotations_from_entries(&state.entries));
        }
        self.state = state;
        self.sync_realtime(cx);
        cx.notify();
    }

    pub fn is_authenticated(&self) -> bool {
        self.state.api_token.is_some()
    }

    pub fn networks(&self) -> &[IntelNetwork] {
        &self.state.networks
    }

    pub fn selected_network(&self) -> Option<&NetworkDetail> {
        self.state.selected_network.as_ref()
    }

    pub fn active_network_id(&self) -> Option<i64> {
        self.state.active_network_ids.first().copied()
    }

    pub fn active_network(&self) -> Option<&IntelNetwork> {
        let id = self.active_network_id()?;
        self.state.networks.iter().find(|n| n.id == id)
    }

    pub fn resolve(&self, pilot: &PilotIntel) -> Vec<ResolvedAnnotation<'_>> {
        resolve_pilot_annotations(pilot, &self.annotations)
    }

    /// Custom tags in use on `network_id`: from looked-up entries, plus the
    /// network's full annotation list when it is the selected network.
    pub fn network_custom_tags(&self, network_id: i64) -> Vec<(String, String)> {
        let mut index = self.annotations.clone();
        if let Some(detail) = self.selected_network().filter(|n| n.id == network_id) {
            for annotation in detail
                .entries
                .iter()
                .filter_map(|e| Annotation::from_entry_detail(e, detail.id, &detail.name))
            {
                index
                    .entry((annotation.target_type, annotation.target_id))
                    .or_default()
                    .push(annotation);
            }
        }
        custom_tags(&index, network_id)
    }

    pub fn annotation_index(&self) -> &AnnotationIndex {
        &self.annotations
    }

    /// Restarts the websocket whenever the token or the active network
    /// changes, so it always authenticates with the current token.
    fn sync_realtime(&mut self, cx: &mut Context<Self>) {
        let wanted = self.state.api_token.clone().zip(self.active_network_id());
        let current = self
            .realtime
            .as_ref()
            .map(|(token, network, _)| (token.clone(), *network));
        if wanted == current {
            return;
        }
        self.realtime = None;
        self._realtime_events = None;
        let Some((token, network_id)) = wanted else {
            return;
        };

        info!("[Realtime] Subscribing to network {}", network_id);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let handle = {
            let _guard = runtime::handle().enter();
            realtime::spawn(token.clone(), network_id, tx)
        };
        self.realtime = Some((token, network_id, handle));
        self._realtime_events = Some(cx.spawn(async move |this, cx| {
            while let Some(event) = rx.recv().await {
                if this
                    .update(cx, |this, cx| this.on_realtime(event, cx))
                    .is_err()
                {
                    return;
                }
            }
        }));
    }

    fn on_realtime(&mut self, event: RealtimeEvent, cx: &mut Context<Self>) {
        match event {
            RealtimeEvent::EntryChanged { network_id } => {
                self.select_network(network_id, cx);
            }
            RealtimeEvent::EntryDeleted { entry_id, .. } => {
                let service = self.service.clone();
                runtime::spawn(async move { service.forget_entry(entry_id).await }).detach();
            }
            RealtimeEvent::ScanShared {
                network_id,
                scan_id,
            } => {
                let service = self.service.clone();
                self.run(
                    async move { service.fetch_scan(network_id, scan_id).await },
                    cx,
                    |_, scan, cx| cx.emit(IntelEvent::ScanShared(scan)),
                );
            }
            RealtimeEvent::Connected | RealtimeEvent::Disconnected => {}
        }
    }

    /// Runs a service call on tokio and reports failures as an
    /// [`IntelEvent::Error`].
    fn run<T, F>(
        &self,
        future: F,
        cx: &mut Context<Self>,
        on_ok: impl FnOnce(&mut Self, T, &mut Context<Self>) + 'static,
    ) where
        T: Send + 'static,
        F: Future<Output = Result<T, String>> + Send + 'static,
    {
        let task = runtime::spawn(future);
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| match result {
                Ok(value) => on_ok(this, value, cx),
                Err(e) => {
                    warn!("[Intel] {}", e);
                    cx.emit(IntelEvent::Error(e));
                }
            });
        })
        .detach();
    }

    pub fn call<T, F>(
        &self,
        f: impl FnOnce(Arc<IntelService>) -> F,
    ) -> runtime::TokioTask<Result<T, String>>
    where
        T: Send + 'static,
        F: Future<Output = Result<T, String>> + Send + 'static,
    {
        runtime::spawn(f(self.service.clone()))
    }

    pub fn set_api_token(&mut self, token: String, cx: &mut Context<Self>) {
        let service = self.service.clone();
        self.run(
            async move { service.set_api_token(token).await },
            cx,
            |this, (), cx| this.after_networks_loaded(cx),
        );
    }

    pub fn logout(&mut self, _cx: &mut Context<Self>) {
        let service = self.service.clone();
        runtime::spawn(async move { service.logout().await }).detach();
    }

    pub fn fetch_networks(&mut self, cx: &mut Context<Self>) {
        let service = self.service.clone();
        self.run(
            async move { service.fetch_networks().await },
            cx,
            |this, (), cx| this.after_networks_loaded(cx),
        );
    }

    /// Re-selects the active network, or drops it if it no longer exists.
    fn after_networks_loaded(&mut self, cx: &mut Context<Self>) {
        let Some(active) = self.active_network_id() else {
            return;
        };
        if self.state.networks.iter().any(|n| n.id == active) {
            self.select_network(active, cx);
        } else {
            self.set_active_network(None, cx);
        }
    }

    pub fn set_active_network(&mut self, id: Option<i64>, cx: &mut Context<Self>) {
        let service = self.service.clone();
        self.run(
            async move {
                service
                    .set_active_network_ids(id.into_iter().collect())
                    .await;
                match id {
                    Some(id) => service.select_network(id).await.map(|_| ()),
                    None => {
                        service.clear_selected_network().await;
                        Ok(())
                    }
                }
            },
            cx,
            |_, (), _| {},
        );
    }

    pub fn select_network(&mut self, id: i64, cx: &mut Context<Self>) {
        let service = self.service.clone();
        self.run(
            async move { service.select_network(id).await },
            cx,
            |_, _, _| {},
        );
    }

    pub fn clear_selected_network(&mut self, _cx: &mut Context<Self>) {
        let service = self.service.clone();
        runtime::spawn(async move { service.clear_selected_network().await }).detach();
    }

    pub fn create_network(&mut self, name: String, cx: &mut Context<Self>) {
        let service = self.service.clone();
        self.run(
            async move { service.create_network(&name).await },
            cx,
            |_, _, _| {},
        );
    }

    pub fn delete_network(&mut self, id: i64, cx: &mut Context<Self>) {
        let service = self.service.clone();
        self.run(
            async move { service.delete_network(id).await },
            cx,
            |_, (), _| {},
        );
    }

    pub fn clear_entries(&mut self, _cx: &mut Context<Self>) {
        let service = self.service.clone();
        runtime::spawn(async move { service.clear_entries().await }).detach();
    }

    pub fn lookup_intel(&mut self, ids: Vec<i64>, cx: &mut Context<Self>) {
        if !self.is_authenticated() || ids.is_empty() {
            return;
        }
        let service = self.service.clone();
        self.run(
            async move { service.lookup_intel(&ids).await },
            cx,
            |_, (), _| {},
        );
    }

    /// Creates, updates or (with no tags and no note) deletes an annotation.
    pub fn save_annotation(
        &mut self,
        network_id: i64,
        existing: Option<i64>,
        target: Target,
        tags: Vec<String>,
        note: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let note = note.filter(|n| !n.trim().is_empty());
        if tags.is_empty() && note.is_none() {
            if let Some(entry_id) = existing {
                self.remove_entry(network_id, entry_id, cx);
            }
            return;
        }
        let input = EntryInput {
            entity_type: target.entity_type.as_str().to_string(),
            entity_id: target.id,
            entity_name: target.name,
            color: annotation_save_color(&tags).to_string(),
            label: serialize_annotation_tags(&tags),
            notes: note,
        };
        let service = self.service.clone();
        match existing {
            Some(entry_id) => self.run(
                async move { service.update_entry(network_id, entry_id, input).await },
                cx,
                |_, _, _| {},
            ),
            None => self.run(
                async move { service.add_entry(network_id, input).await },
                cx,
                |_, _, _| {},
            ),
        }
    }

    /// Adds or removes one tag on `target`'s annotation in `network_id`.
    pub fn toggle_tag(
        &mut self,
        network_id: i64,
        target: Target,
        tag: &str,
        cx: &mut Context<Self>,
    ) {
        let existing = find_annotation(&self.annotations, network_id, &target).cloned();
        match toggle_tag(existing.as_ref(), tag) {
            TagToggle::Create(tags) => {
                self.save_annotation(network_id, None, target, tags, None, cx)
            }
            TagToggle::Update { entry_id, tags } => {
                let note = existing.and_then(|a| a.note);
                self.save_annotation(network_id, Some(entry_id), target, tags, note, cx)
            }
            TagToggle::Remove { entry_id } => self.remove_entry(network_id, entry_id, cx),
        }
    }

    pub fn remove_entry(&mut self, network_id: i64, entry_id: i64, cx: &mut Context<Self>) {
        let service = self.service.clone();
        self.run(
            async move { service.remove_entry(network_id, entry_id).await },
            cx,
            |_, (), _| {},
        );
    }

    pub fn add_access(&mut self, network_id: i64, input: AccessInput, cx: &mut Context<Self>) {
        let service = self.service.clone();
        self.run(
            async move { service.add_access(network_id, input).await },
            cx,
            |_, _, _| {},
        );
    }

    pub fn remove_access(&mut self, network_id: i64, access_id: i64, cx: &mut Context<Self>) {
        let service = self.service.clone();
        self.run(
            async move { service.remove_access(network_id, access_id).await },
            cx,
            |_, (), _| {},
        );
    }

    /// Shares a scan to the active network; failures are only logged, as the
    /// scan itself already ran locally.
    pub fn share_scan(&mut self, kind: ScanInputKind, text: &str, _cx: &mut Context<Self>) {
        let Some(network_id) = self.active_network_id().filter(|_| self.is_authenticated()) else {
            return;
        };
        let scan_type = match kind {
            ScanInputKind::Local => "local",
            ScanInputKind::Dscan => "dscan",
        };
        let text = text.to_string();
        let service = self.service.clone();
        runtime::spawn(async move {
            if let Err(e) = service.share_scan(network_id, scan_type, &text, None).await {
                error!("[Intel] Failed to share scan: {}", e);
            }
        })
        .detach();
    }

    pub fn fetch_scans(
        &self,
        network_id: i64,
        page: i64,
    ) -> runtime::TokioTask<Result<PaginatedScans, String>> {
        self.call(move |service| async move { service.fetch_scans(network_id, page).await })
    }

    pub fn search_entities(
        &self,
        query: String,
        category: Option<EntityType>,
    ) -> runtime::TokioTask<Result<Vec<SearchResult>, String>> {
        self.call(move |service| async move {
            service
                .search_entities(&query, category.map(EntityType::as_str))
                .await
        })
    }
}

pub fn init(cx: &mut App) -> Entity<IntelStore> {
    cx.new(IntelStore::new)
}
