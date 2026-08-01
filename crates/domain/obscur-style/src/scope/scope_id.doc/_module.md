Opaque identifier for a style scope.

A [`ScopeId`] names a style scope (the boundary that scopes a component's or
module's CSS, per §4.15) without carrying any behaviour. It is a lightweight
newtype over an integer so it can key maps and be compared and ordered
cheaply. The behaviour of a scope (isolation, boundary-crossing) lives on
[`crate::scope::StyleScope`]; this type is only its identity.
