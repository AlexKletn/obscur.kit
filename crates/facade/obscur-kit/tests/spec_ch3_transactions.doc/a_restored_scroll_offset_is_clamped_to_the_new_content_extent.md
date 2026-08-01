§3.8 — "scroll positions" restored via "Optional versioned helpers may
serialize and restore".

A persisted offset of 1500 restored into a session whose content is now only
600px tall (400px viewport → max scroll 200) must clamp to **200** — not
1500, and not 0.

Translation note: the kit exposes no persistence-snapshot round trip, so the
restore is expressed as writing the persisted offset into the scroll state,
which is what a restore helper would do.
