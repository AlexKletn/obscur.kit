Declares that `name` is a command a handler in this document may bind
(fluent; spec §2.13).

Declaring the first command **closes** the command surface: from then on an
`on:<event>=<handler>` naming anything undeclared is a
[`Validation`](CompileDiagnostic::Validation) diagnostic instead of the
silent no-op it used to be. Declare every command the application
registers, or none — a partial list reports the rest as unknown.

This is the same shape as [`granting`](Self::granting), and for the same
reason: a command is claimed by a *runtime registration*
(`UiRuntime::on_command`, `App::on_command`), which is a value that exists
only once the application runs, so no pass over the document can discover
it. The caller knows; the compiler is told. See [`CommandSurface`] for why
the default is "unchecked" rather than "nothing is claimed", and for the
half of the problem the `okui!` macro cannot reach.
