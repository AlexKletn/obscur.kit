§6.11 Shell integration: "`ShellProvider` may open:" / "- file;" /
"- URI;" / "- document handle." with "Results:" / "- opened;" / "- denied;"
/ "- unsupported;" / "- failed."

A build with no shell integration reports exactly `unsupported` — a value of
the result type, not a bare `Ok(())`.

WEAKENED 2026-07-19: this originally opened with a half asserting that
`RecordingShellProvider` answers `failed`/`unsupported` for
`obscur-nonexistent-scheme://x`. That subject is a TEST DOUBLE, documented at
shell_provider.rs:63-90 as a recorder whose whole contract is to accept every
target, push it onto a list and answer `Opened` so a caller can assert what a
component asked to open. The half therefore tested the mock, not the kit, and
making it pass would have meant teaching a recorder to adjudicate schemes.
§6.11 lists the four result values and says nothing about which provider
returns which for an unregistered scheme. Candidate spec amendment: state
whether a `ShellProvider` is obliged to pre-check handler availability
(`can_open`) before reporting `opened`, or whether `opened` means only
"handed to the platform".
