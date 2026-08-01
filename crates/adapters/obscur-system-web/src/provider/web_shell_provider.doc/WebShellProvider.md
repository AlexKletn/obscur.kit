A web [`ShellProvider`] backed by `window.open` (spec §6.11).

The provider is stateless: it forwards each target to the browser's
`window.open`. Because pop-ups may be blocked by the browser unless the call
originates from a user gesture, a successful spawn is not guaranteed — a
blocked open is reported as [`ShellResult::Failed`].
