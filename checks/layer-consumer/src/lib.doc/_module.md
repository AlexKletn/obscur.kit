What an application can do with `obscur-kit` and nothing else, on a **desktop
surface**.

`checks/facade-consumer` is the same idea for a window, and everything it says
about the one-dependency rule applies here. This fixture exists because that one
opens a window, and the two runners diverge after `AppParts::boot`: nothing in
this repository built a layer surface *and* used the authoring surface on it, so
the whole layer branch went unexercised until an application outside the
repository tried it (`docs/layer-consumer-audit.md`).

**The feature set is half the check** — see `Cargo.toml`. `window-winit` is
absent, which is the only configuration where the layer runner is the sole
reader of what boot produces.

This is not a second test suite for the toolkit; it is a *boundary* check.
