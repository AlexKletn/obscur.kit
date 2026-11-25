//! Executor for trusted native plugin packages.
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(any(unix, windows))]
mod native_callback_status;
#[cfg(any(unix, windows))]
mod native_plugin;
#[cfg(any(unix, windows))]
pub use native_plugin::NativePlugin;
#[cfg(any(unix, windows))]
mod native_dispatch_state;
mod native_plugin_handle;
#[cfg(any(unix, windows))]
pub use native_plugin_handle::NativePluginHandle;
#[cfg(any(unix, windows))]
mod native_service_session;
#[cfg(any(unix, windows))]
pub use native_service_session::NativeServiceSession;
#[cfg(any(unix, windows))]
pub use obscur_plugin_abi::function_table::HostFunctionTable;
#[cfg(any(unix, windows))]
pub use obscur_plugin_abi::value::{AbiBuffer, AbiCallResult, AbiHandle, AbiSlice, AbiStatus};
