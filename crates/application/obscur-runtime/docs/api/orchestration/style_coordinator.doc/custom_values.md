The CSS grammar a substituted custom property is read against, once a host
has installed one.

[`None`] until then, which is what a runtime with no CSS pipeline has: its
custom properties are whatever a caller put in `StyleInputs` directly, and
those are already typed values.
