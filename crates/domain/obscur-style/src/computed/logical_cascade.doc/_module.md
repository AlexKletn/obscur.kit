Which declaration a physical box-model property takes when a **logical** one
names the same edge (spec §4.13), and the two axis keywords that decide which
edge that is.

`margin-inline-start` and `margin-left` are separate [`PropertyId`]s that
cascade independently, so the winner between them is not a cascade result — it
has to be decided here, by comparing the priorities the cascade recorded.
Deciding it needs the computed `direction` and `writing-mode`, which is why
all three live together: [`resolves_rtl`] and [`resolves_writing_mode`] exist
**only** to feed [`PropertyId::physical_under`], and the resolver reads them
before any box-model property precisely so that this file's answer cannot
depend on iteration order.
