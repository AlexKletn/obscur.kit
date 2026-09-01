Attaches `metrics`, and the installed custom-value grammar, to `ctx`.

Both walks rebuild their context on reaching the root element (to swap in
the real `rem` base), and a rebuild that forgot a port would silently
un-resolve `ch`, or every `var()` whose custom property was declared as
plain text, for the whole document *below the root* while the root itself
stayed correct — a bug that hides in exactly the tree shape tests use.
Funnelling every construction through here is what stops that.
