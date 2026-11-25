[`CounterComponent`]'s decoded props.

Public because it is the props type of a *second* fixture too: the guarded
component takes the same `Props`, so the same undecodable `step` fails both, and
a boundary test can change the policy while holding the failure identical.
