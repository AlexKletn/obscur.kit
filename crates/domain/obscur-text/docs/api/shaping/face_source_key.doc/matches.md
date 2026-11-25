Whether this key was derived from `source`, compared **in place** against a
borrowed [`FontSource`] instead of building a second key to compare against.

The point is that a cache probe allocates nothing:
[`from_source`](Self::from_source) clones the name into a `Box<str>`, so
calling it on the hit path would allocate once per lookup. The `Memory` arm
compares the same thinned pointer + length identity `from_source` records,
so it never touches the font bytes.
