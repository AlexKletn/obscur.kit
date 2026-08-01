The long press to arm for a **direct** contact that went down on `target`,
or [`None`] when nothing there would answer one (spec §6.16).

Armed only where the gesture is actually bound, which is not an
optimisation but the difference between an idle app and one that renders
continuously: a long press is the one gesture that must fire while nothing
is happening, so an armed hold keeps asking for frames until it resolves.
Arming on every contact would spend half a second of frames on every tap
in the application, almost always to conclude nothing.

Direct devices only. A mouse has a second button for what a hold reaches.
