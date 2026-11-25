Registers `handler` to run when `command` reaches the scope `id` on
§2.6's routing chain — link 2 — replacing any handler previously
registered for that `(scope, command)` pair.

The handler is dropped when the scope is closed
([`close_focus_scope`](Self::close_focus_scope)). Registering for a scope
that is not open is harmless: the route can never name it, so the handler
simply never fires until that scope opens.
Like [`on_command`](Self::on_command), `handler` may take nothing or take
the propagating [`UiEvent`] (see [`IntoCommandHandler`]).

[`UiEvent`]: obscur_interaction::event::UiEvent
