CSS custom properties, end to end: declared in a sheet or on an element,
cascaded, inherited, and substituted into the longhand that reads them
(§4.14 — "custom properties participate in cascade and inheritance before
computed-value resolution").

This suite exists because every part of that sentence was implemented and
none of it was connected. The compiler parsed `--brand`, the resolver knew
how to substitute a `var()`, and in between the declarations were filtered
out of the rule that declared them and never handed to the runtime — so
every `var()` in every stylesheet computed to `initial`, with no diagnostic
to say so. Nothing in the workspace noticed, because nothing asked what
colour a `var()` produced through the assembled pipeline.

The tests therefore go through [`CssPipeline`](obscur_kit::bootstrap::CssPipeline)
and a real [`UiRuntime`](obscur_kit::runtime::UiRuntime) frame rather than
through the compiler or the resolver alone: the defect lived in neither of
them, only in the seam.
