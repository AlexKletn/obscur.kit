Collapses an already-absolutized math function to the plain `px` length it is
worth, when it has one.

CSS Values 4 §10.1: a math function is valid *wherever* its resolved type is,
and one that no longer depends on anything unknown computes to the equivalent
plain value. That is not a nicety here — it is the difference between a
property working and being dropped. Downstream readers ask a computed value
for its [`Length`]; a `Calc` is not one, so every reader that does not itself
know how to fold an expression silently contributes zero. `border-radius:
min(3vmin, 14px)` painted square corners for exactly that reason, and so would
`border-width`, `outline-offset`, and every other property whose consumer
reads a length rather than an expression.

Collapsing here fixes all of them at once, because it happens at the
computed-value step every property already passes through.

An expression that still depends on something unknown stays an expression:
[`LengthExpr::resolve`] returns [`None`] for a percentage with no basis (this
context deliberately carries none — a `%` resolves against a containing block
only layout knows), for `auto`, and for an unmeasured `ch`. Those keep the
symbolic form the later passes need, so `calc(50% - 2em)` is untouched.
