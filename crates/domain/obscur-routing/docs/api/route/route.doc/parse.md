Parses a URL-like string into a [`Route`] (spec §6.13).

Parsing splits the input in order: the `#fragment` is removed first,
then the `?query` is removed from the remainder, and whatever is left is
the path (normalized to begin with `/`). The query is split on `&` into
pairs, and each pair on the first `=` into a key and value; a pair with
no `=` yields an empty value. Values are stored raw (see the type-level
note on percent-decoding).
