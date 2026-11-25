The faces to install: the application's own first, then the host's generic
`sans-serif` / `serif` / `monospace`.

This is what a runner hands to
[`install_text_stack`](Self::install_text_stack), and the reason it is not
just [`system_font_set`](Self::system_font_set): a font database can only be
asked for fonts the machine has, so a family an application ships with itself
has to enter the set here or it enters nowhere at all.

Ordering is the application's advantage. The registry keeps the first face
among equally-near cuts of a family, so an application face wins a tie
against a same-named one the machine happens to have installed — which is the
answer an application that bundled the file deliberately wants.

Without the `font-fontconfig` feature (or on a machine with no fontconfig)
this is exactly the application's own faces, and an empty set still paints:
[`install_text_stack`](Self::install_text_stack) appends the bundled
last-resort face.
