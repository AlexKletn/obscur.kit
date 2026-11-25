Why [`OkuiScreen::compile`](super::OkuiScreen::compile) rejected a source.

The two variants are the two halves of the front end: the `.okui` **parser**
(syntax) and the **compiler** (§2.13 contract validation, §2.12 slots,
capabilities). Each carries one message per diagnostic rather than a single
joined string, so a host can report them individually — and neither is ever a
silently empty screen, which is what a `.okui` root that failed to compile used
to render as.

`AppError` converts from this, so a windowed
application sees one error type; the split is kept here because the screen
front end is display-free and usable without `App`.
