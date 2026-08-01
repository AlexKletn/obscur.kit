Writes `node`'s offset directly, **without** refolding the projection.

For the two writers whose clamp comes from a measure the live
[`ScrollContainer`] does not carry, and only those:

- a `<VirtualFor>` container reset to the top because the app replaced the
  list's model (spec §4.9) — the offset belonged to the old content, and
  this runs *inside* the frame that then computes the window against it;
- a fixed-height `<textarea>`'s caret-follow, which clamps against the
  **wrapped text's** height rather than the retained boxes' (CSS Overflow 3
  §3.1) — the box it would clamp against has not been laid out yet.

Both run inside an operation that ends in a frame, and a frame refolds, so
the projection is not left stale. A caller that does **not** schedule a
frame must use [`set`](Self::set) instead.
