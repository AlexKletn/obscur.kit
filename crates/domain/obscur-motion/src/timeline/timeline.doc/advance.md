Advances the timeline by `delta` logical seconds — the deterministic
integrator (spec §4.17).

Only a [`Running`](TimelineState::Running) timeline advances; otherwise
this is a no-op. `delta` is added to `elapsed` in the current direction
(forward increases it, reverse decreases it) and progress is recomputed as
`(elapsed / duration).clamp(0, 1)`.

**Multi-iteration carry.** A single large `delta` — for example one
spanning several skipped frames — may complete more than one iteration. To
handle that, the method consumes the delta in a `while` loop: each pass
fills the time remaining in the current iteration, and any *overshoot* is
carried into the next iteration rather than discarded. When an iteration
completes the [`iteration`](TimelinePosition::iteration) counter increments
(saturating) and, if [`alternates`](Self::alternating) is set, the
direction flips. The loop stops when the delta is exhausted or the repeat
count runs out; in the latter case the position is clamped to the end and
the state becomes [`Finished`](TimelineState::Finished). An
[`Infinite`](RepeatCount::Infinite) timeline never finishes.
