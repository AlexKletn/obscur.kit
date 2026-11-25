Appends a diagnostic for every `on:<event>` handler given as a string
literal — string-based method dispatch, which §2.3/§2.13 forbid.

A handler is bound as an expression (`on:pressed={handle_submit}`) or, on
the file path, a bare reference (`on:pressed=handle_submit`); a *quoted*
value names a method by string, and that is the form refused. A prop
attribute's string value is untouched — only an event's is a dispatch
reference.
