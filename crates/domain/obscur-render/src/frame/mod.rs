//! Frame module.
mod device_loss;
pub use device_loss::DeviceLoss;
mod device_loss_reason;
pub use device_loss_reason::DeviceLossReason;
mod frame_submission;
pub use frame_submission::FrameSubmission;
mod present_mode;
pub use present_mode::PresentMode;
mod present_result;
pub use present_result::PresentResult;
mod render_frame;
pub use render_frame::RenderFrame;
