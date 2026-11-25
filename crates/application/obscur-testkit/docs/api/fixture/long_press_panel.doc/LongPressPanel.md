The [`InteractionPanel`] geometry with `a` bound to a **click** and `b` bound to
a click *and* a **long press** (spec §6.16).

The two gestures are told apart by which handler ran, which is why both children
bind a click: an implementation that simply broke clicking on long-pressable
elements would pass a fixture where only `b` could be clicked. `b` is the hold
target, at [`HOLD_TARGET_Y`](Self::HOLD_TARGET_Y); `a` is the control that arms
nothing, at [`TAP_TARGET_Y`](Self::TAP_TARGET_Y).

The view and the sheet are kept because a hold is the one gesture that fires
while nothing else is happening: a test has to be able to keep running frames
(`run_due_frame`) to count the countdown down.
