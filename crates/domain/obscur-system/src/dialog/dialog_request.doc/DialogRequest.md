A request to present a dialog to the user (spec §6.9).

A [`DialogProvider`](crate::dialog::DialogProvider) consumes a request and
answers with a [`DialogResult`](crate::dialog::DialogResult). The request is a
pure value describing *what* to ask — file pickers, a save target, a folder
picker, an informational message, or a confirmation — never *how* the platform
renders it. A provider may back the same request with a native dialog, an
external portal, or a kit-rendered surface.
