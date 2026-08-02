§2.12: "- named slots; - one default slot;" (the partitioning half)

A `SpecPanel` declares `header` and the default slot, and renders an outlet for
each. Three children go in — one asking for `header`, two asking for nothing —
and each half must land where it was addressed and nowhere else.

WEAKENED 2026-07-19: the default half of this case was removed, because §2.12
then listed only "- named slots;" and said nothing about an unnamed one. The
`ViewNode::slot("")` outlet was the test inventing a mechanism, and a spec case
may not assert an invention. The named half was kept.

RESTORED 2026-08-02: §2.12 now states the default slot — what it is, why a text
run makes it necessary rather than convenient, and that it is declared under a
reserved symbol no author-written name can produce. `PanelProps` declares it
with `#[contract(slot(default))]`, the spelling the derive gained in the same
change, so the fixture no longer invents anything: it declares what the spec
describes and asserts what the spec says happens.

The implementation reached this before the spec did. It was built because the
base-element `<button>` could not be mounted without it — `<button>Save</button>`
states its text as a child, a text run can name no slot, and a control that could
not project it would eat the author's markup. That is the order this project
tries not to work in; `docs/component-tier.md` records how it happened.
