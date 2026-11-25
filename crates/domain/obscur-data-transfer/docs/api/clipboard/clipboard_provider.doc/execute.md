Performs `request` against the clipboard, reporting the honest outcome
(spec §6.20).

This is the unified copy/cut/paste/clear entry point; it never panics on
an unsatisfiable request, returning the appropriate non-success
[`ClipboardResult`] instead.
