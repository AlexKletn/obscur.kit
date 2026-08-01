Adds `name` to the contract's allowed `on:<event>` names (fluent).

Adding any allowed event closes the event surface: once the list is
non-empty, only listed events are permitted; every other `on:*` binding
is an unknown-event violation.
