//! Capture module.
mod capture_provider;
pub use capture_provider::CaptureProvider;
mod null_capture_provider;
pub use null_capture_provider::NullCaptureProvider;
mod stub_capture_provider;
pub use stub_capture_provider::StubCaptureProvider;
mod capture_request;
pub use capture_request::CaptureRequest;
mod capture_source;
pub use capture_source::CaptureSource;
mod capture_result;
pub use capture_result::CaptureResult;
mod captured_image;
pub use captured_image::CapturedImage;
