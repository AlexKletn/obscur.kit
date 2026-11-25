Runtime-facing CSS motion inputs: [`CssMotionRules`] and its rule types.

The CSS frontend lowers `transition`/`@keyframes` into `obscur-motion` tracks
and keeps them associated with the selector the rule matched. This module is
the runtime's *domain-only* view of that association — [`SelectorList`],
[`PropertyId`], [`Easing`], [`KeyframeTrack`] and durations, with no
dependency on the CSS crate. Building one from a compiled stylesheet and
installing it with [`set_css_motion`](crate::UiRuntime::set_css_motion) is the
facade's job — `obscur_kit::bootstrap::CssPipeline::install` does it, so a
host that installs CSS through the kit never touches these types. Each style
frame the
runtime matches these rules against the retained tree and drives the motion
through the [`MotionScheduler`](obscur_motion::scheduler::MotionScheduler)
(spec §4.17).
