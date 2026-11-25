Maps this backend error into the domain's [`WindowError`] (spec §2.7).

Every winit-side failure is a platform failure from the domain's point of
view, so each variant maps to [`WindowError::BackendFailure`] carrying the
human-readable reason; [`WinitBackendError::NoActiveEventLoop`] carries the
fixed message `"no active event loop"`.
