//! The one-call entry for a **layer surface** — a panel, dock, bar or overlay
//! that is part of the desktop rather than an application window (spec §6.5).
//!
//! ```ignore
//! use obscur_kit::layer::LayerApp;
//!
//! LayerApp::top_bar("panel", 32)
//!     .stylesheet(".bar { background-color: #1c2233; color: #eef }")
//!     .view(|| build_bar())
//!     .run()?;
//! ```
//!
//! Everything above the window is the same toolkit: the same `View`, the same
//! CSS, the same runtime, the same Vello renderer, the same touch and gesture
//! handling. What differs is only how the surface reaches the screen.
//!
//! # Where it runs
//!
//! `wlr-layer-shell` is a Wayland protocol implemented by wlroots-based
//! compositors (Sway, Hyprland, river, Wayfire, …) and by KDE. GNOME has
//! declined it. On a compositor without it, [`LayerApp::run`] returns
//! [`LayerAppError::Unsupported`] — a capability answer a host can branch on,
//! not a crash.
mod layer_app;
pub use layer_app::{LayerApp, LayerAppError};
mod runner;
