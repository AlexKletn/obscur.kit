The replaceable port that chooses a font for one itemized run (spec §5.10
font fallback).

Given a [`FontResolveRequest`] — a fallback chain, the requested attributes,
and the run's text and script — a resolver returns the **first** font in the
chain that can render the run, as a [`ResolvedFont`]. `None` means nothing in
the chain covers the run, leaving the caller to fall back to a default face.

The domain holds a resolver as `Box<dyn FontResolver>`, so the trait is
deliberately **object-safe** — no generic or `Self`-returning methods. The
colocated [`MockFontResolver`] is a deterministic reference implementation
that maps scripts to fonts directly; the real, coverage-driven resolver that
inspects a font's character map lives outside the domain.
