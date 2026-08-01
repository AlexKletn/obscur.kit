§3.3 Rules — "The core is not tied to Tokio."

Tokio must appear zero times in the default-feature dependency graph of every
foundation / domain / application crate; any Tokio integration lives in a
separately named runtime-adapter crate that the core does not depend on.
