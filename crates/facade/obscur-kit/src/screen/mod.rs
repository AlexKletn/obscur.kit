//! The **run-time** `.okui` front end: a source compiled once, lowered per frame
//! (spec §2.9).
//!
//! §2.9 makes `.okui` a peer of the Rust DSL — "compiled into the same UI IR" —
//! and there are two ways to get there. This module is the second one:
//!
//! | | build time (`build.rs` + [`ViewCodegen`](crate::okui_compiler::codegen::ViewCodegen)) | run time ([`OkuiScreen`]) |
//! |---|---|---|
//! | when the source is parsed | while the crate compiles | in `App::run` |
//! | contract errors (§2.13) | compile errors | a [`ScreenError`] at startup |
//! | what a `{binding}` reads | the application's own Rust bindings | [`ScreenData`] |
//! | shape can depend on state | yes (it is Rust) | no — structure is fixed |
//!
//! The build-time route is the one §2.9's "compile-time contract validation"
//! describes, and the better answer whenever the markup is part of the crate. This
//! route exists for a screen that is only text at build time — one loaded from disk
//! or handed over by a host — and it used to be strictly static: the compiled
//! `View` was cloned for every frame, so nothing an application did could change
//! what it showed. Now the *IR* is kept and lowered each frame, and the lowering
//! resolves bindings against a table the application writes.
//!
//! Nothing here needs a display, a window or a GPU: it is the front end plus the
//! bridge, so a headless host and `App` share one implementation.
mod okui_screen;
pub use okui_screen::OkuiScreen;
mod screen_data;
pub use screen_data::ScreenData;
mod screen_error;
pub use screen_error::ScreenError;
