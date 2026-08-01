The keyboard's door into the runtime: the modifier set a chord is read against,
the focused node a key is routed to, and the three things a press can be — a
text edit, a bound shortcut, or an activation of the focused control.

The order those three are tried in is the whole content of this file, and it is
not interchangeable. Editing is asked first, because a bare `a` in a field is a
character and not a shortcut; a key the editing session declines is then
resolved against the [`CommandRouter`](obscur_interaction::command::CommandRouter)
as a chord; and Enter or Space activates the focused control *in addition to*
whatever shortcut fired, so a button with no binding still responds to the key a
user expects it to. The frame request at the end is skipped for a consumed edit
for the same reason: the editing path has already chosen a possibly
compose-only frame, and overriding it with a full input frame would throw away
the per-keystroke fast path.

What each of the three then does is elsewhere — `try_text_edit` in the editing
surface, the router in [`command_routing`](super::command_routing), the
activation in `activate` — because each of them is also reachable from the
pointer. This file is only the arbitration.
