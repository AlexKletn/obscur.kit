//! Bootstrap module.
//!
//! # The §5.18 bundled fallback this module ships
//!
//! Spec §5.18 asks for a "bundled fallback set for WASM/isolated environments".
//! The kit **ships one**: the vendored `assets/fonts/NotoSans-Regular.ttf`,
//! embedded at compile time (so it needs no filesystem — which the isolated / WASM
//! target lacks) and appended by
//! [`KitBootstrap::install_text_stack`] as a last-resort face after the host's own.
//! A host that installs an empty [`FontSet`] therefore still renders text, from the
//! bundled face; a host that supplies its own faces is unaffected, since those
//! match first and the bundled one only fills a gap nothing else covers. This is
//! §5.18 **met** — the earlier deliberate deviation (no bundled set) has been
//! reversed. See `docs/conformance-audit.md` §5-B.
mod css_pipeline;
pub use css_pipeline::CssPipeline;
mod kit_bootstrap;
pub use kit_bootstrap::KitBootstrap;
mod text_stack;
pub use text_stack::{FontFaceBytes, FontSet, InstalledFonts};
