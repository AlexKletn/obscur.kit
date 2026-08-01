The hexagonal port for decoding encoded bytes into a CPU representation (spec
§5.15 decode).

An adapter implements this once per format family (image, svg, font, CSS),
and the domain stores it as `Box<dyn ResourceDecoder>`; the trait is
therefore deliberately **object-safe** — no generic methods and no
`Self`-returning methods — so decoders can be held polymorphically and
composed. A decoder reports failure through [`ResourceError`] rather than
panicking.
