Whether a change to a property can change a box: the paint-only allow-list
(spec §3.4 phase 5, §4.3).

This is what the frame's invalidation classifier consults to decide whether a
restyle may reuse the retained layout, and it is one fact per [`PropertyId`]
that the catalogue does not state anywhere: a variant's doc says what the
property *is*, never what a change to it costs. The allow-list is deliberately
conservative in the direction that cannot be wrong — see
[`PropertyId::is_paint_only`].

That this was already a separate concern is visible in the file it came out
of: it arrived there with a test module of its own, beside — not inside — the
catalogue's name/inheritance tests.
