A deterministic reference [`FontProvider`] backed by an in-memory registry of
faces (spec §5.18).

`MockFontProvider` holds the faces registered via [`register`](Self::register)
and resolves against them with no real font files, which makes the port
exercisable in tests. Its [`resolve`](FontProvider::resolve) picks the
registered face whose family matches by [`FontFamily::name`] with the
smallest [`FontAttributes::distance`]; ties are broken deterministically in
favour of the **first** registered face. Its
[`fallback`](FontProvider::fallback) returns the requested family followed by
the generic `sans-serif`.
