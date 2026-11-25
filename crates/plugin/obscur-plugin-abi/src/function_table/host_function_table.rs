//! C ABI callback table for calls from a native plugin into its host.

use crate::value::{AbiBuffer, AbiCallResult, AbiHandle, AbiSlice, AbiStatus};

type HostCall = unsafe extern "C" fn(AbiHandle, AbiSlice, AbiBuffer) -> AbiCallResult;

/// Versioned host request and reply callback for one native instance.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HostFunctionTable {
    /// Host table version.
    pub abi_version: u32,
    /// Size of this table in bytes.
    pub byte_size: u32,
    /// Callback receiving the instance handle and request and reply storage.
    pub call: Option<HostCall>,
}

impl HostFunctionTable {
    /// Required reply capacity and maximum request size for the kit's framed service transport.
    pub const SERVICE_PACKET_BYTES: usize = 64 * 1024;

    /// Native symbol that accepts the host callback table during initialization.
    pub const BIND_ENTRYPOINT: &str = "obscur_plugin_bind_host_v1";

    /// Builds the version-1 host call table.
    #[must_use]
    pub const fn new(call: HostCall) -> Self {
        Self {
            abi_version: 1,
            byte_size: size_of::<Self>() as u32,
            call: Some(call),
        }
    }

    /// Checks that the version-1 callback is available.
    #[must_use]
    pub const fn supports_v1(&self) -> bool {
        self.abi_version == 1 && self.byte_size as usize >= size_of::<Self>() && self.call.is_some()
    }

    /// Calls the host with borrowed buffers and checks the reply length.
    ///
    /// # Safety
    ///
    /// The table must contain a trusted callback that is valid for this
    /// instance. The callback must respect the buffer sizes and must not
    /// retain either pointer or unwind across the call.
    pub unsafe fn exchange<'a>(
        &self,
        instance: AbiHandle,
        request: &[u8],
        reply: &'a mut [u8],
    ) -> Result<&'a [u8], AbiStatus> {
        if !self.supports_v1() {
            return Err(AbiStatus::REJECTED);
        }
        let call = self.call.ok_or(AbiStatus::REJECTED)?;
        // SAFETY: the caller guarantees a valid callback, and the borrowed buffers live through the call.
        let result = unsafe {
            call(
                instance,
                AbiSlice::from_bytes(request),
                AbiBuffer::from_bytes(reply),
            )
        };
        if result.status != AbiStatus::OK {
            return Err(result.status);
        }
        reply.get(..result.written).ok_or(AbiStatus::REJECTED)
    }
}
