//! SDE index lifecycle state machine with effects-as-data.
//!
//! `crate::sde::ensure_sde_index` drives this machine: it seeds the phase
//! from the on-disk cache, feeds `CheckRequested`, executes each returned
//! effect (network/disk work), and feeds the outcome back in as the next
//! event until no effects remain.
//!
//! Transition table (all other (phase, event) pairs are stale/spurious and
//! leave the phase unchanged with no effects):
//!
//! | Phase              | Event               | Next phase        | Effects            |
//! |--------------------|---------------------|-------------------|--------------------|
//! | Missing            | CheckRequested      | Checking(None)    | FetchRemoteBuild   |
//! | Ready(b)           | CheckRequested      | Checking(Some(b)) | FetchRemoteBuild   |
//! | Failed(_)          | CheckRequested      | Checking(None)    | FetchRemoteBuild   |
//! | Updating(t)        | CheckRequested      | Updating(t)       | (denied, none)     |
//! | Checking(Some(b))  | RemoteBuild(b)      | Ready(b)          | (up to date, none) |
//! | Checking(c)        | RemoteBuild(l ≠ c)  | Updating(l)       | StartUpdate(l)     |
//! | Checking(Some(b))  | UpToDate            | Ready(b)          | (keep cache, none) |
//! | Checking(None)     | UpToDate            | Failed(msg)       | (none)             |
//! | Updating(_)        | UpdateFinished(b)   | Ready(b)          | InvalidateIndex    |
//! | Updating(_)        | UpdateFailed(e)     | Failed(e)         | (none)             |

/// Error produced when the remote build can't be determined and there is no
/// cached index to fall back on (same message the old code returned).
pub const NO_BUILD_ERROR: &str = "Unable to determine latest SDE build and no cached index exists";

/// Build numbers are `i64` to match the cache file / `SdeStatus` fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdePhase {
    /// No cached index on disk.
    Missing,
    /// A cached index for `build` exists and is usable.
    Ready { build: i64 },
    /// Determining the latest remote build; `current` is the cached build.
    Checking { current: Option<i64> },
    /// Downloading/building the index for `target`.
    Updating { target: i64 },
    /// The last check or update failed.
    Failed { error: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdeEvent {
    /// `ensure_sde_index` was invoked.
    CheckRequested,
    /// The remote build number was determined.
    RemoteBuild(i64),
    /// The remote build could not be determined; keep whatever we have.
    UpToDate,
    /// Download + index build for this build number completed.
    UpdateFinished(i64),
    /// Download + index build failed.
    UpdateFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdeEffect {
    /// HEAD the SDE URL to learn the latest build number.
    FetchRemoteBuild,
    /// Download the archive and rebuild the on-disk index cache.
    StartUpdate(i64),
    /// Drop the in-memory index so the next read reloads the new cache.
    InvalidateIndex,
}

pub fn step(phase: SdePhase, event: SdeEvent) -> (SdePhase, Vec<SdeEffect>) {
    match (phase, event) {
        // A check while an update is in flight is denied (no-op) — the
        // driver's mutex already serializes callers, this encodes it.
        (SdePhase::Updating { target }, SdeEvent::CheckRequested) => {
            (SdePhase::Updating { target }, vec![])
        }
        (SdePhase::Missing, SdeEvent::CheckRequested) => (
            SdePhase::Checking { current: None },
            vec![SdeEffect::FetchRemoteBuild],
        ),
        (SdePhase::Ready { build }, SdeEvent::CheckRequested) => (
            SdePhase::Checking {
                current: Some(build),
            },
            vec![SdeEffect::FetchRemoteBuild],
        ),
        // Failure recovery: a new check starts from scratch.
        (SdePhase::Failed { .. }, SdeEvent::CheckRequested) => (
            SdePhase::Checking { current: None },
            vec![SdeEffect::FetchRemoteBuild],
        ),
        (SdePhase::Checking { current }, SdeEvent::RemoteBuild(latest)) => {
            if current == Some(latest) {
                (SdePhase::Ready { build: latest }, vec![])
            } else {
                (
                    SdePhase::Updating { target: latest },
                    vec![SdeEffect::StartUpdate(latest)],
                )
            }
        }
        (
            SdePhase::Checking {
                current: Some(build),
            },
            SdeEvent::UpToDate,
        ) => (SdePhase::Ready { build }, vec![]),
        (SdePhase::Checking { current: None }, SdeEvent::UpToDate) => (
            SdePhase::Failed {
                error: NO_BUILD_ERROR.to_string(),
            },
            vec![],
        ),
        (SdePhase::Updating { .. }, SdeEvent::UpdateFinished(build)) => {
            (SdePhase::Ready { build }, vec![SdeEffect::InvalidateIndex])
        }
        (SdePhase::Updating { .. }, SdeEvent::UpdateFailed(error)) => {
            (SdePhase::Failed { error }, vec![])
        }
        // Stale/spurious events never change the phase.
        (phase, _) => (phase, vec![]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_from_missing_fetches_remote_build() {
        let (phase, effects) = step(SdePhase::Missing, SdeEvent::CheckRequested);
        assert_eq!(phase, SdePhase::Checking { current: None });
        assert_eq!(effects, vec![SdeEffect::FetchRemoteBuild]);
    }

    #[test]
    fn check_from_ready_carries_current_build() {
        let (phase, effects) = step(SdePhase::Ready { build: 100 }, SdeEvent::CheckRequested);
        assert_eq!(phase, SdePhase::Checking { current: Some(100) });
        assert_eq!(effects, vec![SdeEffect::FetchRemoteBuild]);
    }

    #[test]
    fn check_from_failed_recovers_from_scratch() {
        let failed = SdePhase::Failed {
            error: "boom".into(),
        };
        let (phase, effects) = step(failed, SdeEvent::CheckRequested);
        assert_eq!(phase, SdePhase::Checking { current: None });
        assert_eq!(effects, vec![SdeEffect::FetchRemoteBuild]);
    }

    #[test]
    fn check_while_updating_is_denied() {
        let (phase, effects) = step(SdePhase::Updating { target: 200 }, SdeEvent::CheckRequested);
        assert_eq!(phase, SdePhase::Updating { target: 200 });
        assert!(effects.is_empty());
    }

    #[test]
    fn matching_remote_build_is_up_to_date() {
        let checking = SdePhase::Checking { current: Some(100) };
        let (phase, effects) = step(checking, SdeEvent::RemoteBuild(100));
        assert_eq!(phase, SdePhase::Ready { build: 100 });
        assert!(effects.is_empty());
    }

    #[test]
    fn newer_remote_build_starts_update() {
        let checking = SdePhase::Checking { current: Some(100) };
        let (phase, effects) = step(checking, SdeEvent::RemoteBuild(200));
        assert_eq!(phase, SdePhase::Updating { target: 200 });
        assert_eq!(effects, vec![SdeEffect::StartUpdate(200)]);
    }

    #[test]
    fn remote_build_with_no_cache_starts_update() {
        let (phase, effects) = step(
            SdePhase::Checking { current: None },
            SdeEvent::RemoteBuild(200),
        );
        assert_eq!(phase, SdePhase::Updating { target: 200 });
        assert_eq!(effects, vec![SdeEffect::StartUpdate(200)]);
    }

    #[test]
    fn up_to_date_with_cache_stays_ready_on_unknown_remote() {
        // Remote build undeterminable but a cache exists: keep serving it.
        let checking = SdePhase::Checking { current: Some(100) };
        let (phase, effects) = step(checking, SdeEvent::UpToDate);
        assert_eq!(phase, SdePhase::Ready { build: 100 });
        assert!(effects.is_empty());
    }

    #[test]
    fn up_to_date_without_cache_fails() {
        let (phase, effects) = step(SdePhase::Checking { current: None }, SdeEvent::UpToDate);
        assert_eq!(
            phase,
            SdePhase::Failed {
                error: NO_BUILD_ERROR.to_string()
            }
        );
        assert!(effects.is_empty());
    }

    #[test]
    fn update_finished_becomes_ready_and_invalidates_index() {
        let (phase, effects) = step(
            SdePhase::Updating { target: 200 },
            SdeEvent::UpdateFinished(200),
        );
        assert_eq!(phase, SdePhase::Ready { build: 200 });
        assert_eq!(effects, vec![SdeEffect::InvalidateIndex]);
    }

    #[test]
    fn update_failed_records_the_error() {
        let (phase, effects) = step(
            SdePhase::Updating { target: 200 },
            SdeEvent::UpdateFailed("download died".into()),
        );
        assert_eq!(
            phase,
            SdePhase::Failed {
                error: "download died".into()
            }
        );
        assert!(effects.is_empty());
    }

    #[test]
    fn stray_events_leave_phase_unchanged() {
        // Events that don't belong to the current phase are ignored.
        let cases = [
            (SdePhase::Missing, SdeEvent::RemoteBuild(1)),
            (SdePhase::Missing, SdeEvent::UpdateFinished(1)),
            (SdePhase::Ready { build: 5 }, SdeEvent::UpToDate),
            (
                SdePhase::Ready { build: 5 },
                SdeEvent::UpdateFailed("x".into()),
            ),
            (
                SdePhase::Checking { current: None },
                SdeEvent::UpdateFinished(9),
            ),
            (
                SdePhase::Checking { current: None },
                SdeEvent::CheckRequested,
            ),
            (SdePhase::Updating { target: 9 }, SdeEvent::RemoteBuild(9)),
            (SdePhase::Updating { target: 9 }, SdeEvent::UpToDate),
            (
                SdePhase::Failed { error: "e".into() },
                SdeEvent::RemoteBuild(9),
            ),
        ];
        for (phase, event) in cases {
            let (next, effects) = step(phase.clone(), event);
            assert_eq!(next, phase);
            assert!(effects.is_empty());
        }
    }

    #[test]
    fn full_update_cycle_walks_the_happy_path() {
        let (phase, _) = step(SdePhase::Ready { build: 100 }, SdeEvent::CheckRequested);
        let (phase, effects) = step(phase, SdeEvent::RemoteBuild(200));
        assert_eq!(effects, vec![SdeEffect::StartUpdate(200)]);
        let (phase, effects) = step(phase, SdeEvent::UpdateFinished(200));
        assert_eq!(phase, SdePhase::Ready { build: 200 });
        assert_eq!(effects, vec![SdeEffect::InvalidateIndex]);
    }

    #[test]
    fn failure_then_recheck_can_succeed() {
        let (phase, _) = step(SdePhase::Missing, SdeEvent::CheckRequested);
        let (phase, _) = step(phase, SdeEvent::RemoteBuild(200));
        let (phase, _) = step(phase, SdeEvent::UpdateFailed("net".into()));
        assert!(matches!(phase, SdePhase::Failed { .. }));

        let (phase, effects) = step(phase, SdeEvent::CheckRequested);
        assert_eq!(phase, SdePhase::Checking { current: None });
        assert_eq!(effects, vec![SdeEffect::FetchRemoteBuild]);
        let (phase, _) = step(phase, SdeEvent::RemoteBuild(200));
        let (phase, _) = step(phase, SdeEvent::UpdateFinished(200));
        assert_eq!(phase, SdePhase::Ready { build: 200 });
    }
}
