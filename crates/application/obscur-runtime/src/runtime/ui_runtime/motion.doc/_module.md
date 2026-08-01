[`UiRuntime`]'s motion surface (spec §4.17): the CSS transition/`@keyframes`
rules a stylesheet compiles down to, the two entry points that start a timeline,
the tick that advances every running one, and the two readings of how many
timelines are running.

Starting and advancing live here rather than on
[`MotionDriver`](crate::runtime::MotionDriver) for the reason the driver's own
boundary states: both owe the scheduler a frame, and the driver is not allowed
to hold the scheduler. `animate_with` starts the timeline and then asks for an
[`FrameReason::Animation`] frame; `advance_motion` asks for one only while
motion remains. A driver that could request its own frames would be a second
frame source, and §4.17's "active animations keep requesting frames" would have
two implementations to keep in step.

`advance_motion` also drives the two gesture clocks — the fling coast and the
long-press hold — because it is the runtime's one "time passed" signal, the tick
a host already calls. Giving inertia its own entry point would mean every host
had to learn a second clock, and the one that forgot would have inertia only on
the frames something else happened to be animating.

The two readings are deliberately both here. `active_animations` is §4.17's
keep-ticking signal — the answer to "does this runtime still owe a frame?" —
and `active_animation_count` is §7.6's DevTools counter; they are the same
number arrived at by two routes, and each is public because a different consumer
reads it.
