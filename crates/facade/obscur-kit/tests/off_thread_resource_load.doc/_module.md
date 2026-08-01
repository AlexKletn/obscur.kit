§1.5 parallelism, proved on the one facility that has a port for it: a
resource load must run on **another thread**, and the deterministic harness
must keep running it on the caller's.

§1.5 lists eight candidates for parallel work — resource loading, image
decoding, CSS parsing, text shaping, isolated layout work, scene preparation,
rendering, plugin executors — and says results "return as typed messages and
are committed at deterministic synchronization points". Resource loading is
the only one with a port (`BlockingExecutor`, spec §3.3). Until now nothing
in the workspace could put a thread behind it: the only implementations were
the headless `DeterministicExecutor`, which by contract runs jobs inline on
the caller, and a test stub.

Every assertion here is on `ThreadId`, not on the existence of a spawner. A
spawner that queues the job and runs it on the caller — which is exactly what
the deterministic one does, deliberately — satisfies every weaker check.
