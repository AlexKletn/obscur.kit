//! The outcome of a capture request (spec §6.17 — capture and privacy).

use obscur_geometry::Size;

/// A descriptor of successfully captured pixels (spec §6.17).
///
/// `CapturedImage` names the [`size`](Self::size) and pixel [`format`](Self::format)
/// of a capture; the actual bytes live in an interface adapter, never in the pure
/// domain. It is the payload of [`CaptureResult::Captured`].
#[derive(Debug, Clone, PartialEq)]
pub struct CapturedImage {
    /// The pixel dimensions of the captured image.
    size: Size,
    /// The pixel format token (for example `"rgba8"`).
    format: Box<str>,
}

impl CapturedImage {
    /// Creates a descriptor for pixels of `size` in `format` (spec §6.17).
    #[must_use]
    pub fn new(size: Size, format: impl Into<Box<str>>) -> Self {
        Self {
            size,
            format: format.into(),
        }
    }

    /// Returns the pixel dimensions of the captured image (spec §6.17).
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// Returns the pixel format token (spec §6.17).
    #[must_use]
    pub fn format(&self) -> &str {
        &self.format
    }
}

/// The outcome a [`CaptureProvider`](crate::capture::CaptureProvider) returns for a
/// [`CaptureRequest`](crate::capture::CaptureRequest) (spec §6.17).
///
/// The four outcomes mirror the spec's vocabulary: the target was captured, was
/// denied by permission policy, is unsupported on this host, or failed. Only
/// [`Captured`](Self::Captured) carries a [`CapturedImage`]; [`Failed`](Self::Failed)
/// carries a human-readable reason.
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

    #[test]
    fn captured_image_stores_size_and_format() {
        let image = CapturedImage::new(Size::new(2.0, 3.0), "rgba8");
        assert_eq!(image.size(), Size::new(2.0, 3.0));
        assert_eq!(image.format(), "rgba8");
    }

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
        assert_eq!(
            CaptureResult::Captured(image.clone()).image(),
            Some(&image)
        );
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
