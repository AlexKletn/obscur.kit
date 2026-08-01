#![doc = include_str!("capture_result.doc/_module.md")]

use obscur_geometry::Size;

#[doc = include_str!("capture_result.doc/CapturedImage.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CapturedImage {
    #[doc = include_str!("capture_result.doc/size.md")]
    size: Size,
    #[doc = include_str!("capture_result.doc/format.md")]
    format: Box<str>,
}

impl CapturedImage {
    #[doc = include_str!("capture_result.doc/new.md")]
    #[must_use]
    pub fn new(size: Size, format: impl Into<Box<str>>) -> Self {
        Self {
            size,
            format: format.into(),
        }
    }

    #[doc = include_str!("capture_result.doc/size_2.md")]
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    #[doc = include_str!("capture_result.doc/format_2.md")]
    #[must_use]
    pub fn format(&self) -> &str {
        &self.format
    }
}

#[doc = include_str!("capture_result.doc/CaptureResult.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureResult {
    #[doc = include_str!("capture_result.doc/Captured.md")]
    Captured(CapturedImage),
    #[doc = include_str!("capture_result.doc/Denied.md")]
    Denied,
    #[doc = include_str!("capture_result.doc/Unsupported.md")]
    Unsupported,
    #[doc = include_str!("capture_result.doc/Failed.md")]
    Failed(Box<str>),
}

impl CaptureResult {
    #[doc = include_str!("capture_result.doc/is_captured.md")]
    #[must_use]
    pub const fn is_captured(&self) -> bool {
        matches!(self, Self::Captured(_))
    }

    #[doc = include_str!("capture_result.doc/image.md")]
    #[must_use]
    pub const fn image(&self) -> Option<&CapturedImage> {
        match self {
            Self::Captured(image) => Some(image),
            _ => None,
        }
    }

    #[doc = include_str!("capture_result.doc/failure_reason.md")]
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
