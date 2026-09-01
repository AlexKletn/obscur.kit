#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

// The one-call standalone entry point (spec §1.4). It owns a surface and the event
// loop, so it needs a renderer and at least one surface backend to talk to — either
// an ordinary window (`window-winit`) or a desktop layer surface (`shell-layer`),
// with `App::layer` choosing between them. Entirely absent from the headless
// `default` build, which has neither.
#[cfg(all(feature = "render-vello", feature = "surface"))]
pub mod app;
pub mod bootstrap;
// Live control of a running runtime (§2.6 positional commands, §2.3 listeners,
// §4.12 stylesheet invalidation) and the run-time `.okui` front end. Both are
// display-free on purpose: `App` composes them for a standalone window, and a guest
// host that embeds a `UiRuntime` itself uses the same types.
pub mod control;
pub mod default_stylesheet;
pub mod prelude;
pub mod screen;
pub mod system_theme;

#[cfg(all(feature = "render-vello", feature = "shell-layer"))]
pub use app::LayerControl;
#[cfg(all(feature = "render-vello", feature = "surface"))]
pub use app::{App, AppError, FrameInfo};
pub use control::AppControl;
pub use default_stylesheet::{ua_css, ua_stylesheet};
pub use screen::{OkuiScreen, ScreenData, ScreenError};
pub use system_theme::ThemeVars;

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
pub use obscur_localization as localization;
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

// --- components (built-in element/widget tier) ---
pub use obscur_primitives as primitives;
pub use obscur_theme_system as theme_system;
pub use obscur_widgets as widgets;

// --- frontends (UI authoring: CSS, UI IR, DSL macros, .okui) ---
pub use obscur_css as css;
pub use obscur_okui_compiler as okui_compiler;
pub use obscur_okui_syntax as okui_syntax;
pub use obscur_ui_ir as ui_ir;
pub use obscur_ui_macros as ui_macros;

// --- diagnostics: external DevTools (spec §7.6/§7.7) ---
pub use obscur_devtools_protocol as devtools_protocol;
pub use obscur_devtools_server as devtools_server;
pub mod devtools;
pub mod perf;

// --- optional adapters (feature-gated) ---
#[cfg(feature = "accesskit")]
pub use obscur_accessibility_accesskit as accesskit_bridge;
#[cfg(feature = "cache-fs")]
pub use obscur_cache_fs as cache_fs;
#[cfg(feature = "font-fontconfig")]
pub use obscur_font_fontconfig as font_fontconfig;
#[cfg(feature = "render-cpu")]
pub use obscur_render_cpu as render_cpu;
#[cfg(feature = "render-vello")]
pub use obscur_render_vello as render_vello;
#[cfg(feature = "resource-decode")]
pub use obscur_resource_decode as resource_decode;
#[cfg(feature = "resource-fs")]
pub use obscur_resource_fs as resource_fs;
#[cfg(feature = "shell-layer")]
pub use obscur_shell_layer as shell_layer;
#[cfg(feature = "task-smol")]
pub use obscur_task_smol as task_smol;
#[cfg(feature = "task-tokio")]
pub use obscur_task_tokio as task_tokio;
#[cfg(feature = "text-harfrust")]
pub use obscur_text_harfrust as text_harfrust;
#[cfg(feature = "text-unicode")]
pub use obscur_text_unicode as text_unicode;
#[cfg(feature = "window-winit")]
pub use obscur_window_winit as window_winit;

// --- platform system provider (selected by target) ---
#[cfg(target_os = "linux")]
pub use obscur_system_linux as system_platform;
#[cfg(target_arch = "wasm32")]
pub use obscur_system_web as system_platform;
