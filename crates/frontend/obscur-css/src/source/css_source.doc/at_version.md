Returns this source at revision `version` (fluent).

An owner that re-reads a stylesheet from disk stamps the new text with the
next revision, which is what invalidates a
[`StylesheetCache`](crate::cache::StylesheetCache) entry — explicitly, per
spec §4.12, rather than by the cache re-hashing text behind the owner's back.
