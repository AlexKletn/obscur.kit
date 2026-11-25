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
the `Renderer` port carries no sink, so a generic runtime cannot install one —
only whoever constructs this concrete backend can. Until it is called, the two
scans that raise these diagnostics do not run at all: each is a full walk of
the display list, every frame, and with no sink their every product goes to a
sink whose `emit` has an empty body. Calling this turns them back on from the
next frame. §5.2's "explicit diagnostics" are therefore *available* here, not
*delivered* — unchanged from before, except that being unavailable now also
costs nothing.

Raised-once suppression lives in [`EffectFallbackReporter`] and is not rewound
by this call: a scene that earned a diagnostic while no sink was installed
raises it on the first frame after one is.
