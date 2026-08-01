Installs the [`DiagnosticSink`] the §5.2 explicit diagnostics are emitted to.
Two kinds arrive here, each raised the first time it applies:

- a [`RendererError::UnsupportedEffect`] per effect a rendered scene demands
  that this backend reports [`Unavailable`](SupportLevel::Unavailable);
- an [`EffectFallback`] per effect it *did* draw, approximated, because the
  restriction its [`Limited`](SupportLevel::Limited) level names actually bit
  on that scene — today, a text run this renderer has no font for, which is
  dropped silently and so is invisible in the output.

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
