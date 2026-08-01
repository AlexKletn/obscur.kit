A single directional selection over a UTF-8 text buffer (spec §5.11 editing
engine).

A selection is defined by two *byte* offsets: the [`anchor`](Self::anchor),
the fixed end that stays put while the selection is extended, and the
[`focus`](Self::focus), the moving end that follows the caret. The order of
the two encodes direction — a forward selection has `focus >= anchor`, a
backward one has `focus < anchor` — which drives shift-arrow extension and
caret placement. A [`CaretAffinity`] disambiguates the focus when it lands on
a soft line-wrap boundary. When `anchor == focus` the selection is
*collapsed* and behaves as a plain caret.

This type models exactly one selection. Multi-selection (several disjoint
ranges edited together) is a set of `TextSelection`s owned and reconciled by
the buffer/editing engine, not represented here.
