A pointer was released over the hit node — `on:pointerup="…"`.

Dispatched **before** the release is resolved into a click, so a listener that
calls `prevent_default` suppresses the click, the activation it would have
dispatched (`on:click` / `on:change`) and the virtualized row-click that shares
the same release.

What a refusal does **not** suppress is the bookkeeping every release owes: the
press anchor is still consumed, the pointer is still completed, capture is still
released and a pan still in flight is still thrown. Those are not default
*actions* an application can decline — declining them would leave a stuck
`:active`, a capture nothing ever releases and a finger that lifted while the
runtime still believes it is down.
