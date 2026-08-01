Paints one §4.16 generated run (`::before` / `::after` content or the
input placeholder) into `node_paint`, the owning node's own paint unit.

The run is shaped with the OWNING node's font (so a `::before` decoration
picks up the element's `font-size` / family) and drawn from the node's box
origin. Its ink is the run's resolved `color` when it has one (a
`::part(placeholder)` or `::before { color }` rule), else the node's own
`currentColor`. It emits only `DrawText` leaves, so `node_paint` stays
balanced, and — living in the element's own unit — it sorts before the
element's real text child in paint order.
