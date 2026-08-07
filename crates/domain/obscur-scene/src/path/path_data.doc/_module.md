SVG path data — `"M 0 0 L 100 0 A 50 50 0 0 1 200 0"` — as a [`Path`].

The road that lets a shape ride in a node. A `ViewValue` carries text and not
geometry, and `obscur-view` does not depend on this crate, so a path cannot be
a property; a STRING can, and this is what it means. It also makes a shape
authorable from markup, which a Rust-only `Path` never would be.

Arcs are parsed rather than refused. An author drawing a gauge writes `A`, not
four cubics, and a parser that stops at the interesting command has not made
arbitrary shapes possible. They are converted by SVG 1.1 F.6's endpoint-to-
centre parameterisation and emitted as one cubic per quarter turn.

It fails closed and says where: a number before any command, an unknown verb, a
command missing its arguments and a number trailing a `Z` — which takes none —
are all refused with a byte offset, rather than guessed into a drawing the
author did not write.
