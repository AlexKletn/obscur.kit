The document-wide bases a computed `<length>` is absolutized against (§4.14).

These are the things a node's own style cannot supply: the **root** element's
computed `font-size` (the base for `rem`), the **viewport** dimensions (the
bases for `vw` / `vh` / `vmin` / `vmax`), and the **font metrics** port that
measures the `0` glyph (the base for `ch`). The node's own `font-size` — the
base for `em` — is not here because the resolver computes it itself, before
every other property.

`rem` has no chicken-and-egg problem: the root's font-size depends only on the
root's own cascaded value and the initial `medium`, so it is known before any
other node is resolved. Build the root's context with
[`ResolutionContext::for_root`], read the root's computed `font-size` back with
[`StyleResolver::font_size_of`](super::StyleResolver::font_size_of), and pass it to
[`ResolutionContext::new`] for every descendant.

`ch` is the one base that is a *measurement* rather than a number, so it
arrives as a port ([`ChMetrics`]) rather than as an `f32`: its value depends
on the node's own resolved font, which only the resolver knows, so the resolver
has to be able to *ask* mid-resolution rather than be told up front. Attach one
with [`with_metrics`](Self::with_metrics); a context without one resolves every
other unit exactly as before and leaves `ch` to the specified fallbacks.

This does not implement [`PartialEq`]: a port has no meaningful value equality,
and nothing compares contexts. [`Debug`] is hand-written for the same reason —
so that implementing [`ChMetrics`] does not oblige an implementor to derive
[`Debug`], which no other port in this workspace requires.
