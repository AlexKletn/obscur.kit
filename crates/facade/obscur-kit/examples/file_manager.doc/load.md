Reads `dir`, sorting directories first and then by name. A read failure is
reported in the status line and leaves the previous listing in place — the
§1.x recoverable-failure posture, not a panic.
