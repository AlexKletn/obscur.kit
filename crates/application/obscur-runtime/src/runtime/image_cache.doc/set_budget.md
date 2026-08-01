Installs `budget` and evicts down to it, returning the URLs evicted (spec
§5.19 memory budgets).

Tightening the budget of an already-full cache takes effect immediately
rather than waiting for the next insert to notice.
