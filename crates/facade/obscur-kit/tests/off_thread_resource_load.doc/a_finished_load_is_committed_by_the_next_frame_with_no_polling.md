The commit claim on its own, with no threads and no timing: a result that has
come back is committed by the **frame**, without anyone calling
`poll_resource_tasks` (§1.5 "committed at deterministic synchronization
points").

The deterministic executor is the instrument that makes this exact: `run()`
runs the queued job inline, so the typed message is provably on the channel
before the next frame starts — where the off-thread test above can only wait
for a pool. `run_tasks` is deliberately not called; it is the method that
polls, and polling is the discipline under test.
