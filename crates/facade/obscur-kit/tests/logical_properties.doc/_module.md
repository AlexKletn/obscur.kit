NOTE (2026-07-20): two RTL cases were removed here by decision. They asserted
that `margin-inline-start` / `padding-inline-start` under `direction: rtl` leave
a fixed-width block at x≈0 — but CSS 2.1 §10.3.3 over-constrained resolution moves
the block: the ignored margin (margin-left under rtl) absorbs the slack, so a 50px
item in a 300px box lands at x=210 (margin) or x=250 (padding, explicit width),
not 0. The spec-derived suite (spec_ch7_rtl) covers the same mapping correctly.
These two encoded the pre-§10.3.3 behaviour and defended it against the fix.
End-to-end: **logical box-model properties** authored in CSS reach layout, and
follow the computed `direction` (spec §4.13 "Supported: logical properties").

`inline-start` is the LEFT edge under `ltr` and the RIGHT edge under `rtl` —
from one declaration. That is the whole reason the family exists, and it is
checked here against real arranged boxes rather than computed values, so a
mapping that resolved but never moved anything would fail.
