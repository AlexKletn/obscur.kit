Seats everything a [`Down`](obscur_interaction::pointer::PointerEventKind::Down)
establishes: focus, the editing session for the focused editable, the press
anchor that a later release is matched against, the caret or the word/line the
click selected, and the gestures the contact speculatively armed. Returns
whether focus moved, and reports through `press_changed` whether the `:active`
styling state changed.

A press inside a **disabled** subtree seats none of it and — deliberately —
blurs nothing either: clicking a disabled control is not a blur, so focus, the
editing session and the press anchor stay exactly where they were. Only the
gesture arming still runs, because a finger resting on a disabled control can
still be starting a scroll of the list that contains it.
