The embeddable UI runtime and application-core integrator: [`UiRuntime`].

This file holds the struct and nothing else — the 47 fields, their
documentation, and the `Debug` that summarises them. Every method the type has
lives in one of the child modules declared above, each named for the *subject*
it answers for rather than for the order it was written in.

They are children rather than siblings for one reason, and it is the reason the
struct's encapsulation survived the split: a private field is visible to its
defining module **and its descendants**, so `ui_runtime/scrolling.rs` reaches
`self.scroll` with exactly the visibility it had when the whole `impl` was in
this file. A sibling module could not, and paying for the move in `pub(crate)`
fields would have opened all 47 of them to the crate to tidy one file.

A method that one child calls on another is `pub(super)` — visible to this
module and therefore to every child, and to nothing outside it. Reading the
`pub(super)` list is a fair way to see which of the runtime's internal seams are
actually load-bearing.
