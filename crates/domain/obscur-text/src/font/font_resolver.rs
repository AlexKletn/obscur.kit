//! The port that picks a covering font for one itemized run (spec §5.10).

use crate::font::{FontResolveRequest, ResolvedFont};

#[doc = include_str!("../../docs/api/font/font_resolver.doc/FontResolver.md")]
pub trait FontResolver {
    /// Resolves `request` to the first covering font in its chain, or `None`
    /// when nothing in the chain can render the run (spec §5.10).
    #[must_use]
    fn resolve(&self, request: &FontResolveRequest) -> Option<ResolvedFont>;
}
