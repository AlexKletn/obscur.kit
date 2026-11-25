A source of presentation values sampled by normalized progress (spec §4.17).

Every track — [`TransitionTrack`](super::TransitionTrack),
[`KeyframeTrack`](super::KeyframeTrack),
[`SpringTrack`](super::SpringTrack) — implements this one method: given a
normalized progress `t`, produce the [`MotionValue`] to present. The trait is
deliberately **object-safe** (no generics, no `Self`-returning methods) so a
heterogeneous set of tracks can be held as `Box<dyn MotionTrack>`.

`t` is normalized progress in `[0, 1]`. Implementations that shape `t`
through a bounded easing curve clamp it to `[0, 1]` first (documented on each
impl); spring-shaped tracks intentionally do *not* clamp, because a spring
overshoots past its target before settling.
