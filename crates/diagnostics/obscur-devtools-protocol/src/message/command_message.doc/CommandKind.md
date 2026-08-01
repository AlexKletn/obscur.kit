The typed body of a [`CommandMessage`].

Each variant maps to exactly one [`PermissionScope`] via
[`required_scope`](CommandKind::required_scope), the single source of truth
the server uses to authorize a command.
