A builder that appends [`PaintCommand`]s in paint order while tracking
push/pop nesting (spec §5.1).

Commands are appended in the exact order they are issued; that order becomes
the display list's deterministic paint order. The builder maintains a live
nesting [`depth`] using saturating arithmetic, so a stray pop can never
underflow.

The builder does **not** enforce balance: [`build`] returns the accumulated
commands verbatim without auto-closing any open clips or layers. An
unbalanced builder therefore yields an unbalanced [`DisplayList`], which the
caller can detect via [`DisplayList::is_balanced`]. A `pop_*` call on an
empty stack is recorded verbatim as its command, but keeps [`depth`] clamped
at zero so it never underflows or panics.

[`depth`]: DisplayListBuilder::depth
[`build`]: DisplayListBuilder::build
