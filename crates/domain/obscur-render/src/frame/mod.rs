#![doc = include_str!("mod.doc/_module.md")]
mod device_loss;
pub use device_loss::{DeviceLoss, DeviceLossReason};
mod frame_submission;
pub use frame_submission::{FrameSubmission, PresentMode};
mod present_result;
pub use present_result::PresentResult;
mod render_frame;
pub use render_frame::RenderFrame;
