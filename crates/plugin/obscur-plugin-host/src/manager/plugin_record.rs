//! `PluginRecord` (skeleton stub for the spec-defined entity; not yet implemented).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Instant;

use crate::lifecycle::PluginSuspension;
use obscur_plugin_api::{plugin::PluginApi, status::PluginApiError};
use obscur_plugin_model::{lifecycle::PluginLifecycleState, manifest::Version};

/// The committed lifetime of one built-in plugin ID.
pub struct PluginRecord {
    pub(super) instance: Option<Box<dyn PluginApi>>,
    pub(super) version: Option<Version>,
    pub(super) suspension: Option<PluginSuspension>,
    #[cfg(feature = "plugin-wasm")]
    pub(super) wasm: Option<obscur_plugin_executor_wasm::executor::ManagedWasmHandle>,
}

impl PluginRecord {
    pub(crate) fn deactivate(self) -> Result<(), PluginApiError> {
        self.instance
            .map_or(Ok(()), |instance| Self::deactivate_instance(instance, None))
    }

    pub(super) fn deactivate_instance(
        mut instance: Box<dyn PluginApi>,
        deadline: Option<Instant>,
    ) -> Result<(), PluginApiError> {
        catch_unwind(AssertUnwindSafe(move || match deadline {
            Some(deadline) => instance.deactivate_until(deadline),
            None => instance.deactivate(),
        }))
        .map_err(|_| PluginApiError::Panicked)?
    }

    /// Returns the current committed lifecycle state.
    #[must_use]
    pub fn state(&self) -> PluginLifecycleState {
        if self.suspension.is_some() {
            PluginLifecycleState::Suspended
        } else if self.instance.is_some() {
            PluginLifecycleState::Active
        } else {
            PluginLifecycleState::Deactivated
        }
    }
}
