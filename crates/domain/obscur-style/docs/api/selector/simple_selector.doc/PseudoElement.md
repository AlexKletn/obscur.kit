A pseudo-element such as `::before`; stores the name (no `::`).

Never matched by the ordinary single-element matcher: a pseudo-element
selects a *generated box* (§4.16 `::before`/`::after`), not the
originating element, so
[`SelectorMatcher`](super::selector_matcher::SelectorMatcher) fails
closed here rather than styling the originating element by mistake. The
runtime resolves generated-content rules through a dedicated
pseudo-element path (`matches_complex_for_pseudo`) that anchors the rule
on the originating element while keeping its declarations off that
element's own box.
