Installs the CSS-authored motion (`transition`/`@keyframes`) the runtime
drives, replacing any previously installed set (spec §4.17 S4).

Called once the stylesheet is (re)compiled — by the facade's
`obscur_kit::bootstrap::CssPipeline::install`, which builds the rules from
the compiled sheet, rather than by each host. Each style frame the runtime
then matches these selector-bound rules against the retained tree and
registers the corresponding timelines with the
[`MotionScheduler`](obscur_motion::scheduler::MotionScheduler): a
`transition` fires when a matched element's watched property *changes*
value, and a `@keyframes` animation starts when an element first matches
the rule that references it. Installing a new set resets the per-node
change-detection baselines so the new sheet re-observes from scratch.
