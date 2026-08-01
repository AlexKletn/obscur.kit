Whether a property inherits from its parent element by default.

This captures the property's *default* inheritance behaviour — the CSS-wide
keywords `inherit`, `initial` and `unset` (modelled on
[`crate::property::PropertyValue`]) can override this per declaration during
the cascade, but the default recorded here is what `unset` resolves against.
