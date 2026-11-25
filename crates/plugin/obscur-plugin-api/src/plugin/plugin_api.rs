//! `PluginApi` (skeleton stub for the spec-defined entity; not yet implemented).

use std::time::Instant;

use obscur_plugin_model::lifecycle::ShutdownConcern;

use super::PluginRegistration;
use crate::status::PluginApiError;

/// The in-process contract for a trusted plugin compiled into the host application.
pub trait PluginApi {
    /// Stages contributions. The host publishes them after successful initialization and validation.
    fn initialize(&mut self, registration: &mut PluginRegistration) -> Result<(), PluginApiError>;

    #[doc = include_str!("../../docs/api/plugin/plugin_api.doc/deactivate.md")]
    fn deactivate(&mut self) -> Result<(), PluginApiError> {
        Ok(())
    }

    /// Pauses message delivery while retaining the initialized instance.
    fn suspend(&mut self) {}

    /// Restores message delivery for a retained instance.
    fn resume(&mut self) {}

    /// Releases plugin-owned state before the shutdown deadline. The default skips
    /// the callback once the deadline has elapsed. In-process implementations must
    /// return promptly. Isolated adapters can enforce the deadline during execution.
    fn deactivate_until(&mut self, deadline: Instant) -> Result<(), PluginApiError> {
        if Instant::now() >= deadline {
            return Err(PluginApiError::ExecutionTimedOut);
        }
        self.deactivate()
    }

    /// Reports pending work before shutdown. Return promptly and treat the deadline as an upper bound. The host decides whether to wait.
    fn shutdown_requested(
        &mut self,
        _deadline: Instant,
    ) -> Result<Option<ShutdownConcern>, PluginApiError> {
        Ok(None)
    }
}
