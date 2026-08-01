Measures the preferred size of `node`'s content under `constraints`,
within the ambient [`LayoutEnvironment`].

The returned size should already lie within `constraints`; layout will
nonetheless [`BoxConstraints::constrain`] it defensively, so a provider
need not treat the bounds as an exact contract.
