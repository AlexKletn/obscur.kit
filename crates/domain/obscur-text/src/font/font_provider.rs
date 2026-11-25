#![doc = include_str!("../../docs/api/font/font_provider.doc/_module.md")]

use crate::font::{FontAttributes, FontFace, FontFallback, FontFamily};

#[doc = include_str!("../../docs/api/font/font_provider.doc/FontProvider.md")]
pub trait FontProvider {
    #[doc = include_str!("../../docs/api/font/font_provider.doc/resolve.md")]
    #[must_use]
    fn resolve(&self, family: &FontFamily, attributes: &FontAttributes) -> Option<FontFace>;

    /// Returns the fallback chain to consult when `family` cannot be resolved
    /// (spec §5.18 fallback resolution, §5.10 font fallback).
    #[must_use]
    fn fallback(&self, family: &FontFamily) -> FontFallback;

    /// Returns whether this provider can supply any face for `family`
    /// (spec §5.18).
    #[must_use]
    fn is_available(&self, family: &FontFamily) -> bool;
}
