Tears the instance down to [`LifecycleState::Unmounted`] and clears its
subscription scope (automatic cleanup, spec §2.1).

Valid from any active phase (`Mounted` or `Updating`). Returns `true` if
the transition applied; an already-unmounted or never-mounted instance
returns `false` and is left untouched.
