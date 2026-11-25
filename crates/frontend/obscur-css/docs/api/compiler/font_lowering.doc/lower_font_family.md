Lowers a `font-family` value list into an ordered
[`PropertyValue::FontFamilies`], preserving author order.

The component list is split on commas into one group per family. Within a
group, a single quoted string is taken verbatim (`"Helvetica Neue"`), while
one-or-more bare idents are joined with a single space (`Helvetica Neue`);
a lone generic keyword (`serif`, `sans-serif`, …) is kept as its own name so
the runtime can map it to a generic family. Empty groups (a stray or trailing
comma) are skipped. Returns [`None`] when no family name can be recovered, so
the declaration degrades to an unsupported-feature diagnostic rather than
being dropped silently.
