use obscur_plugin_abi::value::{AbiBuffer, AbiCallResult, AbiHandle, AbiSlice};

use super::NativeServiceSession;

pub(super) type NativeDispatch =
    unsafe extern "C" fn(AbiHandle, AbiSlice, AbiBuffer) -> AbiCallResult;

pub(super) struct NativeDispatchState {
    pub(super) active: bool,
    pub(super) dispatch: Option<NativeDispatch>,
    pub(super) services: Option<NativeServiceSession>,
}
