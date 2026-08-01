Applies the presentation `mode` (fullscreen, kiosk, and related policy)
to window `id` (spec §6.5).

`mode` is borrowed because the caller may reuse it across windows.
Reports the honest [`WindowOperationResult`], including the
[`SupportLevel`] for an emulated or conditionally-supported mode.
