The replaceable port for capturing windows, surfaces, nodes, and regions
(spec §6.17).

An adapter implements this trait once per platform and the domain holds it as
`Box<dyn CaptureProvider>`. The trait is deliberately **object-safe** — no
generic or `Self`-returning methods — so providers can be stored
polymorphically.

Contract: a provider reports honest, typed outcomes and **never panics** on a
source it cannot capture — it answers [`CaptureResult::Unsupported`] (or
[`CaptureResult::Denied`]) instead (spec §6.17).
