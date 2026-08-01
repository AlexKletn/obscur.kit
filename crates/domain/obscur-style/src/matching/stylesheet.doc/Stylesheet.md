A collection of [`StyleRule`]s that all share a single [`CascadeOrigin`]
(§4.14).

A stylesheet represents exactly one origin band: the system-default sheet,
the theme sheet, the global-app sheet, a component/module sheet, or the
inline sheet. A complete application therefore has several stylesheets, one
per origin; the [`StyleMatcher`](crate::matching::StyleMatcher) accepts an
ordered slice of them (low origin → high origin) and stamps each rule's
declarations with this origin during matching.
