§7.3/§7.4's reproducibility, which the change above must not have bought its
parallelism with: the headless host's deterministic executor still runs every
load on the caller's thread, when `run_tasks` says so and never before.
