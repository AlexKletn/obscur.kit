Computes the node's [`ComputedStyle`] from its cascaded declarations, its
cascaded custom properties, and its parent's computed style (§4.14).

`parent` is [`None`] for the root; every inherited property then falls
back to its initial value. The returned style stores the node's effective
(inheritable) custom properties for its descendants.

`ctx` supplies the document-wide `rem` / viewport bases; the node's own
`font-size` supplies `em`. Every `<length>` on the returned style is
absolute, except a `%` (whose containing block layout has not produced), an
`auto`, and a `ch` (see the module docs).
