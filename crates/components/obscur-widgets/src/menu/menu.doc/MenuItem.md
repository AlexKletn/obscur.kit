A single entry in a [`Menu`].

Each item carries a display `label`, the `command` it fires when activated,
and whether it is currently [`enabled`](Self::is_enabled). Disabled items are
still rendered but are skipped by keyboard highlight navigation and cannot be
activated.
