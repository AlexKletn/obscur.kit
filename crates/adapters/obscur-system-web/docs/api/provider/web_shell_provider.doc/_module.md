A web [`ShellProvider`] that opens targets via `window.open` (spec §6.11).

[`WebShellProvider`] implements the pure
[`ShellProvider`](obscur_system::shell::ShellProvider) port in the browser by
calling [`Window::open_with_url`](web_sys::Window::open_with_url). The open is
best-effort: a browser may block the pop-up (for example when the call is not
made in direct response to a user gesture), in which case `open` returns no
window handle and this provider reports [`ShellResult::Failed`].

The `window.open` binding only exists on `wasm32`; on any other target the
provider reports [`ShellResult::Unsupported`] and [`Self::can_open`] is
`false`, so the crate stays honest on the default host build.
