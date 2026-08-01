Converts an obscur-scene [`DisplayList`] into a [`vello::Scene`].

This is the top-level pure conversion of the adapter (spec §2.7): it walks the
display list's commands **in paint order** and replays them onto a
`vello::Scene`, delegating geometry/paint/clip translation to the other
converters in this module. Paint order, clipping, and layer nesting are
preserved exactly; only rasterization is Vello's (spec §5.2 determinism
boundary).
