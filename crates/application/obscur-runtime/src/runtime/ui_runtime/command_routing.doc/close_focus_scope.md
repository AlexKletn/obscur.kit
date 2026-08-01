Closes the scope with this `id`, returning whether one was open.

Its [`on_scope_command`](Self::on_scope_command) handlers are dropped
with it, so a closed dialog's commands stop routing.
