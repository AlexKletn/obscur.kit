End-to-end: a `.okui` handler reference is **checked** against the commands a
caller declares (spec §2.13, "Unknown attributes, events and string method
names are compile errors").

# The failure this closes

`on:click=increment` names a handler by string. Both front ends hash that
string into a command symbol (`name_binding::fallback_symbol`, pinned equal to
`InternedSymbol::named` by `okui_name_binding_drift.rs`) and the runtime matches
it against whatever the application registered. The hashing was never the
problem; the problem was that **nothing asked whether the name was claimed by
anybody**, so a misspelling was a silent no-op — no compile error, no runtime
diagnostic, a button that does nothing.

# Where the check can and cannot live

A command is claimed by a runtime registration (`App::on_command`,
`UiRuntime::on_command_at`), which is a value that exists only once the
application runs. No compiler can discover it, so the checkable form of the rule
is the one §2.13 already uses for capabilities: the caller declares what exists
and the compiler holds the document to it. That is
`Compiler::claiming_command` / `Compiler::with_commands`, exercised below
through the facade's own re-export.

The `okui!` macro is the documented residue: it expands while the application
crate is still compiling, so the registrations do not exist yet and it has no
argument to be handed a surface through. It must not guess — a false "unknown
command" is worse than the no-op — so `okui!` keeps reporting nothing here, and
`a_macro_binding_is_deliberately_not_checked` pins that as a decision rather
than an omission.
