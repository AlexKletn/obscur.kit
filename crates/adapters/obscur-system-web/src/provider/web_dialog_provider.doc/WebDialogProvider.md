A web [`DialogProvider`] backed by `window.confirm`/`window.alert`
(spec §6.9).

The provider is stateless. [`Self::origin`] is `"web"`; [`Self::supports`]
reports [`SupportLevel::Native`] for message and confirmation dialogs and
[`SupportLevel::Unavailable`] for file/folder dialogs (which need async DOM
interaction); and [`Self::present`] drives the corresponding synchronous
browser dialog.
