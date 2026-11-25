Runs the text-input handler registered for `command` (if any) with the
field's resulting `text` and, when one ran, requests a
[`FrameReason::Input`] frame so the app's mirrored text reaches paint on
the next reactive frame. Returns whether a handler actually ran.

With no handler registered this is a no-op that returns `false` and
schedules no frame — a bound `on-input` whose handler the app never
registered can never panic or perturb runtime state.
