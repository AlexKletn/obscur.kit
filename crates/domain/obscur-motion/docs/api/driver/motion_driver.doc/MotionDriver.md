A source of normalized motion progress sampled by tracks (spec §4.17).

Every driver — [`TimeDriver`](super::TimeDriver),
[`ProgressDriver`](super::ProgressDriver),
[`SignalDriver`](super::SignalDriver),
[`ScrollDriver`](super::ScrollDriver) — implements this port: it reports the
[`progress`](Self::progress) it currently dictates and whether it
[`is_active`](Self::is_active). The trait is deliberately **object-safe** (no
generics, no `Self`-returning methods) so drivers of different kinds can be
held together as `Box<dyn MotionDriver>` or borrowed as `&dyn MotionDriver`.
