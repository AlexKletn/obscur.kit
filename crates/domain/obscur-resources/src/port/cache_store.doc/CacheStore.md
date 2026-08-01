The hexagonal port for the persistent derived-artifact cache (spec §5.20).

An adapter implements this over a real persistent backend, and the domain
stores it as `Box<dyn CacheStore>`; the trait is therefore deliberately
**object-safe** — no generic methods and no `Self`-returning methods. Reads
take `&self` and writes take `&mut self`. Because every entry is derived data,
the store is disposable: dropping or [`Self::clear`]ing it is always safe
(spec §5.20).
