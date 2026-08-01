A flex item's own percentage size resolves against the **flex container's
content box**, in the cross-size measure as well as in the main-size solve.

`specs/w3c/css-flexbox-1.txt:640`, vendored here: "Flex containers form a
containing block for their contents exactly like block containers do." The
sizing corollary is at `:2389` — "percentage values of flex-basis are resolved
against the flex item's containing block (i.e. its flex container)".

The defect this pins is the audit's grid finding 2-4 (a) wearing flex's
clothes (`docs/external-consumer-audit.md`): a caller that knows the
containing block and does not say it. Flex resolves an item's `width: 50%`
against the container correctly, then measures that item's cross size with the
main axis pinned to the used size it just computed — and handed no basis, so
the item read *that* number back as its containing block and laid its own
content out at half again. Anything whose block size depends on its inline
size then answered for a box that does not exist: the item below measured two
wrapped lines (120) where its arranged self has one (60), so the item's height
and its own content disagreed on screen.

The geometry is chosen to be font-free: `flex-wrap` on two rigid 60px boxes
couples block size to inline size without a single glyph, so the numbers here
are decided by the flex algorithm alone.
