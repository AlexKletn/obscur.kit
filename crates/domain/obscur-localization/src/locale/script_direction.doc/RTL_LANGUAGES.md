Every language whose *likely* script is right-to-left, sorted for binary
search.

This table is what makes a bare `ar` — which is how a locale is actually
spelled; nobody writes `ar-Arab` — resolve to
[`TextDirection::RightToLeft`]. It is **derived**, not transcribed: it is
every bare language subtag in CLDR release-48's `likelySubtags.xml` whose
likely script appears in [`RTL_SCRIPTS`]. That file is not vendored (900 KB,
and no line of it is quoted); `specs/cldr/README.md` records the pinned URL
and the exact derivation.

All 258 of them, including the deprecated aliases CLDR still carries (`iw`
for `he`, `ji` for `yi`, `in` is *not* here because Indonesian is LTR). A
hand-picked shortlist would have been the Anglocentric failure in a different
costume: it would have covered Arabic, Hebrew and Persian and quietly laid out
Sindhi, N'Ko and Dhivehi backwards.
