Runs the change handler registered for `command` (if any) with the field's
committed `text` and, when one ran, requests a [`FrameReason::Input`] frame.
Returns whether a handler actually ran. With no handler registered this is a
no-op that returns `false` and schedules no frame.
