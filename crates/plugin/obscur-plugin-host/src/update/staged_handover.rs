//! Provisional replacement of an active plugin.

mod previous_plugin;

use previous_plugin::PreviousPlugin;

use obscur_plugin_api::status::PluginApiError;
use obscur_plugin_model::identity::PluginId;

use crate::contribution::MountedView;
use crate::manager::{PluginManager, PluginRecord};

/// Retains the old instance and routes until the host confirms the first frame, and restores them if the trial is abandoned.
#[must_use = "dropping a handover rolls it back"]
pub struct StagedHandover<'a> {
    manager: &'a mut PluginManager,
    previous: Option<PreviousPlugin>,
}

impl<'a> StagedHandover<'a> {
    pub(crate) fn new(
        manager: &'a mut PluginManager,
        id: PluginId,
        record: PluginRecord,
        pages: Vec<MountedView>,
    ) -> Self {
        Self {
            manager,
            previous: Some(PreviousPlugin { id, record, pages }),
        }
    }

    /// Returns the provisional manager state for rendering the first frame.
    #[must_use]
    pub fn trial(&self) -> &PluginManager {
        self.manager
    }

    #[cfg(feature = "plugin-wasm")]
    pub(crate) fn attach_wasm(
        &mut self,
        handle: obscur_plugin_executor_wasm::executor::ManagedWasmHandle,
    ) {
        self.manager.attach_wasm(handle);
    }

    /// Confirms a successful first frame and deactivates the previous instance.
    /// A faulted WASM replacement rolls back and reports the guest fault.
    /// A teardown error leaves the confirmed replacement active.
    pub fn commit(mut self) -> Result<(), PluginApiError> {
        #[cfg(feature = "plugin-wasm")]
        if self
            .previous
            .as_ref()
            .is_some_and(|previous| self.manager.wasm_faulted(&previous.id))
        {
            return self.restore();
        }
        if let Some(PreviousPlugin { id, record, pages }) = self.previous.take() {
            drop(pages);
            self.manager.deactivate_handover_record(&id, record)?;
        }
        Ok(())
    }

    /// Restores the previous instance and reports any candidate teardown error.
    pub fn rollback(mut self) -> Result<(), PluginApiError> {
        self.restore()
    }

    fn restore(&mut self) -> Result<(), PluginApiError> {
        if let Some(PreviousPlugin { id, record, pages }) = self.previous.take() {
            self.manager.restore_handover(id, record, pages)?;
        }
        Ok(())
    }
}

impl Drop for StagedHandover<'_> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
