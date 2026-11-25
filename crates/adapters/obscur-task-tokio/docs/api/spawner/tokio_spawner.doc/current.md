Creates a spawner bound to the Tokio runtime of the current context.

# Panics

Panics if called outside a Tokio runtime context, matching the standard
behavior of [`Handle::current`].
