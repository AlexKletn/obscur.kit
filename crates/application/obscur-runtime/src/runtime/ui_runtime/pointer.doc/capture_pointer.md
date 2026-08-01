Captures `pointer` to `node`, so every subsequent event from that pointer
targets `node` instead of whatever it is over (spec §2.4).

This is how a control keeps a gesture it has claimed: a thumb being
dragged, a row a finger started on and then slid past. Capture is *per
pointer*, so capturing one finger leaves every other pointer routing
normally.

Returns `false` — capturing nothing — for a node that is not in the tree,
so a stale id cannot silently swallow a pointer's events.

The capture is released when the pointer completes ([`Up`] or [`Cancel`]),
when `node` leaves the tree, or on
[`release_pointer_capture`](Self::release_pointer_capture). §2.4 also
releases on window close and system loss; the runtime has no signal for
either yet, so those are **not** covered here.

[`Up`]: obscur_interaction::pointer::PointerEventKind::Up
[`Cancel`]: obscur_interaction::pointer::PointerEventKind::Cancel
