The typed failure of a [`Renderer`](crate::renderer::Renderer) operation
(spec §5.2).

Spec §5.2 requires that unsupported effects and fallbacks be **explicit
diagnostics** rather than silent failures or panics. Each variant names a
distinct, actionable cause: a lost device the backend must handle, an effect
the backend cannot reproduce (the explicit fallback signal), an invalid
target, an out-of-memory condition, or a generic backend failure carrying a
human-readable reason.
