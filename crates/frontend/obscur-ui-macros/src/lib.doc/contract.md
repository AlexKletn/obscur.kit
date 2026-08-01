Derives a typed `ComponentContract` for a props struct (spec §2.13).

Generates an inherent `contract()` associated function describing the
component's props and events, so misuse is caught without hand-registering a
surface. Only structs with named fields are supported.
