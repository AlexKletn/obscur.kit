The action invoked after each scheduled delay elapses. Shared so the
timer stays [`Clone`] and each spawned task can hold its own reference.
