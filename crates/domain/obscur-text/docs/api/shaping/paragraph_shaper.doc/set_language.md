Installs the BCP-47 `language` hint every [`ShapingRequest`] this
orchestrator issues will carry (spec §5.10, §7.2 text shaping
integration).

The language is a property of the *content's locale*, not of a run: it is
uniform across a paragraph and constant until the application changes
locale, so it is installed here alongside the ports rather than threaded
through [`shape_paragraph`](Self::shape_paragraph)'s signature. `None` —
the default — leaves the hint unset, and a shaper then guesses from the
text.

It matters because a shaper cannot infer it. Serbian and Russian are both
`Cyrl`, and their italic б differ; `locl` is the OpenType feature that
picks, and it is driven by the language tag alone. The
[`ShapingRequest::language`](crate::shaping::ShapingRequest::language)
field has existed all along and nothing ever set it — this is the setter
that gives it a value.

The paragraph cache is flushed, for the same reason a port swap flushes
it: the memoized runs were shaped under the outgoing hint.
