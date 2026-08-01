The outgoing text ports a host installs, and the language it shapes in
(spec §1.1, §7.2).

Four of the five are ports in the hexagonal sense — a shaper, an itemizer, a
font resolver, a hyphenator — and they are one file because they share one
invariant that none of them states on its own: **the retained measure and
arrange results were produced by the port being replaced.** A different shaper
wraps to different sizes, and a different size is a different position, so the
arranged boxes go with the measurements. Nothing else in the frame can see it —
a computed-style diff compares styles, and no style changed — so the swap has to
say so itself, which is what `invalidate_shaped_caches` is for.

`set_locale` is the fifth because it feeds the same orchestrator: the language
subtag becomes the BCP-47 hint on every shaping request, in both the measure and
the compose pass. It is *not* a port swap, and it deliberately does not clear the
caches — the language is a member of `LayoutInputs`, so the next frame's
classifier sees the mismatch and runs a full layout, which clears them anyway.
Its own documentation carries that argument, because the asymmetry between it
and its four neighbours is exactly the thing a reader of this file will ask
about.
