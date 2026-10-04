//! Entrance animations for content that streams in.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use gpui_kit::{
    Animation, AnimationExt as _, AnyElement, ElementId, IntoElement, Styled, ease_out_quint, px,
};

pub const ENTER: Duration = Duration::from_millis(260);
pub const GROW: Duration = Duration::from_millis(520);

/// How long after arriving an element still plays its entrance. A bit longer
/// than the animation so a late first frame doesn't skip it, short enough that
/// scrolling a virtual list back to a row doesn't replay it.
const FRESH: Duration = Duration::from_millis(600);

/// Remembers when each keyed item first showed up.
#[derive(Default)]
pub struct Arrivals(HashMap<u64, Instant>);

impl Arrivals {
    /// Records the arrival of every key not seen before and forgets the ones
    /// that are gone, so a new scan animates again.
    pub fn sync(&mut self, keys: impl IntoIterator<Item = u64>) {
        let now = Instant::now();
        let keys: HashSet<u64> = keys.into_iter().collect();
        self.0.retain(|key, _| keys.contains(key));
        for key in keys {
            self.0.entry(key).or_insert(now);
        }
    }

    pub fn is_fresh(&self, key: u64) -> bool {
        self.0.get(&key).copied().is_some_and(is_recent)
    }
}

pub fn is_recent(at: Instant) -> bool {
    at.elapsed() < FRESH
}

fn animation(duration: Duration) -> Animation {
    Animation::new(duration).with_easing(ease_out_quint())
}

// The wrapper is always present, even for items that aren't fresh, because it
// scopes the element ids beneath it; adding it later would reset their state.

/// Fades `element` in while sliding it up a few pixels, if it is `fresh`.
pub fn enter<E>(id: impl Into<ElementId>, element: E, fresh: bool) -> AnyElement
where
    E: IntoElement + Styled + 'static,
{
    element
        .with_animation(id, animation(ENTER), move |el, p| {
            let p = if fresh { p } else { 1. };
            el.opacity(p).relative().top(px((1. - p) * 6.))
        })
        .into_any_element()
}

/// Runs `animator` from 0 to 1 with a slow ease out if `fresh`, for bars
/// filling up and numbers counting up. Otherwise it is called with 1.
pub fn grow<E>(
    id: impl Into<ElementId>,
    element: E,
    fresh: bool,
    animator: impl Fn(E, f32) -> E + 'static,
) -> AnyElement
where
    E: IntoElement + 'static,
{
    element
        .with_animation(id, animation(GROW), move |el, p| {
            animator(el, if fresh { p } else { 1. })
        })
        .into_any_element()
}
