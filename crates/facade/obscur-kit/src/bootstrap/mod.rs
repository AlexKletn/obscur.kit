//! Bootstrap module.
//!
//! # The §5.18 deviation this module makes explicit
//!
//! Spec §5.18 asks for a "bundled fallback set for WASM/isolated environments".
//! The kit **does not ship one**, deliberately, and nothing here silently covers
//! for its absence: a host that installs no faces renders no text, and
//! [`KitBootstrap::install_text_stack`] says so rather than degrading behind the
//! author's back. Fonts are large, licence-bearing binaries, and shipping a set in
//! every build to serve the one target that cannot read a font off disk is a cost
//! the owner declined. See `docs/conformance-audit.md` §5-B for the recorded
//! deviation.
mod css_pipeline;
pub use css_pipeline::CssPipeline;
mod kit_bootstrap;
pub use kit_bootstrap::KitBootstrap;
mod text_stack;
pub use text_stack::{FontFaceBytes, FontSet, InstalledFonts};
