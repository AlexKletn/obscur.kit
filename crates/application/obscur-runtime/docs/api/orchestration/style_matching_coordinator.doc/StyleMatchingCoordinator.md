The orchestration glue that turns authored [`Stylesheet`]s plus the retained
tree into the [`StyleInputs`] the [`StyleCoordinator`](super::StyleCoordinator)
threads into the style resolver.

This is the seam that finally lets authored CSS reach paint: it walks every
element node in the tree, wraps it in a [`TreeElement`] cursor, runs the
`obscur-style` [`StyleMatcher`] to cascade the matching rules into that
node's winning per-property values, and records any non-empty result under
the node's id. Non-element nodes (text, synthetic parts, the root grouping
node) carry no CSS identity and are skipped.

The coordinator is zero-sized and stateless: it reimplements no matching or
cascade logic — both live entirely in `obscur-style`.
