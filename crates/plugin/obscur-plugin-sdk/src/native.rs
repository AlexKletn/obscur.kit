//! Version-1 SDK glue for a trusted native plugin.

pub use obscur_plugin_abi::{
    boundary::PanicBoundary,
    function_table::{HostFunctionTable, PluginFunctionTable},
    value::{AbiBuffer, AbiCallResult, AbiHandle, AbiSlice, AbiStatus},
};

/// Exports the optional native message callback. It takes an `AbiHandle` and
/// a borrowed byte slice and returns `Result<Vec<u8>, AbiStatus>`.
/// Invoke this macro once per plugin library and reserve its entry-point symbol.
#[macro_export]
macro_rules! export_native_dispatch {
    ($dispatch:path $(,)?) => {
        // SAFETY: the plugin library reserves this versioned symbol for this ABI.
        #[unsafe(no_mangle)]
        #[doc = "Version-1 native message entry point."]
        #[doc = ""]
        #[doc = "# Safety"]
        #[doc = "The caller must supply live, initialized input and exclusively writable output."]
        #[doc = "The buffers must be disjoint and valid for the duration of the call."]
        pub unsafe extern "C" fn obscur_plugin_dispatch_v1(
            handle: $crate::native::AbiHandle,
            input: $crate::native::AbiSlice,
            output: $crate::native::AbiBuffer,
        ) -> $crate::native::AbiCallResult {
            let mut written = 0;
            let status = $crate::native::PanicBoundary::status(|| {
                const LIMIT: usize = $crate::native::HostFunctionTable::SERVICE_PACKET_BYTES;
                // SAFETY: valid disjoint storage is required by this entry point.
                let Some(input) = (unsafe { input.as_bytes(LIMIT) }) else {
                    return $crate::native::AbiStatus::REJECTED;
                };
                match $dispatch(handle, input) {
                    // SAFETY: the entry point requires writable output disjoint from the reply.
                    Ok(reply) if reply.len() <= LIMIT && unsafe { output.write(&reply) } => {
                        written = reply.len();
                        $crate::native::AbiStatus::OK
                    }
                    Err(status) if status != $crate::native::AbiStatus::OK => status,
                    _ => $crate::native::AbiStatus::REJECTED,
                }
            });
            $crate::native::AbiCallResult { status, written }
        }
    };
}

#[doc = include_str!("../docs/api/native.doc/export_native_plugin.md")]
#[macro_export]
macro_rules! export_native_plugin {
    ($initialize:path, $deactivate:path, host = $bind:path $(,)?) => {
        $crate::export_native_plugin!($initialize, $deactivate);

        // SAFETY: the plugin library reserves this versioned symbol for this ABI.
        #[unsafe(no_mangle)]
        #[doc = "Version-1 native host table binding entry point."]
        pub extern "C" fn obscur_plugin_bind_host_v1(
            handle: $crate::native::AbiHandle,
            table: $crate::native::HostFunctionTable,
        ) -> $crate::native::AbiStatus {
            if !table.supports_v1() {
                return $crate::native::AbiStatus::REJECTED;
            }
            $crate::native::PanicBoundary::status(|| $bind(handle, table))
        }
    };
    ($initialize:path, $deactivate:path $(,)?) => {
        // SAFETY: the plugin library reserves this versioned symbol for this ABI.
        #[unsafe(no_mangle)]
        #[doc = "Version-1 native plugin lifecycle entry point."]
        pub extern "C" fn obscur_plugin_lifecycle_v1() -> $crate::native::PluginFunctionTable {
            extern "C" fn __obscur_initialize_v1(
                handle: $crate::native::AbiHandle,
            ) -> $crate::native::AbiStatus {
                $crate::native::PanicBoundary::status(|| $initialize(handle))
            }

            extern "C" fn __obscur_deactivate_v1(
                handle: $crate::native::AbiHandle,
            ) -> $crate::native::AbiStatus {
                $crate::native::PanicBoundary::status(|| $deactivate(handle))
            }

            $crate::native::PluginFunctionTable::new(__obscur_initialize_v1, __obscur_deactivate_v1)
        }
    };
}
