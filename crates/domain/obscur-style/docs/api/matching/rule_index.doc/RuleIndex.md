A bucketed view of one ordered set of [`Stylesheet`]s: `class -> rules`,
`id -> rules`, `type -> rules`, plus the rules that fit no bucket.

Built once per sheet set and consulted once per element, it answers a single
question — *which rules are worth calling
[`SelectorMatcher::matches_list`](crate::selector::SelectorMatcher::matches_list)
on* — and answers nothing else. Every candidate it yields is still matched by
the same selector engine, with the same arguments, that the unindexed walk
uses; the index only removes calls whose answer is provably `None`.

# The invariant the whole thing rests on

A complex selector matches an element only if the element matches the
selector's **subject** compound (the rightmost one) — that is where
[`SelectorMatcher::matches_complex`](crate::selector::SelectorMatcher::matches_complex)
starts. So if the subject compound contains `#main`, the element's id *must*
be `main`; if it contains `.key`, the element *must* carry the class `key`; if
it contains `button`, the element's local name *must* be `button`
(ASCII-case-insensitively). Each of those is a necessary condition, so a rule
whose every selector carries one can be filed under it and skipped for every
element that fails it.

A selector whose subject compound has no id, class or type — `*`, `[data-x]`,
`:hover` alone, `div > *` — states no such necessary condition, and its rule
goes into the **unkeyed** bucket, which is offered to *every* element. Losing
one of those is the failure this design is most exposed to, and the test named
`the_bucketless_rules_are_offered_to_every_element` is the guard against it.

# What it does not do

It does not decide matches, cascade order, specificity or conditions. It does
not know about `::part()` scoping — the component-boundary path
(`scoped_internal_declarations` in the runtime) keys on exported part names
rather than on the subject compound, and is not indexed by this type.

# Ordering

Candidates are yielded in ascending **global rule index** — sheet order first,
then rule order within the sheet — which is exactly the order the unindexed
walk visits rules in. Because each candidate also carries the global
declaration index its block starts at, a skipped rule still contributes the
same `source_order` shift it would have contributed by being walked and
rejected. Document-order tie-breaks are therefore unchanged, which is the
whole correctness argument for skipping rules at all.
