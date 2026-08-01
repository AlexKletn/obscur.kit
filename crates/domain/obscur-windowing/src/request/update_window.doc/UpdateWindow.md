A delta of property changes to apply to an existing window (spec §6.2).

Spec §6.2 makes window properties dynamically mutable. An `UpdateWindow`
carries only the changes an application wants to make: every property is an
`Option`, and a `None` field means "leave this property unchanged". Applying
the delta with [`UpdateWindow::apply`] writes only the fields that are set.

The [`UpdateWindow::position`] field is two-level to distinguish "do not
touch the position" from "clear the explicit position and let the platform
choose": the outer `None` leaves the position unchanged, while an outer
`Some(None)` clears it to a platform-chosen position (matching
[`WindowProperties::position`]).
