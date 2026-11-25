The injected loader that resolves a `background-image: url(...)` source's
raw encoded bytes (spec §5.15), or [`None`] when no loader was installed —
in which case `url()` backgrounds silently resolve to nothing (the
honest-capability contract). The runtime holds only the domain
[`ResourceLoader`] port; the facade injects a concrete adapter (e.g. an
`FsResourceLoader`) via [`set_providers`](Self::set_providers).
