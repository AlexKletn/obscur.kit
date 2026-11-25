# Contributing to Obscur Kit

[Русская версия](CONTRIBUTING.ru.md) · [README](README.md)

Follow the [code quality agreement](CODE_QUALITY.md) and write commit messages in
English using [Conventional Commits](COMMIT_CONVENTIONS.md). Keep changes within
the appropriate workspace layer and consult the [crate architecture](specs/obscur-kit-2-crate-architecture/).

Before submitting a pull request, run the checks below from the workspace root.
For local build limits and the command wrapper, see [AGENTS.md](AGENTS.md).

```sh
cargo fmt --all --check
cargo test --workspace -j 2
cargo clippy --workspace --all-targets -j 2
cargo run -p xtask -- check
cargo run -p xtask -- consumer
```

Describe what changed and how you verified it. For platform-specific changes,
include the OS, display server or compositor, and device or renderer used
when relevant.
