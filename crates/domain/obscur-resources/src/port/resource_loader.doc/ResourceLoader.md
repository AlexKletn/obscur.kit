The hexagonal port for loading a resource's raw encoded bytes (spec §5.15
custom loaders).

An adapter implements this once per loading strategy, and the domain stores
it as `Box<dyn ResourceLoader>`; the trait is therefore deliberately
**object-safe** — no generic methods and no `Self`-returning methods — so
loaders can be held polymorphically and composed. A loader turns the pure
*description* of a [`ResourceSource`] into the encoded bytes behind it,
reporting failure through [`ResourceError`] rather than panicking.
