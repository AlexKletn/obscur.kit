Installs the [`DiagnosticSink`] the §5.2 explicit fallback diagnostics are
emitted to.

This backend approximates the GPU-class effects by design (see
[`Renderer::supports_effect`]); the sink is how a host is *told* which of the
effects its scene asked for will not appear, rather than discovering it by
looking at the pixels. Two kinds arrive here, each once:

- a [`RendererError::UnsupportedEffect`] per effect a rendered scene demands
  that this backend reports [`Unavailable`](SupportLevel::Unavailable);
- an [`EffectFallback`] per effect it *did* draw, approximated, because the
  restriction its [`Limited`](SupportLevel::Limited) level names actually bit
  on that scene — today, a text run whose font is not registered here.

Not every approximation is announced — the text fallback above is the only one
today. [`Renderer::supports_effect`] remains the complete statement of what
this backend does with an effect.

Installing a sink is the **host's** job and nothing in this workspace does it:
the default is a [`NullDiagnosticSink`], and the `Renderer` port carries no
sink, so a generic runtime cannot install one — only whoever constructs this
concrete backend can. Until it is called, both diagnostics above are built and
dropped. That is deliberate (the render path does not branch on whether anyone
is listening), but it does mean §5.2's "explicit diagnostics" are *available*
here, not *delivered*.
