CSS-module isolation applied at a component / scope root.

Per §4.15, CSS-module mode applies only at a component or scope root. It
isolates the local identifiers declared inside that scope — internal class
names need not be globally unique — while keeping the normal cascade,
inheritance and custom-property (variable) behaviour intact inside the
scope. A [`ModuleScope`] is the operation that performs that local-identifier
isolation: it maps a bare local name into a [`ScopedIdent`] qualified by its
scope so it cannot clash with another module's same-named identifier.
