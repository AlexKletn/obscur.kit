#![doc = include_str!("mod.doc/_module.md")]
mod app;
pub use app::{App, FrameInfo};
mod app_error;
pub use app_error::AppError;
mod app_parts;
#[cfg(feature = "shell-layer")]
mod layer_control;
#[cfg(feature = "shell-layer")]
pub use layer_control::LayerControl;
#[cfg(feature = "shell-layer")]
mod layer_loop_stats;
#[cfg(feature = "shell-layer")]
mod layer_runner;
#[cfg(feature = "window-winit")]
mod runner;
#[cfg(feature = "shell-layer")]
mod turn_clock;
