Whether this element's computed `direction` is `rtl`.

Cascaded value first, then the parent's computed one (`direction` inherits),
then the CSS initial `ltr`. This is the same resolution the `Direction`
property gets in the resolver's per-property loop; it is done early because
[`cascaded_for`] depends on it.
