Taking keys that would otherwise go to the focused window.

[`LayerFocusPolicy::Exclusive`](crate::layer::LayerFocusPolicy::Exclusive)
asks for this. Where it is unavailable the honest fallback is on-demand focus,
reported as [`Emulated`](obscur_capability::SupportLevel::Emulated).
