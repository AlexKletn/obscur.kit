Advances active motions and reports frame demand (spec §4.17).

The `MotionScheduler` is the domain-level scheduler of the motion engine: it
owns the [`ActiveMotionSet`], advances every active timeline by an explicit
`delta` each frame, drops finished ones, and reports whether another frame is
needed. It is **pure and deterministic** — it reads no wall-clock and reacts
only to explicit calls — and stays smooth under event-driven scheduling and
**skipped frames**, since a skipped frame is simply a larger `delta` absorbed
by [`Timeline::advance`]'s multi-iteration carry.

While any timeline is still running, [`needs_frame`](Self::needs_frame)
returns `true`; the runtime maps that signal to
`FrameReason::Animation` so the frame scheduler keeps requesting animation
frames until the last motion finishes (spec §4.17 "active animations keep
requesting frames"). This is the domain scheduler; the runtime
`FrameScheduler` merely consumes this demand signal.
