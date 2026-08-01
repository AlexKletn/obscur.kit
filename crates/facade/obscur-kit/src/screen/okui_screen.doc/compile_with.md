Parses and compiles `source` against `data`, keeping the IR and the table.

Every name **currently bound** in `data` is declared to the compiler as the
screen's state, which is what lets an attribute binding reference it — and what
makes a reference to anything else a §2.13 compile error ("unknown attributes,
events and string method names are compile errors") instead of a value that
silently never arrives. So bind a name to its initial value *before* compiling;
for `App` that means writing
`screen_data` before `run`.

Body interpolations are not name-checked on this route — an unbound `{count}`
renders its own path, visibly — which is a gap in the front end's resolution
pass rather than a decision made here.

# Errors

[`ScreenError::Parse`] when the source does not parse, and
[`ScreenError::Compile`] when it parses but fails §2.13/§2.12 validation —
each carrying the diagnostics, because a screen that failed to compile must
not render as a silently empty document.
