//! Owns candidate state until the host confirms the trial or drops the plan.

use std::io;

use crate::storage::{PluginStorage, StateStore};

/// Owns candidate state until the host confirms the trial or drops the plan.
#[derive(Debug)]
#[must_use = "dropping the plan discards the candidate state"]
pub struct RollbackPlan<'a> {
    current: &'a mut PluginStorage,
    candidate: PluginStorage,
}

impl<'a> RollbackPlan<'a> {
    pub(super) const fn new(current: &'a mut PluginStorage, candidate: PluginStorage) -> Self {
        Self { current, candidate }
    }

    /// Returns the isolated migrated state for trial activation.
    #[must_use]
    pub const fn candidate(&self) -> &PluginStorage {
        &self.candidate
    }

    /// Publishes the migrated state against the original persisted snapshot.
    /// The in-memory snapshot changes only after a successful exchange.
    pub fn commit_to(self, store: &(impl StateStore + ?Sized)) -> io::Result<bool> {
        if !store.compare_exchange(Some(self.current), &self.candidate)? {
            return Ok(false);
        }
        self.commit();
        Ok(true)
    }

    /// Adopts the migrated state after the host confirms the trial.
    pub fn commit(self) {
        *self.current = self.candidate;
    }
}
