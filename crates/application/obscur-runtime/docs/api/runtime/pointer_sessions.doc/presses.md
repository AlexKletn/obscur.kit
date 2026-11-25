The outstanding presses, keyed by the pointer holding each — the anchors
for click detection. A [`PointerEventKind::Down`] records its pointer's
press; a [`PointerEventKind::Up`] whose hit-test target equals that
press's target completes a *click* on it, and a
[`PointerEventKind::Cancel`] discards it without a click. Empty between
clicks (nothing outstanding).

Keyed rather than single because a touchscreen has one press per finger:
with one slot, a second contact overwrote the first, so the first
finger's release found the wrong anchor (or none) and activated nothing.
A mouse only ever occupies the one [`PointerId::MOUSE`] entry, so its
behaviour is unchanged.

[`PointerEventKind::Down`]: obscur_interaction::pointer::PointerEventKind::Down
[`PointerEventKind::Up`]: obscur_interaction::pointer::PointerEventKind::Up
[`PointerEventKind::Cancel`]: obscur_interaction::pointer::PointerEventKind::Cancel
