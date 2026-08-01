A percentage size resolves against its **containing block's content box** —
including inside a parent whose own width is a percentage (CSS 2.1 §10.2).

The defect this pins: a box arranged at a decided size re-read its own
authored `width: 100%` and resolved it against the constraint it had just
been handed — its own border box — so it came back one padding wider, and the
error compounded one nesting level at a time until the content walked out of
the window. A fixed `px` width resolves to the same number against any basis,
which is exactly why the bug hid: only percentages diverged, and only under a
padded percentage-width ancestor.

The fix states the containing block explicitly (`BoxConstraints::percent_basis`)
rather than letting a decided box re-derive it — Servo's
`SizeConstraint::Definite`, in its first half.
