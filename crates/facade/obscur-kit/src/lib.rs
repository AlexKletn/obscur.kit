//! Umbrella facade for Obscur Kit: a stable prelude, a high-level bootstrap, and
//! curated re-exports of the toolkit's crates (spec §2.9).
//!
//! Applications depend on this one crate instead of the individual foundation,
//! domain, application, and adapter crates. It contains no domain implementation
//! and is never used as a dependency of internal crates.
//!
//! # Re-exports
//!
//! Every toolkit crate is re-exported under a short module alias, so
//! `obscur_kit::scene::Color` reaches the same type as `obscur_scene::color::Color`.
//! Optional adapters are re-exported only when their Cargo feature is enabled,
//! and the platform system-service provider is selected automatically by target.
//!
//! # Features
//!
//! The `default` set is portable (`render-cpu`, `task-smol`, `resource-fs`,
//! `cache-fs`) and needs no display or GPU. The `full` set additionally enables
//! `render-vello`, `window-winit`, `task-tokio`, and `accesskit`.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod bootstrap;
pub mod prelude;

// --- foundation ---
pub use obscur_capability as capability;
pub use obscur_diagnostics as diagnostics;
pub use obscur_geometry as geometry;
pub use obscur_identity as identity;
pub use obscur_task as task;

// --- domain ---
pub use obscur_accessibility as accessibility;
pub use obscur_data_transfer as data_transfer;
pub use obscur_interaction as interaction;
pub use obscur_layout as layout;
pub use obscur_motion as motion;
pub use obscur_render as render;
pub use obscur_resources as resources;
pub use obscur_routing as routing;
pub use obscur_scene as scene;
pub use obscur_style as style;
pub use obscur_system as system;
pub use obscur_text as text;
pub use obscur_tree as tree;
pub use obscur_view as view;
pub use obscur_windowing as windowing;

// --- application ---
pub use obscur_headless as headless;
pub use obscur_runtime as runtime;

// --- optional adapters (feature-gated) ---
#[cfg(feature = "accesskit")]
pub use obscur_accessibility_accesskit as accesskit_bridge;
#[cfg(feature = "cache-fs")]
pub use obscur_cache_fs as cache_fs;
#[cfg(feature = "render-cpu")]
pub use obscur_render_cpu as render_cpu;
#[cfg(feature = "render-vello")]
pub use obscur_render_vello as render_vello;
#[cfg(feature = "resource-fs")]
pub use obscur_resource_fs as resource_fs;
#[cfg(feature = "task-smol")]
pub use obscur_task_smol as task_smol;
#[cfg(feature = "task-tokio")]
pub use obscur_task_tokio as task_tokio;
#[cfg(feature = "window-winit")]
pub use obscur_window_winit as window_winit;

// --- platform system provider (selected by target) ---
#[cfg(target_os = "linux")]
pub use obscur_system_linux as system_platform;
#[cfg(target_arch = "wasm32")]
pub use obscur_system_web as system_platform;
