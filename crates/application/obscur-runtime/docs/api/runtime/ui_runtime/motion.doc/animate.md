Starts an animation of `property` on `node`, sampling `track` over
`duration` logical seconds and lowering samples per `kind`, reconciling any
in-flight animation of the same target per `interruption`; returns the
[`TimelineId`] that drives the resulting motion (spec §4.17
"interruption / reversal / blend").

A fresh, already-*started* [`Timeline`] is registered with the owned
[`MotionScheduler`] and a matching [`AnimationBinding`] is recorded, so the
next frame's overlay pass samples the track at the timeline's progress and
writes the result onto `node`'s computed style before layout/paint.

When a prior binding for the **same** `node`+`property` is still present,
the [`TransitionInterruption`] policy — not a blind remove — decides what
happens (§4.17):

- [`Blend`](TransitionInterruption::Blend) (the default): the new track is
  re-based to begin at the element's **current sampled value**, so the
  redirect is seamless with no visible snap.
- [`Reverse`](TransitionInterruption::Reverse): the new track plays from the
  current value back toward the interrupted transition's origin.
- [`Restart`](TransitionInterruption::Restart): the incoming track runs from
  its authored start (the element may visibly jump).
- [`Ignore`](TransitionInterruption::Ignore): the in-flight animation is
  kept and the incoming request dropped; the existing [`TimelineId`] is
  returned and no new frame is scheduled.

For every case except [`Ignore`](TransitionInterruption::Ignore) the prior
timeline is removed, a new one is started, and a [`FrameReason::Animation`]
frame is requested so the animation reaches paint.

[`Timeline`]: obscur_motion::timeline::Timeline
[`MotionScheduler`]: obscur_motion::scheduler::MotionScheduler
[`AnimationBinding`]: crate::orchestration::AnimationBinding
