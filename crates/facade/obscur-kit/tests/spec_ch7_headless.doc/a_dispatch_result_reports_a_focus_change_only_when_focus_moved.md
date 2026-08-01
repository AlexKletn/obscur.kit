§7.4: "Dispatch results may include:" / "- target;" / "- propagation path;" /
"- default action;" / "- focus changes;" / "- state changes;" /
"- layout changes;".

OBSERVATION MOVED 2026-07-19: nothing here is weakened. The case read
`focus_changed` off the wrong dispatch. Its `click` closure issued a
`pointer_down` then a `pointer_up` and returned only the UP outcome, while
focus moves on the DOWN — so `on_b.focus_changed` was always false and case
(i) reported a missing focus-change that the runtime had in fact already
reported, one dispatch earlier. The closure now returns both halves, (i)
asserts the down reports the move, and (ii) asserts NEITHER half reports one
for a click that moves nothing, which is strictly stronger than before.

(ii)'s "no-op dispatch" premise was also false twice over, and its fixture is
repaired rather than its assertion weakened. Clicking the background while B
is focused blurs B, and a blur IS a focus change; and the element the case
calls "a non-interactive background node" is in fact focusable, so even the
first click on it with nothing focused moves focus None -> bg. Neither is a
no-op. (ii) now runs first and clicks the background TWICE, asserting on the
second — a dispatch that genuinely changes nothing. Recorded as a finding,
not acted on: that a plain container element is click-focusable at all looks
wrong against §2.5's "- tab order;" / "- focus-visible semantics;", but it is
outside this pass's scope.
