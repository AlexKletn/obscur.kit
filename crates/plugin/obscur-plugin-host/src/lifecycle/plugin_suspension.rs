//! Retained views and the start of a plugin suspension.

use std::time::{Duration, Instant};

use crate::contribution::MountedView;

/// Records when a retained plugin stopped presenting its contributions.
pub struct PluginSuspension {
    since: Instant,
    pub(crate) views: Vec<MountedView>,
}

impl PluginSuspension {
    pub(crate) fn new(views: Vec<MountedView>) -> Self {
        Self {
            since: Instant::now(),
            views,
        }
    }

    pub(crate) fn expired(&self, now: Instant, idle_for: Duration) -> bool {
        now.checked_duration_since(self.since)
            .is_some_and(|elapsed| elapsed >= idle_for)
    }
}
