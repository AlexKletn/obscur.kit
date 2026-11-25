//! The outcome of a capture request (spec §6.17 — capture and privacy).

use super::CapturedImage;

#[doc = include_str!("../../docs/api/capture/capture_result.doc/CaptureResult.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureResult {
    /// The capture succeeded, yielding an image descriptor.
    Captured(CapturedImage),
    /// The capture was denied by permission policy.
    Denied,
    /// The host does not support capturing this source.
    Unsupported,
    /// The capture was attempted but failed, carrying a human-readable reason.
    Failed(Box<str>),
}

impl CaptureResult {
    /// Returns whether the capture succeeded (spec §6.17).
    #[must_use]
    pub const fn is_captured(&self) -> bool {
        matches!(self, Self::Captured(_))
    }

    /// Returns the captured image when the outcome is [`Captured`](Self::Captured),
    /// else `None` (spec §6.17).
    #[must_use]
    pub const fn image(&self) -> Option<&CapturedImage> {
        match self {
            Self::Captured(image) => Some(image),
            _ => None,
        }
    }

    /// Returns the failure reason when the outcome is [`Failed`](Self::Failed),
    /// else `None` (spec §6.17).
    #[must_use]
    pub fn failure_reason(&self) -> Option<&str> {
        match self {
            Self::Failed(reason) => Some(reason),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Size;

    #[test]
    fn is_captured_only_for_captured() {
        let captured = CaptureResult::Captured(CapturedImage::new(Size::new(1.0, 1.0), "rgba8"));
        assert!(captured.is_captured());
        assert!(!CaptureResult::Denied.is_captured());
        assert!(!CaptureResult::Unsupported.is_captured());
        assert!(!CaptureResult::Failed(Box::from("x")).is_captured());
    }

    #[test]
    fn image_only_for_captured() {
        let image = CapturedImage::new(Size::new(1.0, 1.0), "rgba8");
        assert_eq!(CaptureResult::Captured(image.clone()).image(), Some(&image));
        assert_eq!(CaptureResult::Denied.image(), None);
        assert_eq!(CaptureResult::Failed(Box::from("x")).image(), None);
    }

    #[test]
    fn failure_reason_only_for_failed() {
        assert_eq!(
            CaptureResult::Failed(Box::from("gpu lost")).failure_reason(),
            Some("gpu lost")
        );
        assert_eq!(CaptureResult::Denied.failure_reason(), None);
        assert_eq!(CaptureResult::Unsupported.failure_reason(), None);
        assert_eq!(
            CaptureResult::Captured(CapturedImage::new(Size::new(1.0, 1.0), "rgba8"))
                .failure_reason(),
            None
        );
    }
}
