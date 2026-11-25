The host-provided port for satisfying network fetches (spec §5.15).

The kit opens no sockets: an adapter implements this once per host (a real
HTTP client, a sandboxed loader, a test double) and the domain drives it as
`Box<dyn NetworkService>`. The trait is therefore deliberately
**object-safe** — no generic methods and no `Self`-returning methods.
Fetches take `&mut self` because a real host mutates connection/pool state
as it works; failures are reported through [`NetworkError`] rather than
panics.
