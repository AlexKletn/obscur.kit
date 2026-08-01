Whether this box is a **scroll container** — its computed `overflow` is
`scroll` / `auto` / `hidden` (anything but `visible` / `clip` / absent).

css-flexbox-1 §4.5 makes this decide a flex item's automatic minimum size:
a non-scrollable item's `min-*: auto` resolves to its content-based
minimum, but a scroll container's resolves to **zero** ("for scroll
containers the automatic minimum size is zero", css-flexbox-1.txt:1084-1085).
Default `false` (the initial `overflow: visible`), so a box that never
scrolls keeps its content-based automatic minimum unchanged.
