mod capture_provider;
pub use capture_provider::{CaptureProvider, NullCaptureProvider, StubCaptureProvider};
mod capture_request;
pub use capture_request::{CaptureRequest, CaptureSource};
mod capture_result;
pub use capture_result::{CaptureResult, CapturedImage};
