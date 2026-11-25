Replaces the authored stylesheet set with `sheets`, in order (spec §4.12).

This is §4.12's explicit invalidation, from application code: a theme switch,
a re-read of a `.css` file on disk, a density change. The new set replaces the
old one wholesale — it is not appended — and the UA default sheet is
untouched, so authored rules keep winning over it.

Each source is given a stable [`SourceId`] by position and stamped with a new
[`SourceVersion`], which is what the parse cache is keyed by: the cache holds
one entry per id, so a replacement re-parses rather than growing, and a sheet
that keeps its position keeps its identity. Every sheet in the set is
re-parsed, because replacing the set is a statement about the whole set and
this API is handed strings rather than revisions — a host that wants
per-source revisions drives [`CssPipeline`] with its own [`CssSource`]s.

Applied on the next [`apply`](Self::apply); several calls in one turn collapse
to the last, since only one set can be current. Installing schedules a frame.
