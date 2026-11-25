Samples motion tracks at a normalized progress (spec §4.17).

The coordinator is **orchestration only** and stateless: it owns no timelines
(the [`UiRuntime`](crate::UiRuntime) owns the
[`MotionScheduler`](obscur_motion::scheduler::MotionScheduler) and its
bindings) and reimplements no domain math — it merely locates the bracketing
keyframes for a progress `t`, eases the local segment through the lower
keyframe's timing function, and blends the two endpoint values, matching the
domain [`KeyframeTrack`] sampling contract.
