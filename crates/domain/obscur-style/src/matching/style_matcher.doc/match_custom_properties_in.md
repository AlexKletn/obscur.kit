Matches `element` against all `sheets` in `environment` and returns the
custom properties (`--name: value`) that won the cascade for it, keyed by
name.

The companion to [`match_element_in`](Self::match_element_in), and separate
from it for the reason §4.14 gives: custom properties cascade BEFORE
computed-value resolution, because every `var()` a longhand carries is
substituted out of the way before that longhand is computed. They are keyed
by name rather than by [`PropertyId`](crate::property::PropertyId), so they
cannot ride in a [`ResolvedDeclarations`](crate::cascade::ResolvedDeclarations)
at all.

Only the declaration that wins by origin, then specificity, then document
order survives per name — the same order [`match_element_in`](Self::match_element_in)
resolves by. A rule that declares no custom property is skipped before its
selectors are matched: almost every rule declares none, so a sheet that uses
no custom properties costs one branch per rule and nothing else.
