Workspace architecture checks (spec §3.16, §5.8).

Usage: `cargo run -p xtask -- check` (source rules, fast) or
`cargo run -p xtask -- consumer` (builds the out-of-workspace facade
consumer, so it costs a real compile and stays out of `check`).
