Returns `true` only for [`Relative`](Self::Relative) — the one scheme that
offsets an **in-flow** node by its insets.

[`Absolute`](Self::Absolute) and [`Fixed`](Self::Fixed) return `false`:
they are out of flow and are positioned by the coordinator's out-of-flow
path, not by the in-flow relative offset.
