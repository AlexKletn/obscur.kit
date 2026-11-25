The offset a [`Position::Relative`] node adds to its final in-flow
position (and to its whole subtree), in logical pixels (spec §4.1).

Returns `(dx, dy)`, computed from the node's [`inset`](Self::inset)
against the containing block size (`containing_width` / `containing_height`,
the parent content box). Per CSS `left` wins over `right` and `top` over
`bottom`:

* `dx` = `left` if definite, else `-right` if definite, else `0`.
* `dy` = `top` if definite, else `-bottom` if definite, else `0`.

A positive `left` moves the node right; a positive `right` moves it left
(and likewise for `top` / `bottom` on the y axis). Any non-[`Relative`]
scheme — including the [`Position::Absolute`] / [`Position::Fixed`]
placeholders, which are treated as [`Position::Static`] this step —
returns `(0.0, 0.0)`, so a default node is never perturbed.
