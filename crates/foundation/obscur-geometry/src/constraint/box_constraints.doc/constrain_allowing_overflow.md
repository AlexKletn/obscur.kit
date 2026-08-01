Resolves a desired size the way a box that is allowed to **overflow**
should be sized: a *tight* axis dictates, a *loose* axis only offers.

Per axis:

* **tight** (`min == max`, as built by [`tight`](Self::tight)) — the caller
  is not offering space, it is assigning a size. The desired value is
  ignored and the assigned one is used. An absolutely-positioned box with
  both insets set, and the root under the viewport, are sized this way.
* **loose** (`min < max`, as built by [`loose`](Self::loose)) — `max` is the
  space *available*, not a ceiling the content is obliged to fit. Content
  that needs more reports more, and the result exceeds `max`. The value is
  still floored at `min`.

# Why this exists next to `constrain`

[`constrain`](Self::constrain) caps at `max` unconditionally, which reports
the space a box was *offered* rather than the size it actually took. For
content that fits, the two agree. For content that does not, `constrain`
answers with a number that was never true — and the caller cannot tell,
because overflow and exact-fit come back identical. That matters most for
intrinsic sizing, where the whole question is "how much does this need?":
a `min-content` probe offers zero space, and under `constrain` every answer
would be zero.

`constrain` remains right wherever the caller genuinely means "clip this
into range" rather than "size this box".

# A zero axis reads as an offer, not an assignment

`loose(Size::ZERO)` and `tight(Size::ZERO)` are the *same value* — both are
`min == max == 0` — so at zero these two intents are genuinely
indistinguishable in this model, and this method has to pick one. It picks
**offer**: a zero axis means "there is no space here", and the box reports
what it needs.

That is the reading with something real behind it. "You have no space" is
the `min-content` probe, which must work; "you must be exactly zero" is a
degenerate assignment (an absolutely-positioned box stretched across a
zero-wide containing block), and answering it with the content's own size
costs nothing that was not already collapsed. Reading it the other way would
silently make every `min-content` answer zero.

Telling the two apart for real needs tightness carried explicitly rather
than inferred from the numbers — a change to this type, not to this method.
