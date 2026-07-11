//! Progress reducer and batching policy for the pilot lookup pipeline.
//!
//! `commands::lookup` feeds one event per resolved pilot and embeds the
//! current progress snapshot in each emitted "pilot-batch" event, so the
//! values here must reproduce the original inline counting exactly:
//! `current = cache_hits + received`, with the cache pass always running
//! before any fetches. Batching is transport-only: it decides *when* to
//! emit, never *what* the progress values are.

use serde::Serialize;

/// Cadence of "pilot-batch" emissions. There is exactly one delivery mode:
/// every result — cached or fetched — streams out on this tick, so the UI
/// animates identically for hot-cache and fresh scans; a hot cache just
/// drains the queue at full cadence. The frontend throttles rendering to
/// ~100ms anyway, so finer-grained events are pure IPC overhead.
pub const BATCH_INTERVAL_MS: u64 = 100;

/// Upper bound on pilots per batch: a fully cached scan streams as a fast
/// sequence of bounded batches instead of one teleporting mega-batch.
pub const MAX_BATCH_SIZE: usize = 25;

/// Progress snapshot embedded in the "pilot-batch" event — field
/// names/shape are frozen (frontend contract).
#[derive(Clone, Serialize)]
pub struct LookupProgress {
    pub current: usize,
    pub total: usize,
    pub cache_hits: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupEvent {
    /// A pilot was served from the local cache.
    CacheHit,
    /// A pilot lookup returned from the network (success or error result).
    Fetched,
}

/// Counts lookup completions and produces the progress payload to emit.
#[derive(Debug, Clone, Copy)]
pub struct LookupTracker {
    total: usize,
    cache_hits: usize,
    received: usize,
}

impl LookupTracker {
    pub fn new(total: usize) -> Self {
        LookupTracker {
            total,
            cache_hits: 0,
            received: 0,
        }
    }

    /// Record one completed lookup.
    pub fn apply(&mut self, event: LookupEvent) {
        match event {
            LookupEvent::CacheHit => self.cache_hits += 1,
            LookupEvent::Fetched => self.received += 1,
        }
    }

    /// Current progress snapshot, computed on demand so batches can embed
    /// the progress as of their last buffered result.
    pub fn progress(&self) -> LookupProgress {
        LookupProgress {
            current: self.cache_hits + self.received,
            total: self.total,
            cache_hits: self.cache_hits,
        }
    }

    pub fn cache_hits(&self) -> usize {
        self.cache_hits
    }

    /// True once every pilot has been accounted for. The unified streaming
    /// driver tracks completion via its own queue/stream state; this stays
    /// for the tests documenting the tracker's completion semantics.
    #[cfg(test)]
    pub fn is_complete(&self) -> bool {
        self.cache_hits + self.received >= self.total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(tracker: &LookupTracker) -> (usize, usize, usize) {
        let progress = tracker.progress();
        (progress.current, progress.total, progress.cache_hits)
    }

    #[test]
    fn cache_hits_advance_current_and_cache_hits_together() {
        let mut tracker = LookupTracker::new(3);
        tracker.apply(LookupEvent::CacheHit);
        assert_eq!(payload(&tracker), (1, 3, 1));
        tracker.apply(LookupEvent::CacheHit);
        assert_eq!(payload(&tracker), (2, 3, 2));
        assert!(!tracker.is_complete());
    }

    #[test]
    fn fetches_advance_current_but_not_cache_hits() {
        let mut tracker = LookupTracker::new(2);
        tracker.apply(LookupEvent::Fetched);
        assert_eq!(payload(&tracker), (1, 2, 0));
        tracker.apply(LookupEvent::Fetched);
        assert_eq!(payload(&tracker), (2, 2, 0));
        assert!(tracker.is_complete());
    }

    #[test]
    fn cache_pass_then_fetches_reproduces_emitted_sequence() {
        // Mirrors the real driver: all cache hits first, then fetches.
        let mut tracker = LookupTracker::new(4);
        tracker.apply(LookupEvent::CacheHit);
        assert_eq!(payload(&tracker), (1, 4, 1));
        tracker.apply(LookupEvent::CacheHit);
        assert_eq!(payload(&tracker), (2, 4, 2));
        tracker.apply(LookupEvent::Fetched);
        assert_eq!(payload(&tracker), (3, 4, 2));
        assert!(!tracker.is_complete());
        tracker.apply(LookupEvent::Fetched);
        assert_eq!(payload(&tracker), (4, 4, 2));
        assert!(tracker.is_complete());
        assert_eq!(tracker.cache_hits(), 2);
    }

    #[test]
    fn interleaved_events_keep_current_consistent() {
        let mut tracker = LookupTracker::new(3);
        tracker.apply(LookupEvent::Fetched);
        assert_eq!(payload(&tracker), (1, 3, 0));
        tracker.apply(LookupEvent::CacheHit);
        assert_eq!(payload(&tracker), (2, 3, 1));
        tracker.apply(LookupEvent::Fetched);
        assert_eq!(payload(&tracker), (3, 3, 1));
        assert!(tracker.is_complete());
    }

    #[test]
    fn all_cache_hits_completes_without_fetches() {
        let mut tracker = LookupTracker::new(2);
        tracker.apply(LookupEvent::CacheHit);
        assert!(!tracker.is_complete());
        tracker.apply(LookupEvent::CacheHit);
        assert_eq!(payload(&tracker), (2, 2, 2));
        assert!(tracker.is_complete());
    }

    #[test]
    fn empty_input_is_complete_immediately() {
        let tracker = LookupTracker::new(0);
        assert!(tracker.is_complete());
        assert_eq!(tracker.cache_hits(), 0);
        assert_eq!(payload(&tracker), (0, 0, 0));
    }
}
