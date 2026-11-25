use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use obscur_plugin_model::identity::PluginId;

use super::WasmPluginInstance;
use super::instance_lifetime::InstanceLifetime;
use super::managed_wasm_plugin::lifecycle_error;
use crate::error::WasmExecutorError;

/// Host-owned message handle to one managed WASM instance.
#[derive(Clone)]
pub struct ManagedWasmHandle {
    id: PluginId,
    instance: Weak<RefCell<Option<WasmPluginInstance>>>,
    lifetime: Rc<InstanceLifetime>,
}

impl ManagedWasmHandle {
    pub(super) fn new(
        id: PluginId,
        instance: Weak<RefCell<Option<WasmPluginInstance>>>,
        lifetime: Rc<InstanceLifetime>,
    ) -> Self {
        Self {
            id,
            instance,
            lifetime,
        }
    }

    /// Returns the plugin ID from the verified manifest.
    #[must_use]
    pub const fn id(&self) -> &PluginId {
        &self.id
    }

    /// Reports whether the plugin instance has been permanently withdrawn.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.lifetime.is_closed()
    }

    /// Reports whether message delivery is temporarily suspended.
    #[must_use]
    pub fn is_suspended(&self) -> bool {
        self.lifetime.is_suspended()
    }

    /// Waits for deactivation, rollback, or disposal of the managed instance.
    pub async fn closed(&self) {
        self.lifetime.wait_closed().await;
    }

    /// Exchanges bytes on the thread that owns the plugin manager.
    ///
    /// Returns `Inactive` after the instance is withdrawn and `Busy` for
    /// reentrant calls. The handle does not keep an instance alive.
    pub fn exchange(&self, request: &[u8]) -> Result<Vec<u8>, WasmExecutorError> {
        if self.lifetime.is_suspended() {
            return Err(WasmExecutorError::Inactive);
        }
        self.invoke_guest(|instance| instance.exchange(request))
    }

    pub(super) fn invoke_guest(
        &self,
        operation: impl FnOnce(&mut WasmPluginInstance) -> Result<Vec<u8>, WasmExecutorError>,
    ) -> Result<Vec<u8>, WasmExecutorError> {
        let result = self.with_instance(operation);
        if let Err(
            error @ (WasmExecutorError::OutOfFuel
            | WasmExecutorError::DeadlineExceeded
            | WasmExecutorError::GrowthDenied
            | WasmExecutorError::Trap(_)
            | WasmExecutorError::InvalidResponseLength
            | WasmExecutorError::ResponseTooLarge),
        ) = &result
        {
            self.lifetime.record_fault(lifecycle_error(error.clone()));
        }
        result
    }

    /// Reports whether a guest call permanently faulted this instance.
    #[must_use]
    pub fn is_faulted(&self) -> bool {
        self.lifetime.is_faulted()
    }

    /// Takes the pending guest request without keeping the instance borrowed.
    pub fn take_host_call(&self) -> Result<Option<Vec<u8>>, WasmExecutorError> {
        if self.lifetime.is_suspended() {
            return Err(WasmExecutorError::Inactive);
        }
        self.with_instance(|instance| Ok(instance.take_host_call()))
    }

    fn with_instance<T>(
        &self,
        operation: impl FnOnce(&mut WasmPluginInstance) -> Result<T, WasmExecutorError>,
    ) -> Result<T, WasmExecutorError> {
        let instance = self.instance.upgrade().ok_or(WasmExecutorError::Inactive)?;
        let mut state = instance
            .try_borrow_mut()
            .map_err(|_| WasmExecutorError::Busy)?;
        operation(state.as_mut().ok_or(WasmExecutorError::Inactive)?)
    }
}
