A requested font family: either a specific named family or a CSS generic
family keyword (spec §5.18).

This models what an application *asks for*, not a concrete resolved face. A
[`FontProvider`](crate::font::FontProvider) turns a `FontFamily` (together
with [`FontAttributes`](crate::font::FontAttributes)) into a
[`FontFace`](crate::font::FontFace); a generic family names an abstract
category the provider maps onto whatever concrete family it has for that
role.
