Converts an obscur-scene [`DisplayList`] into a main [`vello::Scene`] plus a
list of [`BlurLayer`]s to blur and composite separately (spec §5.2).

A `filter: blur(r)` element's subtree is bracketed by a `PushLayer(Blur)` …
`PopLayer` (by the scene coordinator) and is therefore **contiguous** in
paint order. This walk detects that bracket, collects the bracketed commands
into their own sub-scene (recorded as a [`BlurLayer`] with the layer's clip
bounds and radius), and **omits** them from the main scene so they are not
also drawn un-blurred. Everything else is emitted exactly as [`Self::convert`]
would, so a list with no blur layer returns `(convert(list), vec![])`,
byte-for-byte identical to the previous single render.

# Nesting

Only the top level extracts blurs. A blur layer nested inside another is
drawn as an ordinary layer in the outer sub-scene (its content is blurred
once, by the outer pass, rather than separately) — a documented minimal
nesting limitation, not a panic.
