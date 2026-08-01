The style-matching entry point: selects the rules that apply to an element
and hands the resulting declarations to the [`Cascade`] (§4.14).

[`StyleMatcher`] is a zero-sized handle. Given an ordered slice of
[`Stylesheet`]s and a single element, it matches every rule's selector list
against the element, contextualises the matched rules' authored declarations
with their origin, matched specificity and document position, and returns the
cascaded value per property by reusing [`Cascade::resolve`]. It never
reimplements the cascade ordering.

# Ordering and `source_order`

The caller supplies `sheets` ordered by **ascending cascade origin**
(system-default → theme → global-app → component/module → inline). The
matcher walks the sheets in that order and, for every
[`BlockDeclaration`](crate::matching::BlockDeclaration) it *encounters* in
document order — whether or not its rule matched — increments a single
monotonic `source_order` counter. Later declarations therefore receive a
higher `source_order`, so on an otherwise-exact tie the later declaration
wins (§4.14 "order of appearance").

Because [`CascadePriority`](crate::cascade::CascadePriority) makes the origin
band dominate specificity, the order of `sheets` only affects tie-breaks
*within* a single origin band; it does not let a low-origin sheet outrank a
high-origin one. Ordering the sheets correctly still matters so that
`source_order` reflects true document position across origins.
