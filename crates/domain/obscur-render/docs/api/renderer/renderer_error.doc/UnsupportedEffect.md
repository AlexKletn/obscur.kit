The backend cannot reproduce the named effect at all. This is the
*unsupported effect* half of spec §5.2's explicit diagnostics, in the form
[`SceneEffect::name`](crate::renderer::SceneEffect::name) gives it. The
*fallback* half — an effect that was drawn, but approximated — is
[`EffectFallback`](crate::renderer::EffectFallback), which is not an error:
nothing failed.

Both in-tree backends **emit** it rather than **return** it: they own a
documented fallback for every effect they report
[`Unavailable`](obscur_capability::SupportLevel::Unavailable) (the CPU
rasterizer approximates by design), so they draw the frame and raise this
through their diagnostic sink — see
[`EffectFallbackReporter`](crate::renderer::EffectFallbackReporter). It is
returned from [`render`](crate::renderer::Renderer::render) only by a backend
with no fallback at all, and no such backend exists in this workspace today.
