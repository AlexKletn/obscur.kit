§4.16: "Other system-generated parts may use synthetic retained nodes hidden
from the component tree but visible in DevTools:" / "- placeholders;" /
"- exported style parts;"

A text input with placeholder "Search" and an empty value, whose placeholder
is restyled through the exported-part selector syntax. "Search" paints in
`rgb(128,128,128)`, the input reports 0 component-tree children, the
DevTools tree shows a named placeholder part, and a hit on the glyphs
returns the input itself.
