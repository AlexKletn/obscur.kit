Resolves what `event` landing on `target` does to `current`.

The chain is `target` plus its element ancestors, topmost-first — **unless**
a touchscreen contact ended (an `Up` or a `Cancel` from a `Touch` device),
in which case nothing stays hovered. During contact hover still tracks the
target, matching a touchstart→touchend `:hover` on the web. A pen
genuinely hovers above the surface, so it is deliberately not cleared.

The walk follows logical parents and is bounded by the node count, so a
malformed parent chain terminates.
