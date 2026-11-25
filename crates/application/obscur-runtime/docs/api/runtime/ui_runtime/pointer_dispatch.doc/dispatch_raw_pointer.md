Dispatches the §2.3 raw pointer event for this kind — `pointerdown`,
`pointerup`, `pointermove` or `pointercancel` — to the hit target through
capture / target / bubble, and reports whether a listener refused the runtime's
default action for it.

Returning `true` is what suppresses that default action, and each one is named
where it is suppressed in [`handle_pointer`](Self::handle_pointer):

| event | what a refusal suppresses |
|---|---|
| `pointerdown` | step 3 — the focus move, the caret, the press anchor, gesture arming |
| `pointerup` | step 3b — the click, the activation it dispatches, the virtual row click |
| `pointermove` | step 3a — the live drag-selection |
| `pointercancel` | **nothing, deliberately** — see below |

`pointercancel` is a documented refusal. What follows a cancel is not a default
*action* but the unwinding of state the system has already destroyed: the press
is discarded without a click, capture is released, the pointer is completed. A
runtime that honoured a refusal would keep an `:active` that never clears for a
contact that no longer exists — the exact defect step 3c was written to fix. The
event reports the loss; it does not offer to decline it.

# Cost

`pointermove` arrives at the pointer's report rate, and resolving a binding
means walking the target's ancestor chain, which allocates. That walk does not
happen unless some node in the committed tree actually binds the name:
[`PointerEventBindings`] answers that in one bit test, and it is recomputed once
per structural frame rather than once per event. The gate is tree-wide rather
than per-node for the obvious reason — a per-node answer would need the very
walk it is meant to avoid — which is the same trade
[`CommandDispatch::propagate`] already makes with the listener registry.

# What is not dispatched

A crossing **kind** (`Enter` / `Leave`) is dispatched under no name here.
`pointerenter` and `pointerleave` name a node the hover chain crossed rather
than a kind of report, so `dispatch_hover_crossings` emits them from the
transition — which already sees the crossings these two kinds cause. Naming
them here as well would dispatch each surface crossing twice.

An event with no target dispatches nothing. A raw pointer event is *about* a
node; §2.6's window and application links belong to the shortcut path, where a
chord legitimately aims past every node in the tree.

[`PointerEventBindings`]: crate::runtime::PointerEventBindings
[`CommandDispatch::propagate`]: crate::runtime::CommandDispatch
