Emits one diagnostic per fallback in `fallbacks` not yet reported, and
returns them.

Pass the fallbacks the backend is about to take for the scene it is drawing —
the approximations its [`Renderer::supports_effect`](crate::renderer::Renderer::supports_effect) table already declares,
at the moment one of them actually stands in for what the scene asked for.
Declaring `Text` [`Limited`](obscur_capability::SupportLevel::Limited) says "a run needs its font
registered"; this says "*this* run's font is not, and here is what I drew".

Deduplicated on the whole [`EffectFallback`], reason included: a host learns
about each missing font once rather than once a frame, but two runs missing
two different fonts stay two reports.
