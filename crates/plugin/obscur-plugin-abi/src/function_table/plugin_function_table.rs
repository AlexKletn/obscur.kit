//! Version-1 native plugin lifecycle callbacks in C layout.

use crate::value::{AbiHandle, AbiStatus};

/// Version-1 native plugin lifecycle callbacks in C layout.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PluginFunctionTable {
    /// Version of the table ABI.
    pub abi_version: u32,
    /// Table size in bytes, including its header.
    pub byte_size: u32,
    /// Initialization hook for a host-assigned instance handle.
    pub initialize: Option<extern "C" fn(AbiHandle) -> AbiStatus>,
    /// Logical deactivation hook for the same instance handle.
    pub deactivate: Option<extern "C" fn(AbiHandle) -> AbiStatus>,
}

impl PluginFunctionTable {
    /// Symbol exported by version-1 native plugin libraries.
    pub const ENTRYPOINT: &str = "obscur_plugin_lifecycle_v1";
    /// Optional symbol for exchanging bytes with an active native instance.
    pub const DISPATCH_ENTRYPOINT: &str = "obscur_plugin_dispatch_v1";

    /// Native lifecycle table ABI version.
    pub const VERSION: u32 = 1;

    /// Builds a complete version-1 lifecycle table.
    #[must_use]
    pub const fn new(
        initialize: extern "C" fn(AbiHandle) -> AbiStatus,
        deactivate: extern "C" fn(AbiHandle) -> AbiStatus,
    ) -> Self {
        Self {
            abi_version: Self::VERSION,
            byte_size: size_of::<Self>() as u32,
            initialize: Some(initialize),
            deactivate: Some(deactivate),
        }
    }

    /// Checks the version, size, and required callbacks before invocation.
    #[must_use]
    pub const fn supports_v1(&self) -> bool {
        self.abi_version == Self::VERSION
            && self.byte_size as usize >= size_of::<Self>()
            && self.initialize.is_some()
            && self.deactivate.is_some()
    }
}
