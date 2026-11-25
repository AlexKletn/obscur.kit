The alignment a grid item actually gets on one axis, after `stretch` has been
tested against the precondition the keyword carries.

`specs/w3c/css-align-3.txt:1377` defines the value, not merely its effect:

> **stretch** — When the box's computed width/height (as appropriate to the
> axis) **is auto** and neither of its margins (in the appropriate axis) are
> auto, sets the box's used size to the length necessary to make its outer
> size as close to filling the alignment container as possible … Unless
> otherwise specified, this value falls back to **flex-start** generally.

So `stretch` is conditional, and `size` — the item's computed size on *this*
axis — is the condition. A box that has a size of its own is not stretched at
all: the keyword falls back to `flex-start`, which keeps the declared size and
puts the box at the start of its grid area. The kit used to stretch
regardless, so a `width: 50px` item in a 211px track came out 211 wide,
silently discarding the author's length — while `justify-self: start` on the
same rule gave 50, which is how the external-consumer audit knew the declared
value was understood and the *alignment* was overriding it
(`docs/external-consumer-audit.md`, 2-4 (b)). Both axes are tested, because
the spec's condition is written "as appropriate to the axis".

The test is on the box's COMPUTED size, not on how the alignment value was
arrived at. An author writing `justify-self: stretch` on a `width: 50px` item
gets exactly the same fallback as one who wrote nothing and inherited the
container's `stretch` default: the keyword means the same thing either way.

An intrinsic keyword is not `auto` either, so `width: min-content` also falls
back — correctly, since the closing line of the same definition ("Values other
than stretch cause a width/height of auto to be treated as fit-content") is
what makes the fallback path measure the item shrink-to-fit rather than at the
area extent.

# The half of the condition this does not test, and why

"neither of its margins (in the appropriate axis) are auto" is deliberately
left out: this algorithm does not read item margins at all — `margin` does not
appear anywhere else in this file — so an `auto` margin on a grid item moves
nothing today. That clause exists so an `auto` margin can absorb the area's
free space instead of the box growing into it (css-grid-2 §10.2, "Aligning
with auto margins"). Honouring half of that pair would swap one wrong answer
for another: a `width: auto; margin-left: auto` item would stop stretching and
then sit at the *start* of its area, when the entire point of the margin is to
push it to the end. It keeps stretching — wrong in the direction it was
already wrong — until grid consumes item margins, and this is the line that
then gains the second half of the test.
