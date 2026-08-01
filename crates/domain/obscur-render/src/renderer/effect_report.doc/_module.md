The §5.2 explicit diagnostics: what a scene demands, what a backend will not
reproduce, what it reproduced only approximately, and how both reach a host.

§5.2 asks for two things — "unsupported effects **and fallbacks** are explicit
diagnostics" — and they are found differently. An **unsupported** effect
([`unsupported_effects`] → [`UNSUPPORTED_EFFECT`]) is answered by the backend's
[`Renderer::supports_effect`] table alone: the effect never happens, whatever the
frame. A **fallback** ([`EffectFallback`] → [`EFFECT_FALLBACK`]) is a property of
the frame: a [`SupportLevel::Limited`] effect draws correctly until the input its
restriction names is missing, so only the backend, drawing this scene, can say
that it bit. Hence the table cannot be scanned for it and each backend reports
its own from its render path.
