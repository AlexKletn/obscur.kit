Installs the built-in **user-agent (UA) default** stylesheet — the
toolkit's overridable base component styles.

Unlike [`set_stylesheets`](Self::set_stylesheets), the default sheet is a
separate slot that is matched on **every** render path (`run_frame`,
`run_due_frame`, `render_now`, and the reactive path), *always first* —
prepended below whatever author sheets that frame matches. Because it
carries the lowest cascade origin
([`CascadeOrigin::SystemDefault`](obscur_style::cascade::CascadeOrigin::SystemDefault)),
any author/theme declaration of the same or higher specificity overrides it,
exactly like a browser's UA stylesheet (§4.14).

Passing a sheet at any other origin still works — only the matched *order*
is fixed (default first); precedence is governed entirely by the origins
carried on the rules. Not calling this leaves the runtime with **no** UA
sheet, in which case matching is byte-identical to before (the author sheets
alone).
