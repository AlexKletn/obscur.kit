#[doc = include_str!("../../docs/api/frame/device_loss_reason.doc/DeviceLossReason.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceLossReason {
    /// The device was reset, for example by a driver-level TDR recovery.
    Reset,
    /// The device ran out of memory.
    OutOfMemory,
    /// The physical device was removed (an external GPU was unplugged, say).
    Removed,
    /// A device operation timed out.
    Timeout,
    /// The cause could not be determined.
    Unknown,
}
