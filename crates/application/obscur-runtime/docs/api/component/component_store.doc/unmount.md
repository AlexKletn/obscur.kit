Unmounts and drops the instance at `node`, if one is mounted.

The hook runs while the component's state is still readable; dropping the
value afterwards releases that state and the scoped subscriptions.
