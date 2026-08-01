§4.16: "Other system-generated parts may use synthetic retained nodes hidden
from the component tree but visible in DevTools:" / "- scrollbar parts;"

A `overflow: scroll` container with exactly 2 author children and content
tall enough to show a scrollbar. The component tree must report exactly 2
children; the DevTools tree must additionally show scrollbar part nodes
(track and thumb) with real rects on the container's right edge.
