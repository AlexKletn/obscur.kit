The [`OverscanPolicy`] — how many extra items to materialize beyond the
visible viewport on each side (spec §4.9 *overscan*).

Overscan reduces blank flashes while scrolling: by materializing a handful of
items just past each edge of the viewport, freshly revealed items are already
laid out when they scroll into view. The policy is pure sizing data — it maps
a visible index range to an expanded, saturating one.
