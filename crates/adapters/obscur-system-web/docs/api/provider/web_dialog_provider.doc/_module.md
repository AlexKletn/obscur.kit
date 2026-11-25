A web [`DialogProvider`] backed by `window.confirm`/`window.alert`
(spec §6.9).

[`WebDialogProvider`] implements the pure
[`DialogProvider`](obscur_system::dialog::DialogProvider) port over the
synchronous browser dialogs [`Window::confirm_with_message`](web_sys::Window::confirm_with_message)
and [`Window::alert_with_message`](web_sys::Window::alert_with_message). Only
message and confirmation dialogs have a synchronous browser primitive: file
and folder pickers require asynchronous `<input type="file">` DOM interaction,
which cannot be expressed through this synchronous port, so those requests are
reported [`SupportLevel::Unavailable`] / [`DialogResult::Unsupported`]. Wiring
them over an async surface is a documented future refinement.

All browser calls are gated behind `#[cfg(target_arch = "wasm32")]`; on any
other target every request is [`DialogResult::Unsupported`], keeping the host
build honest.
