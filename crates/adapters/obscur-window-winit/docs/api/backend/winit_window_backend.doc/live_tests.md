The live end-to-end proof that a host can reach a real OS window through this
backend (spec §6.1).

Every other test in this crate is pure — it exercises a projection or a
decision with no platform underneath. That is deliberate, and it is also why
this backend went a whole campaign with zero consumers: nothing here ever
proved that `create_window` *produces* a window a host can render to, only
that the request was well-formed. This one opens a real window.

# Why there is exactly one test here

winit permits **one event loop per process, ever** — a second `build()`
returns [`EventLoopError::RecreationAttempt`](winit::error::EventLoopError)
("The event loop can't be re-created", `winit-0.30/src/error.rs:39`). A second
live test would therefore not fail, it would *silently skip*, and which of the
two skipped would depend on test-harness ordering. So the whole lifecycle —
deferred creation, materialization, handle identity, close — is asserted
across the turns of this single loop.

# It is `#[ignore]`d, and that is deliberate — read this before un-ignoring

This test **does not run** under `cargo test --workspace`. Run it explicitly:

```text
cargo test -p obscur-window-winit --lib -- --ignored a_host_reaches
```

Two independent reasons, and either alone would be enough:

1. **It would prove nothing where CI runs it.** `build()` fails with no
   display, so headlessly [`display_event_loop`] answers `None` and the test
   returns green having asserted *nothing*. A test that passes vacuously in
   the one place it runs unattended is worse than no test: it reports a
   working path it never checked.
2. **It would be a standing hazard where it *does* run.** On a developer
   machine with a display it opens and destroys two real OS windows on every
   `cargo test --workspace`. Hyprland — this project's development
   compositor — has a live SEGV in its `SSwipeUpdateEvent` handler when a
   window is created or destroyed while a touchpad gesture is in flight. That
   is Hyprland's bug and not this toolkit's, but an unignored test makes every
   routine test run roll that die, and it has already taken the compositor
   down once during this work.

So the value it has is as an *attended* proof: run by hand, on a machine with
a display, by someone who is changing this path and wants it checked. It is
the only test in this crate that proves `create_window` yields a window a host
can actually render to — every other test here, including
`native_window_is_none_until_the_handler_pumps` below, is pure and can only
prove the request was well-formed or the deferred state is honest.

**Consequence, stated plainly: the `Some` half of [`WinitWindowBackend::native_window`]
— the half the six windowed examples exist to use — has no automated
regression gate.** Nothing in CI will catch its breaking. That is a real
residual gap, not a solved problem, and it is bounded by the fact that no
display is available to CI rather than by anything this crate chose.
