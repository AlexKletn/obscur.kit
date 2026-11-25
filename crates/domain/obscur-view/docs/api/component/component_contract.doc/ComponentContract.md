The typed, exported contract of a component (spec §2.13).

A contract is the component's public surface, aggregating the sub-contracts
from the [`contract`](crate::contract) module: the props it accepts (and
which carry defaults), the events it exposes, the slots and allowed
children, its style parts, and the capabilities it depends on. It also
carries diagnostics metadata: the [`DiagnosticSubsystem`] failures are
attributed to (defaulting to [`DiagnosticSubsystem::View`]) and an optional
stable component name.

Everything here is plain, comparable data — it derives `Eq`, so no field may
embed a float (the sub-contract types are all `Eq`).
