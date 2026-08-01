Whether the surface maps content to host coordinates unchanged — scale
exactly `1.0` and no content offset. The one condition both
[`root_transform`](Self::root_transform) and
[`to_content_point`](Self::to_content_point) branch on, written once so the
two cannot disagree about when the mapping is trivial.
