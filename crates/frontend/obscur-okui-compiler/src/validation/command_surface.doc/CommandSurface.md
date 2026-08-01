The commands a document may bind a handler to — the declared side of §2.13's
"Unknown attributes, events and **string method names** are compile errors."

# What is being checked, and why nothing checked it

`on:click=increment` names a handler by string. The front ends hash that string
(`name_binding::fallback_symbol`, pinned equal to `InternedSymbol::named` by a
drift test) into the command symbol the runtime dispatches on, and the runtime
matches it against whatever an application registered. The hashing is sound;
what was missing is that **nothing ever asked whether the name is claimed by
anybody**. A misspelled handler was a silent no-op: no compile error, no
runtime diagnostic, a button that does nothing. That is precisely the failure
§2.13's sentence exists to prevent, and it is the one member of its list —
attributes, events, string method names — that was unenforced.

# Why the declaration comes from the caller

A command is claimed by a *runtime registration*
(`UiRuntime::on_command`/`on_command_at`, `App::on_command`), which is a value
that exists when the application runs — after every compiler has finished. No
pass over the document can discover it, so the checkable form of the rule is
the one §2.13 already uses for capabilities: the **caller declares** what
exists, and the compiler holds the document to it.
[`Compiler::granting`](crate::compile::Compiler::granting) does exactly this for
`requires="camera"`; [`Compiler::claiming_command`](crate::compile::Compiler::claiming_command)
does it for handlers.

# Why the default is [`Unchecked`](Self::Unchecked) and not an empty set

Because a false "unknown command" is worse than a silent no-op. A document
carries no declaration of its own — §2.13 names no authoring surface for one,
and inventing syntax the spec does not describe is not this type's business —
so a caller that declares nothing has told the compiler nothing, which is not
the same as telling it that no command exists. The two states are therefore
named rather than distinguished by emptiness:
[`Declared`](Self::Declared)`(vec![])` means "this document may bind no handler
at all", and that is a real, checkable claim a caller can make.

# The half that stays unreachable

The `okui!` macro cannot use this, and no wiring will change that: it expands
while the application crate is still compiling, so the registrations it would
check against do not exist yet, and a proc-macro has no argument to be handed a
surface through. See
[`ContractRegistry::register_surface`](crate::compile::ContractRegistry::register_surface),
whose reachability analysis is the same one, component-for-command.
