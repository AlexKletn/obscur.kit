A typed, ranged, versioned diagnostic emitted by a language service
(spec §5.13 optional language services).

Diagnostics are **typed** ([`kind`](Self::kind)/[`severity`](Self::severity)),
**ranged** ([`range`](Self::range) names the flagged bytes of the document),
and **versioned** ([`version`](Self::version) records the document/text
version the diagnostic was computed against). Because analysis is asynchronous
and incremental (§5.13), a diagnostic can arrive after the document has moved
on; a consumer discards a diagnostic whose version is stale relative to the
current document version (see [`is_stale`](Self::is_stale)).

Each diagnostic may carry replacement [`suggestions`](Self::suggestions)
(e.g. spelling candidates), most-preferred first. This type is pure data: it
performs no analysis and holds no engine state — a [`TextServiceProvider`]
produces it.

[`TextServiceProvider`]: super::TextServiceProvider
