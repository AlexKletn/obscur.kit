Seats everything a [`Down`](obscur_interaction::pointer::PointerEventKind::Down)
establishes: focus, the editing session for the focused editable, the press
anchor that a later release is matched against, the caret or the word/line the
click selected, and the gestures the contact speculatively armed. Returns
whether focus moved, and reports through `press_changed` whether the `:active`
styling state changed.

Focus itself is not moved here. It moves through
[`move_focus_to`](UiRuntime::move_focus_to), which is the one door in the
runtime that assigns it — the editing session it re-seats, §6.15's
scroll-into-view and §2.3's `blur` / `focus` pair are all on the other side of
that call. This press is its only caller today; that is what makes it the only
*path*, not what makes it the only *door*.

A press inside a **disabled** subtree seats none of it and — deliberately —
blurs nothing either: clicking a disabled control is not a blur, so focus, the
editing session and the press anchor stay exactly where they were, and no pair
is announced for a move that did not happen. Only the gesture arming still runs,
because a finger resting on a disabled control can still be starting a scroll of
the list that contains it.
