The hexagonal port for platform window management (spec §6.1-6.6).

Every windowing operation the domain performs funnels through this trait: an
adapter implements it once per platform, and the domain stores it as
`Box<dyn WindowBackend>`. The trait is therefore deliberately **object-safe**
— no generic methods and no `Self`-returning methods — so adapters can be
held polymorphically.

Contract: methods report honest, typed outcomes and **never panic** on an
unsupported operation. Query methods return a best-effort snapshot; mutating
methods return a typed result recording whether — and *how* — the operation
was satisfied.
