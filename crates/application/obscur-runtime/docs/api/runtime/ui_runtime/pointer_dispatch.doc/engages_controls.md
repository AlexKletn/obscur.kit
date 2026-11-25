Whether this press or release is the one that works controls.

The primary button engages: it anchors a click, seats a caret, arms a pan and a
long press, and activates on release. Every other button reports and does not
engage — a right click must not press the button it lands on, a middle click
must not activate it, and a chord's second button must not consume the first
one's outstanding press.

An event that names no button is the primary one. Nothing in this workspace
filled [`PointerEvent::button`](obscur_interaction::pointer::PointerEvent::button)
before the winit runner began delivering the other four, so "unstated" has to
keep meaning what it has always meant here, and a touch contact has no button to
name.
