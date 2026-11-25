The result of routing a keyboard event through
[`UiRuntime::handle_key`](super::UiRuntime::handle_key).

It reports the [`target`](Self::target) and the [`command`](Self::command)
resolved for a shortcut or keyboard activation. Window and application
shortcuts can resolve a command while no node has focus.
[`default_prevented`](Self::default_prevented) reports a refusal by the
raw keyboard handler.
