#![doc = include_str!("../../docs/api/app/mod.doc/_module.md")]
mod app;
pub use app::App;
mod frame_info;
pub use frame_info::FrameInfo;
mod app_error;
pub use app_error::AppError;
mod app_parts;
mod appearance_subscription;
mod booted_app;
#[cfg(feature = "window-winit")]
mod frame_stats;
mod gpu_instance;
mod render_backend;
pub use render_backend::RenderBackend;
#[cfg(feature = "shell-layer")]
mod layer_control;
#[cfg(feature = "shell-layer")]
mod layer_control_state;
#[cfg(feature = "shell-layer")]
mod popup_content;
#[cfg(feature = "shell-layer")]
mod popup_request;
#[cfg(feature = "shell-layer")]
pub use layer_control::LayerControl;
#[cfg(feature = "shell-layer")]
mod layer_loop_stats;
#[cfg(feature = "shell-layer")]
mod layer_popup_ui;
#[cfg(feature = "shell-layer")]
mod layer_runner;
#[cfg(all(feature = "window-winit", target_os = "linux"))]
mod presentation_stats;
#[cfg(feature = "window-winit")]
mod runner;
#[cfg(feature = "shell-layer")]
mod turn_clock;
#[cfg(feature = "window-winit")]
mod window_size_limits;
#[cfg(feature = "window-winit")]
mod window_wake;

#[cfg(any(
    feature = "shell-layer",
    all(feature = "window-winit", target_os = "linux")
))]
mod wayland_media_input;
