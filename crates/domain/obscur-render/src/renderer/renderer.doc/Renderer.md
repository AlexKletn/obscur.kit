The narrow renderer port: a replaceable 2D backend (spec §5.2).

The base 2D backend (Vello) is hidden behind this trait, and alternates —
software/headless, platform, WASM/WebGPU, and test renderers — implement it.
Every backend must reproduce the determinism boundary of spec §5.2 (layout,
hit-test, paint order, clipping, and the [`SceneSnapshot`] it consumes)
identically; only rasterization, anti-aliasing, and precision may differ.

The trait is deliberately **object-safe** — no generic or `Self`-returning
methods — so a runtime can hold the chosen backend as `Box<dyn Renderer>`.

Contract (spec §5.2): "Unsupported effects and fallbacks are explicit
diagnostics" — and backends never panic. Concretely:

1. [`supports_effect`](Self::supports_effect) answers, per [`SceneEffect`], at
   what [`SupportLevel`] this backend reproduces it. The answer must describe
   what the backend **does**, not what it wishes it did: an effect whose
   defining parameter the backend ignores is [`SupportLevel::Unavailable`],
   however much of the mark still reaches the frame.
2. A backend that draws a scene demanding an effect it reports `Unavailable`
   emits the explicit diagnostic —
   [`EffectFallbackReporter`](crate::renderer::EffectFallbackReporter) over
   [`unsupported_effects`](crate::renderer::unsupported_effects) — and then
   draws its documented fallback. Silently dropping the effect is the one thing
   §5.2 forbids.
3. An effect for which the backend has no fallback at all yields
   `Err(`[`RendererError::UnsupportedEffect`]`)`; device loss surfaces through
   the typed outcomes rather than a crash.

Both in-tree backends have a documented fallback for every effect they report
`Unavailable` (the CPU rasterizer approximates by design), so neither *returns*
`UnsupportedEffect` from `render` today — they raise it as a diagnostic instead.
