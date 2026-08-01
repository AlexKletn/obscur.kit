The worker thread driving the executor. `Option` so [`Drop`] can take and
join it exactly once.
