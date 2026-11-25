How a style scope behaves with respect to isolation.

Per §4.15, global stylesheets are allowed but component stylesheets are
scoped by default; a component may additionally opt into CSS-module
isolation of its local identifiers, or explicitly declare itself open /
global. [`ScopeKind`] models that four-way behaviour classification.
