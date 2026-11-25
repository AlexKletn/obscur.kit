Records each resident image's on-screen priority for this frame, so
eviction drops what nothing is showing first (spec §5.19 *visibility
priority*).

A URL is [`Visible`](VisibilityPriority::Visible) when some node that
references it has an arranged box intersecting `viewport`,
[`Offscreen`](VisibilityPriority::Offscreen) when its nodes are all
scrolled out or unarranged, and — by the cache's own default for a URL that
appears here at all — [`Hidden`](VisibilityPriority::Hidden) when the tree
no longer references it. The strongest reference wins: one on-screen use
keeps an image resident.

[`NearViewport`](VisibilityPriority::NearViewport) is never produced: it
would need a prefetch margin, and the runtime has no notion of one.

`viewport` is the visible rect in **document** coordinates, which is what
the absolute layout `boxes` are expressed in.

The referenced set it collects is also what bounds the bridge's per-URL
bookkeeping, so this pass ends by handing it to
[`retain_referenced`](Self::retain_referenced) — the same set, one walk.
