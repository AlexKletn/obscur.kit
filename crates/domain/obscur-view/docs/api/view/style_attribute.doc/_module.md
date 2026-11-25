Well-known style attributes carried on element view nodes for CSS matching.

CSS selectors match on an element's *type* (local name), its `id`, and its
`class` list (spec §4.13). Those live on a [`ViewNode`](super::ViewNode)
element as ordinary [`ViewProperty`] bindings under a small set of reserved
interned symbols, defined here so the whole toolkit agrees on them. The
reconciler reads them via [`StyleAttributes::extract`] and stores the result
on the retained node, where the selector matcher can see it.

The reserved names occupy a low symbol namespace (`0x0000_00xx`), distinct
from the component/primitive prop namespaces (e.g. `0x00C1_xxxx`), so they
never collide with an ordinary prop.
