Maps this backend failure into the domain [`RendererError`] (spec §5.2).

A failed device creation and a missing adapter both mean there is no
usable GPU device, so they become a fatal
[`RendererError::DeviceLost`]. The surface and frame failures are generic
backend faults and become [`RendererError::Backend`], preserving their
reason text.
