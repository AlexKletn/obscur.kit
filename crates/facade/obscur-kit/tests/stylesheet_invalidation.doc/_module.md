§4.12's explicit stylesheet invalidation, reached from an application.

§4.12 requires that "CSS parsing is cached per unique source/version and
invalidated **explicitly**", and the mechanism was all there: a `CssSource`
carries a `SourceVersion`, `StylesheetCache` is keyed by `(SourceId,
SourceVersion)`, and `UiRuntime::set_stylesheets` replaces the set at any time.
What was missing was a *caller*: `App` collected the authored sheets once before
the loop started, handed them over as **anonymous** sources (which the cache
refuses to key, by design), and never exposed the runtime again. So an
application could not invalidate anything, and every style edit cost a rebuild.

These cases drive the same pair of calls `App`'s window runner makes — apply at
the frame boundary, run the frame — so what they prove holds for a standalone
window without needing one (the windowed `App` is gated on `window-winit` +
`render-vello`, and a test behind those features would be silently skipped by the
default feature set that CI runs).
