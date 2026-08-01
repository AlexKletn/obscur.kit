§5.2 "Unsupported effects and fallbacks are explicit diagnostics." Diagnostic
shape from §1.6: "diagnostics include severity, subsystem, stable code,
node/resource/window context and recovery action."

OBSERVATION MOVED + WEAKENED 2026-07-19.

The requirement is met, and the case was watching the wrong wire. It read the
UiRuntime's `DiagnosticSink` after `render_now`, but the runtime does not
rasterize and therefore cannot know what a backend will refuse. The report is
the RENDERER's: `unsupported_effects(&renderer, list)` asks the backend's
capability table what it cannot honour, and `EffectFallbackReporter::report`
emits one `Diagnostic` per effect — code `UNSUPPORTED_EFFECT`, severity
Warning, subsystem Render, recovery Fallback, deduplicated so a per-frame
effect is reported once rather than once a frame
(obscur-render/src/renderer/effect_report.rs:204-247). The case's own
`rasterize()` helper drove `CpuRenderer::render` directly, bypassing that
entirely. The observation now sits where the report is produced.

DROPPED: `d.context.node.is_some()`. A `SceneEffect` is a property of the
display list, not of a node — nothing at that layer knows which node asked
for the blur — and §1.6's phrase is "node/resource/window context", a
disjunction, not a demand for all three on every diagnostic. Candidate spec
amendment: say which of the three contexts is required for a render-class
diagnostic, and if node context is wanted for unsupported effects, carry the
originating node on the scene command so it can be.
