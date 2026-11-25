A handler held so it can be re-registered at a different node without being
consumed.

[`UiRuntime::on_command_at`] takes the handler by value, and a node id is not
stable across a structural change, so a registration that has to *move* cannot
hand the closure over directly. What is registered is a trampoline over this
shared cell; re-binding installs a new trampoline over the same handler.
