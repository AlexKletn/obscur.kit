The flex container's **content box** — the containing block every item's
percentage size resolves against (`specs/w3c/css-flexbox-1.txt:640`,
vendored here: "Flex containers form a containing block for their contents
exactly like block containers do", and `:2389` for the sizing corollary:
"percentage values of flex-basis are resolved against the flex item's
containing block (i.e. its flex container)").

Carried separately from `line_main` / `container_cross` because it answers
a different question: those two are how much space there is *to give*,
this is what a percentage *means*. The cross measure below hands an item a
tight main axis — its already-flexed used size — and without this the item
re-read that number as its own containing block.

An axis of this may be **infinite**, for a container that is itself being
asked its max-content size. That is deliberate and it is the honest answer:
[`Dimension::resolve`] gives [`None`] for a non-finite base, so a
percentage against an indefinite containing block behaves as `auto`
(`specs/w3c/css-sizing-3.txt:554`) instead of quietly resolving against the
probe's own offer.
