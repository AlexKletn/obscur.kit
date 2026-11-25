Evaluates the query against `env`.

An unknown axis resolves by family, and the asymmetry is deliberate:

- An unknown **viewport** axis is non-restrictive (`true`). A live
  runtime always supplies a viewport, so `None` here only ever means a
  caller opted out of gating via [`MediaEnvironment::unbounded`].
- An unknown **container** axis never holds (`false`). `None` means no
  ancestor established a query container, and css-contain-3 §3.1 makes a
  container query against a nonexistent container *false*. Holding here
  would make every `@container` rule apply at every container size —
  which is exactly the bug this asymmetry exists to prevent.
