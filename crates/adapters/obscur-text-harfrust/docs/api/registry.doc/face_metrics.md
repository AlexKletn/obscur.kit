Returns a snapshot of every registered face's real design metrics, keyed by
the [`FontId`] handed out at registration (spec §5.10).

Hand it to
[`HarfrustShaper::with_faces`](crate::HarfrustShaper::with_faces): a shaper
is asked for line metrics by id alone, so without this table it cannot tell
which face an id names and answers every query with the idealized ratios.
A face registered *after* this call is not in the snapshot — take it once
the faces are registered.
