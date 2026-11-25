A parsed URL-like navigation location (spec §6.13).

A `Route` models the three addressable parts of a URL-like location: a
normalized [`path`](Route::path) (always beginning with `/`), an ordered
list of [`query`](Route::query) key/value pairs, and an optional
[`fragment`](Route::fragment). It is pure data produced by
[`Route::parse`]; application navigation policy (what a location *means*)
lives outside the toolkit.

Query values are stored raw. Percent-decoding is intentionally *not*
performed here: encoding rules depend on the frontend's URL scheme, so
decoding is a frontend concern applied on top of these raw strings.
