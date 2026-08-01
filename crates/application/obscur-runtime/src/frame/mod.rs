#![doc = include_str!("mod.doc/_module.md")]
mod frame_context;
pub use frame_context::FrameContext;
// `FrameId` is an identity, so it lives with the others in `obscur-identity`
// rather than here — which is also what lets `obscur-diagnostics` (foundation)
// attribute a span to a frame by TYPE instead of a bare `u64` (spec §7.9).
// Re-exported so `obscur_runtime::frame::FrameId` keeps naming it.
pub use obscur_identity::FrameId;
mod frame_pipeline;
pub use frame_pipeline::FramePipeline;
mod frame_reason;
pub use frame_reason::FrameReason;
mod frame_request;
pub use frame_request::FrameRequest;
mod frame_result;
pub use frame_result::FrameResult;
