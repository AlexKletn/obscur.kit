A request to analyze a region of a document with the language services
(spec §5.13 optional language services).

Analysis is **incremental and versioned** (§5.13): [`text`](Self::text) is the
slice to analyze — the whole document or an incremental fragment —
[`range`](Self::range) locates that slice within the document (so diagnostics
can be mapped back to document byte offsets), and [`version`](Self::version)
records the text version so the resulting diagnostics can be dated.

[`services`](Self::services) selects which analyses to run — an **empty** list
means "all" (see [`wants`](Self::wants)). [`ignored_ranges`](Self::ignored_ranges)
lists **ignored / sensitive** document regions a provider must skip (§5.13),
for example code spans, URLs, or password fields; an offset inside any ignored
range is reported by [`is_ignored`](Self::is_ignored).

This type is pure data: it carries a request but performs no analysis; a
[`TextServiceProvider`] consumes it.

[`TextServiceProvider`]: super::TextServiceProvider
