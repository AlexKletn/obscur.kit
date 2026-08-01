The fully-resolved computed style of a node: [`ComputedStyle`] (§4.14).

A [`ComputedStyle`] is what [`crate::computed::StyleResolver`] produces for a
node once the cascade has run: every property this crate computes maps to a
concrete [`ComputedValue`], and the node's **effective custom properties**
(the ones a child inherits) are stored alongside so resolution can flow down
the tree.

Custom properties are kept as raw [`PropertyValue`]s (already `var()`-
substituted and inherited, per §4.14) rather than [`ComputedValue`]s: they are
substitution inputs for descendants, not per-property computed values in their
own right.
