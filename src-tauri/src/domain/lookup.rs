//! Progress reducer for the pilot lookup pipeline.
//!
//! `commands::lookup` feeds one event per resolved pilot and emits the
//! returned payload verbatim as the "lookup-progress" event, so the values
//! here must reproduce the previous inline counting exactly:
//! `current = cache_hits + received`, with the cache pass always running
//! before any fetches.

use serde::Serialize;

/// Payload of the "lookup-progress" event — field names/shape are frozen
/// (frontend contract).
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

    /// Record one completed lookup and return the payload to emit for it.
    pub fn apply(&mut self, event: LookupEvent) -> LookupProgress {
        match event {
            LookupEvent::CacheHit => self.cache_hits += 1,
            LookupEvent::Fetched => self.received += 1,
        }
        LookupProgress {
            current: self.cache_hits + self.received,
            total: self.total,
            cache_hits: self.cache_hits,
        }
    }

    pub fn cache_hits(&self) -> usize {
        self.cache_hits
    }

    /// True once every pilot has been accounted for (nothing left to fetch).
    pub fn is_complete(&self) -> bool {
        self.cache_hits + self.received >= self.total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(progress: &LookupProgress) -> (usize, usize, usize) {
        (progress.current, progress.total, progress.cache_hits)
    }

    #[test]
    fn cache_hits_advance_current_and_cache_hits_together() {
        let mut tracker = LookupTracker::new(3);
        assert_eq!(payload(&tracker.apply(LookupEvent::CacheHit)), (1, 3, 1));
        assert_eq!(payload(&tracker.apply(LookupEvent::CacheHit)), (2, 3, 2));
        assert!(!tracker.is_complete());
    }

    #[test]
    fn fetches_advance_current_but_not_cache_hits() {
        let mut tracker = LookupTracker::new(2);
        assert_eq!(payload(&tracker.apply(LookupEvent::Fetched)), (1, 2, 0));
        assert_eq!(payload(&tracker.apply(LookupEvent::Fetched)), (2, 2, 0));
        assert!(tracker.is_complete());
    }

    #[test]
    fn cache_pass_then_fetches_reproduces_emitted_sequence() {
        // Mirrors the real driver: all cache hits first, then fetches.
        let mut tracker = LookupTracker::new(4);
        assert_eq!(payload(&tracker.apply(LookupEvent::CacheHit)), (1, 4, 1));
        assert_eq!(payload(&tracker.apply(LookupEvent::CacheHit)), (2, 4, 2));
        assert_eq!(payload(&tracker.apply(LookupEvent::Fetched)), (3, 4, 2));
        assert!(!tracker.is_complete());
        assert_eq!(payload(&tracker.apply(LookupEvent::Fetched)), (4, 4, 2));
        assert!(tracker.is_complete());
        assert_eq!(tracker.cache_hits(), 2);
    }

    #[test]
    fn interleaved_events_keep_current_consistent() {
        let mut tracker = LookupTracker::new(3);
        assert_eq!(payload(&tracker.apply(LookupEvent::Fetched)), (1, 3, 0));
        assert_eq!(payload(&tracker.apply(LookupEvent::CacheHit)), (2, 3, 1));
        assert_eq!(payload(&tracker.apply(LookupEvent::Fetched)), (3, 3, 1));
        assert!(tracker.is_complete());
    }

    #[test]
    fn all_cache_hits_completes_without_fetches() {
        let mut tracker = LookupTracker::new(2);
        tracker.apply(LookupEvent::CacheHit);
        assert!(!tracker.is_complete());
        let progress = tracker.apply(LookupEvent::CacheHit);
        assert_eq!(payload(&progress), (2, 2, 2));
        assert!(tracker.is_complete());
    }

    #[test]
    fn empty_input_is_complete_immediately() {
        let tracker = LookupTracker::new(0);
        assert!(tracker.is_complete());
        assert_eq!(tracker.cache_hits(), 0);
    }
}
