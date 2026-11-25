The port for resolving the bytes of a lazily-offered representation on demand
(spec §6.20 lazy async providers).

The trait is deliberately **object-safe** — no generic or `Self`-returning
methods — so a provider can be stored as `Box<dyn LazyDataProvider>` and held
polymorphically alongside an offer. A provider knows which MIME types it can
supply ([`Self::available_mimes`]) and resolves any one of them to bytes on
request ([`Self::provide`]), returning `None` when it cannot.
