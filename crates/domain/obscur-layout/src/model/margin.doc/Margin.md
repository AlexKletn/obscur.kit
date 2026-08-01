A single margin edge (spec §4.2, CSS 2.1 §8.3).

A margin is a length in logical pixels or the `auto` keyword — and `auto` is
not a number that can be substituted early: it means *"take what is left
over"*, and how much that is depends on the box's resolved size, which is not
known until it has been measured. `margin: 0 auto` centring is exactly that
rule doing its job.

# Why this is not `Dimension`

[`Dimension`](super::Dimension) already models `Auto` / `Px` / `Percent`, and
reusing it here would be a smaller diff. It would also let a caller *say*
`margin-left: max-content`, which is not a thing: since intrinsic sizing landed,
`Dimension` carries `MinContent` / `MaxContent` / `FitContent`, and a type that
can express nonsense pushes the job of rejecting it onto every reader. This
enum says exactly what a margin can be, so there is nothing to reject.
