What an application can do with `obscur-kit` and nothing else.

Every item here is written the way `docs/api/README.md` tells an application
to write it: one dependency, the re-exports, no internal crate named
anywhere. That constraint is the point — see `Cargo.toml`.

This is not a second test suite for the toolkit; it is a *boundary* check, and
it fails for one reason: some authoring surface needs something an application
does not have. The layout follows the workspace's own §3.1 convention (a
structural `lib.rs`) so `xtask`'s rules can be run over this directory too.
