//! A GPU device-loss event and its recoverability (spec §5.2).

use super::DeviceLossReason;

#[doc = include_str!("../../docs/api/frame/device_loss.doc/DeviceLoss.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceLoss {
    /// Why the device was lost.
    pub reason: DeviceLossReason,
    /// Whether the runtime may rebuild the device and continue.
    pub recoverable: bool,
    /// An optional human-readable detail describing the event.
    pub detail: Option<Box<str>>,
}

impl DeviceLoss {
    /// Creates a device-loss event with the given `reason` and `recoverable`
    /// flag, and no detail.
    #[must_use]
    pub const fn new(reason: DeviceLossReason, recoverable: bool) -> Self {
        Self {
            reason,
            recoverable,
            detail: None,
        }
    }

    /// Creates a recoverable device-loss event with the given `reason`.
    #[must_use]
    pub const fn recoverable(reason: DeviceLossReason) -> Self {
        Self::new(reason, true)
    }

    /// Creates a fatal (non-recoverable) device-loss event with the given
    /// `reason`.
    #[must_use]
    pub const fn fatal(reason: DeviceLossReason) -> Self {
        Self::new(reason, false)
    }

    /// Attaches a human-readable detail, returning the updated event.
    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<Box<str>>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Why the device was lost.
    #[must_use]
    pub const fn reason(&self) -> DeviceLossReason {
        self.reason
    }

    /// The human-readable detail, or `None` if none was attached.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// Whether the runtime may rebuild the device and continue.
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        self.recoverable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recoverable_constructor_sets_flag() {
        let loss = DeviceLoss::recoverable(DeviceLossReason::Reset);
        assert_eq!(loss.reason(), DeviceLossReason::Reset);
        assert!(loss.is_recoverable());
        assert_eq!(loss.detail(), None);
    }

    #[test]
    fn fatal_constructor_clears_flag() {
        let loss = DeviceLoss::fatal(DeviceLossReason::Removed);
        assert_eq!(loss.reason(), DeviceLossReason::Removed);
        assert!(!loss.is_recoverable());
    }

    #[test]
    fn new_sets_reason_and_flag() {
        let loss = DeviceLoss::new(DeviceLossReason::OutOfMemory, false);
        assert_eq!(loss.reason(), DeviceLossReason::OutOfMemory);
        assert!(!loss.is_recoverable());
    }

    #[test]
    fn with_detail_attaches_message() {
        let loss =
            DeviceLoss::recoverable(DeviceLossReason::Timeout).with_detail("gpu hung for 2s");
        assert_eq!(loss.detail(), Some("gpu hung for 2s"));
        assert!(loss.is_recoverable());
    }
}
