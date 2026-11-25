//! Prepares a state migration against a declared schema range.

use std::panic::{AssertUnwindSafe, catch_unwind};

use obscur_plugin_api::status::PluginApiError;
use obscur_plugin_model::state::MigrationDescriptor;

use super::RollbackPlan;
use crate::storage::PluginStorage;

use super::StateMigrationError;

/// Prepares a state migration against a declared schema range.
#[derive(Debug)]
pub struct StateMigration;

impl StateMigration {
    /// Converts the state into an isolated candidate for trial activation.
    ///
    /// The host supplies the descriptor declared for the target schema and
    /// invokes this before initializing the candidate plugin.
    pub fn stage<'a>(
        current: &'a mut PluginStorage,
        descriptor: &MigrationDescriptor,
        migrate: impl FnOnce(&[u8]) -> Result<Vec<u8>, PluginApiError>,
    ) -> Result<RollbackPlan<'a>, StateMigrationError> {
        let found = current.schema().version();
        if !descriptor.source().matches(found) {
            return Err(StateMigrationError::IncompatibleSource {
                found: found.clone(),
                accepted: descriptor.source().clone(),
            });
        }
        let bytes = catch_unwind(AssertUnwindSafe(|| migrate(current.bytes())))
            .unwrap_or(Err(PluginApiError::Panicked))
            .map_err(StateMigrationError::Callback)?;
        let candidate = PluginStorage::new(
            current.namespace().clone(),
            descriptor.target().clone(),
            bytes,
        );
        Ok(RollbackPlan::new(current, candidate))
    }
}
