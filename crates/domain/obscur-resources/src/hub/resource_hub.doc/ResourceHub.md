The centralized, versioned registry that manages every typed resource
(spec §5.15 Resource Hub).

The hub is a **pure, deterministic in-memory state machine**. It does *not*
run loaders, decoders, or compilers and touches no wall-clock, thread, or I/O:
those live in the runtime/adapters and are driven around this registry. The
hub's responsibilities are:

- **Deduplication** — a request for an already-tracked `(source, kind)`
  returns the existing handle and bumps a reference count instead of
  allocating a second resource ([`ResourceHub::request`]).
- **Lifecycle tracking** — each resource carries a pending / loading / ready /
  failed [`ResourceState`] the hub advances
  ([`ResourceHub::begin_load`], [`ResourceHub::set_progress`],
  [`ResourceHub::complete`], [`ResourceHub::fail`]).
- **Versioning & stale-job rejection** — each resource has an authoritative
  [`ResourceVersion`]; a load job captures it at [`ResourceHub::begin_load`]
  and is rejected on completion if a hot reload has since superseded it.
- **Hot reload** — [`ResourceHub::reload`] bumps the version and restarts
  loading, invalidating any in-flight job for the old version.
- **Disposal** — [`ResourceHub::release`] evicts a resource once unreferenced,
  and [`ResourceHub::dispose_namespace`] tears down a whole scope at once.

Handles are minted from a plain monotonic `u64` counter (never random or
time-derived), mirroring the identity-allocation idiom used elsewhere in the
kit.
