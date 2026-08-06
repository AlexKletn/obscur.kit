# obscur.kit_v2 — working notes for agents

## Building: cap the memory, and never as a child of the terminal

This box has 30 GiB of RAM and runs **foot as a client-server** terminal: a
system-wide OOM kill takes down every terminal window at once, including the one
you are working in. A `cargo test --workspace` here peaks at **~8 GiB**, so that
is not a theoretical risk — it has happened repeatedly.

**Run every cargo invocation inside its own transient systemd unit with a memory
ceiling.** Not `cargo …` directly, and not `systemd-run --scope` either: a scope
is still a child of this shell. A `--user` *service* is a separate process tree
in its own cgroup, so a kill is confined to it.

```bash
UNIT=kitbuild-$$
LOG=/tmp/$UNIT.log
systemd-run --user --quiet --wait --collect --unit="$UNIT" \
  -p MemoryHigh=8G -p MemoryMax=10G \
  -p WorkingDirectory=/home/alex/projects/obscur.kit_v2 \
  -- /bin/bash -c "taskset -c 12-19 cargo test --workspace -j 2 > $LOG 2>&1
     cat \"/sys/fs/cgroup\$(awk -F: '{print \$3}' /proc/self/cgroup)/memory.peak\" \
       | awk '{print \"[peak] \" \$1/1048576 \" MB\"}' >> $LOG"
cat "$LOG"
```

`MemoryHigh` throttles first — the build slows under reclaim pressure rather
than dying. `MemoryMax` is the hard wall, and only then does the kernel kill,
inside that unit. Reading `memory.peak` afterwards is how you learn whether a
change made the build hungrier.

**Also:**

- `taskset -c 12-19` pins to the E-cores (0–11 are P-cores, 20–21 LP-E). The
  owner asked explicitly not to build on all cores.
- `-j 2` (or 3). `.cargo/config.toml` is machine-local and git-ignored; it
  already pins `jobs` and moves `TMPDIR` off the `/tmp` tmpfs, where a linker
  once died with SIGBUS after Chromium filled it.
- Prefer `cargo check` while iterating — the whole workspace checks in under
  200 MB. Reach for `cargo test` when you actually need to run something.

**Measured peaks** (`-j 2`), so you can tell normal from a regression:

| command | peak |
| --- | --- |
| `cargo check --workspace` | 0.2 GB |
| `cargo clippy --workspace --all-targets` | 1.8 GB |
| `cargo run -p xtask -- consumer` | 1.5 GB |
| `cargo test --workspace` | **8.2 GB** |

The test build is the peak because of **168 integration-test binaries** (139 in
`obscur-kit` alone), each statically linking the whole graph with full debug
info. It is the link count, not the crate count. If it needs to come down, the
levers in order are: fold `tests/*.rs` into fewer binaries with `mod`;
`debug = "line-tables-only"`; the `ld.lld` linker (installed); then
`split-debuginfo = "unpacked"`.

## Verifying a change

All five, all through the wrapper above:

```
cargo test --workspace -j 2      # 6631 passing as of 2026-08-04, 0 failing
cargo clippy --workspace --all-targets -j 2   # 360 warnings is the baseline
cargo fmt --all --check
cargo run -p xtask -- check      # module layout, file size, dependency direction
cargo run -p xtask -- consumer   # an out-of-workspace consumer of the facade
```

A rising clippy count or a new `xtask` violation is a regression, not noise.

## House rules that are easy to violate by accident

- **Documentation lives outside the code.** Every item's docs are a file under
  `<file>.doc/*.md`, pulled in with `#[doc = include_str!(…)]`. Comments in code
  carry only the *why*.
- **One entity per file** (crate architecture §3.1); `xtask check` enforces it.
- **Layering**: foundation → domain → application → frontend/components →
  adapters → facade. `obscur-runtime` may not depend on `obscur-css`; that is
  deliberate, and why the `style` attribute reaches it as an installed port.
- **Spec-derived tests are the oracle.** Never edit one to make an
  implementation pass — ask first.
- **Measure before claiming a performance win.** Several confident hypotheses
  here have been wrong; the last two real wins were found by taking a slow path
  apart, not by reasoning about it.
