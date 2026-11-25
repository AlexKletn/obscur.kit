Releases every subscription this scope owns.

Called automatically when the scope is dropped; exposed so a component
can also tear its subscriptions down eagerly without dropping the scope.
