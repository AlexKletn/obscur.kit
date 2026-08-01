A stable `automation-id` string (spec §7.5).

Separate from [`TestId`](Self::TestId) because §7.5 lists them separately
and they answer to different owners: a test-id is the app's own and renames
freely; an automation-id is a contract with software outside the codebase.
A harness that targets one must not accidentally match the other.
