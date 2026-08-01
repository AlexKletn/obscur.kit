Adds `name` to the contract's allowed (optional) props (fluent).

The prop surface is closed once the contract declares any prop (required
or allowed): only listed props, required props, reserved structural
attributes and `on:*` events are then permitted; every other attribute is
an unknown-attribute violation.
