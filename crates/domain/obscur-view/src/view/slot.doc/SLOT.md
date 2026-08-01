Reserved symbol for a child's `slot="…"` assignment — the name of the slot
the child is projected into (spec §2.12).

Deliberately **not** a [`StyleAttributes`](super::StyleAttributes) member: a
slot assignment is structure, not a CSS selector input, so `extract` never
sees it and no selector can match on it. It lives in the node's ordinary
property list, which is where the reconciler and [`project_slots`] read it —
the same shape [`TEST_ID`](super::TEST_ID) uses, and for the same reason.

It occupies a namespace of its own (`0x0000_03xx`), distinct from the
CSS-relevant reserved attributes (`0x0000_000x`), the reserved events
(`0x0000_010x`) and the stable selectors (`0x0000_02xx`).
