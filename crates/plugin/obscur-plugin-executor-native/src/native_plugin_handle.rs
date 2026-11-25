//! Non-owning dispatch handle for a loaded native plugin instance.

use std::{cell::RefCell, rc::Weak};

use obscur_plugin_abi::{
    function_table::HostFunctionTable,
    value::{AbiBuffer, AbiHandle, AbiSlice},
};
use obscur_plugin_api::status::PluginApiError;

use super::{native_callback_status::check_status, native_dispatch_state::NativeDispatchState};

/// Sends messages to an active native instance on its host thread.
#[derive(Clone)]
pub struct NativePluginHandle {
    pub(super) handle: AbiHandle,
    pub(super) state: Weak<RefCell<NativeDispatchState>>,
}

impl NativePluginHandle {
    /// Exchanges up to 64 KiB of request and response bytes. The instance must be active and export the native dispatch symbol.
    pub fn exchange(&self, request: &[u8]) -> Result<Vec<u8>, PluginApiError> {
        const LIMIT: usize = HostFunctionTable::SERVICE_PACKET_BYTES;
        let state = self.state.upgrade().ok_or(PluginApiError::NotActive)?;
        let state = state
            .try_borrow_mut()
            .map_err(|_| PluginApiError::Rejected("native plugin is busy".into()))?;
        if !state.active {
            return Err(PluginApiError::NotActive);
        }
        let dispatch = state
            .dispatch
            .ok_or_else(|| PluginApiError::Rejected("native dispatch is unavailable".into()))?;
        if request.len() > LIMIT {
            return Err(PluginApiError::Rejected(
                "native request exceeds the byte limit".into(),
            ));
        }
        let mut output = vec![0; LIMIT];
        let mut call = || {
            // SAFETY: the trusted loader pinned the callback and these buffers live for the call.
            unsafe {
                dispatch(
                    self.handle,
                    AbiSlice::from_bytes(request),
                    AbiBuffer::from_bytes(&mut output),
                )
            }
        };
        let result = match &state.services {
            Some(services) => services.enter(self.handle, call),
            None => call(),
        };
        check_status("dispatch", result.status)?;
        if result.written > output.len() {
            return Err(PluginApiError::Rejected(
                "invalid native reply length".into(),
            ));
        }
        output.truncate(result.written);
        Ok(output)
    }
}
