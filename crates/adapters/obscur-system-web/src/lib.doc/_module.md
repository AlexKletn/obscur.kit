Web/WASM system-service provider adapters (spec §6.9 dialogs, §6.11 shell,
§6.12 notifications; §2.7 target-specific providers).

This crate adapts the `obscur_system` dialog, shell, and notification ports to
the browser by calling the corresponding `web-sys` bindings (`window.open`,
`window.confirm`/`window.alert`, and the `Notification` API). Those bindings
only exist on `wasm32`, so every browser call is gated behind
`#[cfg(target_arch = "wasm32")]` with a `#[cfg(not(target_arch = "wasm32"))]`
fallback that returns the port's honest "unsupported" outcome. The crate
therefore compiles cleanly on the default `x86_64` host target (where it is
inert) as well as on `wasm32-unknown-unknown` (where the real browser calls
run).

Because the ports are synchronous, only the browser facilities that are
synchronously available are wired up: `window.open` for the shell, and
`window.confirm`/`window.alert` plus the (already-granted) `Notification`
constructor. Anything that requires asynchronous DOM interaction — file inputs
for dialogs, the async notification permission request, the Badging API — is
reported as unsupported and documented as a future refinement.
