#![doc = include_str!("device_loss.doc/_module.md")]

#[doc = include_str!("device_loss.doc/DeviceLossReason.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceLossReason {
    #[doc = include_str!("device_loss.doc/Reset.md")]
    Reset,
    #[doc = include_str!("device_loss.doc/OutOfMemory.md")]
    OutOfMemory,
    #[doc = include_str!("device_loss.doc/Removed.md")]
    Removed,
    #[doc = include_str!("device_loss.doc/Timeout.md")]
    Timeout,
    #[doc = include_str!("device_loss.doc/Unknown.md")]
    Unknown,
}

#[doc = include_str!("device_loss.doc/DeviceLoss.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceLoss {
    #[doc = include_str!("device_loss.doc/reason.md")]
    pub reason: DeviceLossReason,
    #[doc = include_str!("device_loss.doc/recoverable.md")]
    pub recoverable: bool,
    #[doc = include_str!("device_loss.doc/detail.md")]
    pub detail: Option<Box<str>>,
}

impl DeviceLoss {
    #[doc = include_str!("device_loss.doc/new.md")]
    #[must_use]
    pub const fn new(reason: DeviceLossReason, recoverable: bool) -> Self {
        Self {
            reason,
            recoverable,
            detail: None,
        }
    }

    #[doc = include_str!("device_loss.doc/recoverable_2.md")]
    #[must_use]
    pub const fn recoverable(reason: DeviceLossReason) -> Self {
        Self::new(reason, true)
    }

    #[doc = include_str!("device_loss.doc/fatal.md")]
    #[must_use]
    pub const fn fatal(reason: DeviceLossReason) -> Self {
        Self::new(reason, false)
    }

    #[doc = include_str!("device_loss.doc/with_detail.md")]
    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<Box<str>>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    #[doc = include_str!("device_loss.doc/reason_2.md")]
    #[must_use]
    pub const fn reason(&self) -> DeviceLossReason {
        self.reason
    }

    #[doc = include_str!("device_loss.doc/detail_2.md")]
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    #[doc = include_str!("device_loss.doc/is_recoverable.md")]
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
