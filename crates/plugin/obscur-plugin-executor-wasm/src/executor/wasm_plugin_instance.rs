use std::time::Instant;

use obscur_plugin_package::package::PluginPackage;

use super::{WasmExecutor, WasmLimits, WasmMessageLimits};
use crate::error::WasmExecutorError;

const INITIALIZE: &str = "obscur_plugin_initialize_v1";
const DISPATCH: &str = "obscur_plugin_dispatch_v1";
const DEACTIVATE: &str = "obscur_plugin_deactivate_v1";
const SHUTDOWN_REQUESTED: &str = "obscur_plugin_shutdown_requested_v1";

/// An initialized WASM plugin using the version-1 message ABI.
pub struct WasmPluginInstance {
    executor: WasmExecutor,
    message_limits: WasmMessageLimits,
}

impl WasmPluginInstance {
    /// Validates the lifecycle exports, then initializes a fresh instance.
    ///
    /// The host passes initial state or configuration as bytes and receives
    /// the guest's initialization message for validation before publication.
    /// An invalid export, trap, or malformed reply discards the instance.
    pub fn activate(
        package: &PluginPackage,
        limits: WasmLimits,
        message_limits: WasmMessageLimits,
        initial: &[u8],
    ) -> Result<(Self, Vec<u8>), WasmExecutorError> {
        Self::initialize(
            WasmExecutor::load(package, limits)?,
            message_limits,
            initial,
        )
    }

    /// Activates an instance with the optional guest host-call import.
    pub fn activate_with_host_calls(
        package: &PluginPackage,
        limits: WasmLimits,
        message_limits: WasmMessageLimits,
        initial: &[u8],
        max_call_bytes: usize,
    ) -> Result<(Self, Vec<u8>), WasmExecutorError> {
        Self::initialize(
            WasmExecutor::load_with_host_calls(package, limits, max_call_bytes)?,
            message_limits,
            initial,
        )
    }

    fn initialize(
        mut executor: WasmExecutor,
        message_limits: WasmMessageLimits,
        initial: &[u8],
    ) -> Result<(Self, Vec<u8>), WasmExecutorError> {
        executor.export_i32(DISPATCH)?;
        executor.export_i32(DEACTIVATE)?;
        if executor.has_export(SHUTDOWN_REQUESTED) {
            executor.export_i32(SHUTDOWN_REQUESTED)?;
        }
        let reply = executor.exchange_with(INITIALIZE, initial, message_limits)?;
        Ok((
            Self {
                executor,
                message_limits,
            },
            reply,
        ))
    }

    /// Dispatches a message after initialization.
    pub fn exchange(&mut self, request: &[u8]) -> Result<Vec<u8>, WasmExecutorError> {
        self.executor.exchange(request, self.message_limits)
    }

    pub(super) fn shutdown_requested(
        &mut self,
        budget_ms: u64,
        deadline: Instant,
    ) -> Result<Vec<u8>, WasmExecutorError> {
        if !self.executor.has_export(SHUTDOWN_REQUESTED) {
            return Ok(Vec::new());
        }
        self.executor.exchange_with_deadline(
            SHUTDOWN_REQUESTED,
            &budget_ms.to_le_bytes(),
            self.message_limits,
            Some(deadline),
        )
    }

    /// Removes the pending host request for asynchronous dispatch.
    #[must_use]
    pub fn take_host_call(&mut self) -> Option<Vec<u8>> {
        self.executor.take_host_call()
    }

    /// Runs the guest's shutdown hook and consumes the instance.
    ///
    /// The return value may contain final state. Dropping the instance
    /// releases its memory without running guest code.
    pub fn deactivate(self, request: &[u8]) -> Result<Vec<u8>, WasmExecutorError> {
        self.deactivate_with_deadline(request, None)
    }

    pub(super) fn deactivate_with_deadline(
        mut self,
        request: &[u8],
        deadline: Option<Instant>,
    ) -> Result<Vec<u8>, WasmExecutorError> {
        let _ = self.executor.take_host_call();
        self.executor
            .exchange_with_deadline(DEACTIVATE, request, self.message_limits, deadline)
    }
}
