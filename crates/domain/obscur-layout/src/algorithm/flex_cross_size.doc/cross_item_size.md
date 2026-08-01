Resolves one item's cross-axis size.

When `stretch` is set, the item fills `stretch_target` unless it sets an
explicit cross-axis size (which wins, expanded to its border-box extent).
Otherwise the item's cross size is measured with its final main size fixed and
a loose cross axis bounded by `container_cross`.

# Why the measure states a percentage basis

The measure below fixes the main axis to `main`, the item's **used** main size
after flexing. That is the right thing to offer — the item must report the
cross size it takes at the width it will actually be laid out at — but it is
not the item's containing block, and the two travelled as one value: an item
at `width: 50%` in a 400-wide row was measured with `max.width == 200`, read
*that* back as its basis, and laid its own content out at 100. Anything whose
block size depends on its inline size then answered for the wrong box — a
wrapping inner row reported two lines where its arranged self has one, so the
item's measured height and its arranged content disagreed.

`percent_basis` is the flex container's content box, which is what a flex
item's percentages resolve against: `specs/w3c/css-flexbox-1.txt:640`,
vendored here — "Flex containers form a containing block for their contents
exactly like block containers do" — and `:2389`, "percentage values of
flex-basis are resolved against the flex item's containing block (i.e. its
flex container)". It is also, exactly, the number this algorithm already
resolved the item's own main size against (`main_dim.resolve(container_main)`
above), so stating it is what makes the item's two passes agree.

This is the flex sibling of the audit's grid finding 2-4 (a)
(`docs/external-consumer-audit.md`): the same defect — a caller that knows the
containing block and does not say it — in a different algorithm's clothes.
