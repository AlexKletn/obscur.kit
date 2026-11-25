A lightweight, borrowed description of one itemized run for which a font must
be resolved (spec §5.10 font fallback).

A [`FontResolver`](crate::font::FontResolver) is asked to pick a face for a
single [`TextItem`](crate::shaping::TextItem): the request bundles the
[`families`](Self::families) fallback chain to search, the requested
[`attributes`](Self::attributes), the run's [`text`](Self::text), and its
resolved [`script`](Self::script). It borrows everything rather than owning
it — a request describes one resolution in flight and is never stored — so it
carries a lifetime tying it to the paragraph being shaped.
